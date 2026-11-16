#[path = "agent_administration_postgres/public_profiles.rs"]
mod public_profiles;
#[path = "agent_administration_postgres/research_quota.rs"]
mod research_quota;
#[path = "developer_applications_postgres/support.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use peerfora_api::{app::build_app, database::connect_database, utils::one_time_token};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{create_account_fixture, response_json, send_json_request, test_app_state};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn agent_review_requires_permission_and_audits_capability_decisions() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply migrations");
    let operator = create_account_fixture(&database, "agent_review").await;
    let app = build_app(test_app_state(database.clone(), database_url));
    let registration = send_json_request(
        &app,
        "POST",
        "/api/v1/agents",
        &operator.session_token,
        json!({
            "handle":"reviewable_agent",
            "displayName":"Reviewable Agent",
            "description":"Agent submitted for moderation review.",
            "requestedCapabilities":["research", "cite_sources"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);
    let agent: Value = response_json(registration).await;
    let agent_id = agent["id"].as_str().expect("agent actor ID");

    let student_cannot_list = send_json_request(
        &app,
        "GET",
        "/api/v1/admin/agents?status=pending_review",
        &operator.session_token,
        json!({}),
    )
    .await;
    assert_eq!(student_cannot_list.status(), StatusCode::UNAUTHORIZED);

    let (_, reviewer_token) = create_administrator_session(&database, "agent_reviewer").await;
    let (_, support_token) = create_administrator_session(&database, "support_agent").await;
    let (_, trust_safety_token) =
        create_administrator_session(&database, "trust_safety_lead").await;
    let list = send_admin_request(
        &app,
        "/api/v1/admin/agents?status=pending_review&limit=20&offset=0",
        &reviewer_token,
    )
    .await;
    assert_eq!(list.status(), StatusCode::OK);
    let list: Value = response_json(list).await;
    assert!(list["totalCount"].as_i64().is_some_and(|count| count >= 1));
    let listed_agent = list["items"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == agent_id))
        .expect("operator's pending agent should appear in the review queue");
    assert_eq!(
        listed_agent["requestedCapabilities"],
        json!(["cite_sources", "research"])
    );

    let agent_path = format!("/api/v1/admin/agents/{agent_id}/review");
    let forbidden = send_admin_json_request(
        &app,
        &agent_path,
        &support_token,
        json!({"decision":"approve", "approvedCapabilities":["research"], "reason":"Support cannot review agents."}),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let unrequested_capability = send_admin_json_request(
        &app,
        &agent_path,
        &reviewer_token,
        json!({"decision":"approve", "approvedCapabilities":["publish_posts"], "reason":"Only requested capabilities may be approved."}),
    )
    .await;
    assert_eq!(unrequested_capability.status(), StatusCode::BAD_REQUEST);

    let review = send_admin_json_request(
        &app,
        &agent_path,
        &reviewer_token,
        json!({"decision":"approve", "approvedCapabilities":["research"], "reason":"Research capability is appropriate for this agent."}),
    )
    .await;
    assert_eq!(review.status(), StatusCode::NO_CONTENT);
    let reviewed_status: String =
        sqlx::query_scalar("SELECT review_status FROM agent_actor_identities WHERE actor_id = $1")
            .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
            .fetch_one(&database)
            .await
            .expect("agent review should persist");
    assert_eq!(reviewed_status, "approved");
    let decisions: Vec<(String, String)> = sqlx::query_as(
        "SELECT capability_key, request_status FROM agent_capability_requests
         WHERE agent_actor_id = $1 ORDER BY capability_key",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_all(&database)
    .await
    .expect("capability decisions should persist");
    assert_eq!(
        decisions,
        vec![
            ("cite_sources".to_owned(), "denied".to_owned()),
            ("research".to_owned(), "approved".to_owned())
        ]
    );

    let duplicate_review = send_admin_json_request(
        &app,
        &agent_path,
        &reviewer_token,
        json!({"decision":"approve", "approvedCapabilities":["research"], "reason":"Repeated decision must conflict."}),
    )
    .await;
    assert_eq!(duplicate_review.status(), StatusCode::CONFLICT);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs
         WHERE target_id = $1 AND action = 'agents.registration.approved'",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("agent review should be audited");
    assert_eq!(audit_count, 1);

    let credential = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/agents/{agent_id}/credentials"),
        &operator.session_token,
        json!({"name":"Lifecycle credential", "capabilities":["research"]}),
    )
    .await;
    assert_eq!(credential.status(), StatusCode::CREATED);
    let credential: Value = response_json(credential).await;
    let credential_id = credential["credential"]["id"]
        .as_str()
        .expect("credential ID");

    let project = send_json_request(
        &app,
        "POST",
        "/api/v1/research/projects",
        &operator.session_token,
        json!({"title":"Agent lifecycle task release"}),
    )
    .await;
    assert_eq!(project.status(), StatusCode::CREATED);
    let project: Value = response_json(project).await;
    let project_id = project["id"].as_str().expect("project ID");
    let task = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/research/projects/{project_id}/tasks"),
        &operator.session_token,
        json!({
            "title":"Lifecycle task release",
            "objective":"Ensure suspension frees active agent work.",
            "scope":"The task is a synthetic integration fixture.",
            "expectedDeliverables":["A short evidence summary"]
        }),
    )
    .await;
    assert_eq!(task.status(), StatusCode::CREATED);
    let task: Value = response_json(task).await;
    let task_id = task["id"].as_str().expect("task ID");
    for (suffix, body) in [
        ("queue", json!({})),
        ("assign-agent", json!({"agentId":agent_id})),
    ] {
        let response = send_json_request(
            &app,
            "POST",
            &format!("/api/v1/research/tasks/{task_id}/{suffix}"),
            &operator.session_token,
            body,
        )
        .await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let lifecycle_path = format!("/api/v1/admin/agents/{agent_id}/lifecycle");
    let reviewer_cannot_manage = send_admin_json_request(
        &app,
        &lifecycle_path,
        &reviewer_token,
        json!({"action":"suspend", "reason":"Reviewers cannot manage agent status."}),
    )
    .await;
    assert_eq!(reviewer_cannot_manage.status(), StatusCode::FORBIDDEN);
    let support_cannot_manage = send_admin_json_request(
        &app,
        &lifecycle_path,
        &support_token,
        json!({"action":"suspend", "reason":"Support cannot manage agent status."}),
    )
    .await;
    assert_eq!(support_cannot_manage.status(), StatusCode::FORBIDDEN);

    let suspension = send_admin_json_request(
        &app,
        &lifecycle_path,
        &trust_safety_token,
        json!({"action":"suspend", "reason":"Pause this agent during an investigation."}),
    )
    .await;
    assert_eq!(suspension.status(), StatusCode::NO_CONTENT);
    let (status, revoked_at): (String, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT agent.review_status, credential.revoked_at
            FROM agent_actor_identities agent
            JOIN agent_machine_credentials credential ON credential.agent_actor_id = agent.actor_id
            WHERE agent.actor_id = $1 AND credential.id = $2",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .bind(credential_id.parse::<Uuid>().expect("credential UUID"))
    .fetch_one(&database)
    .await
    .expect("suspension should revoke the credential");
    assert_eq!(status, "suspended");
    assert!(revoked_at.is_some());
    let (task_status, task_assignee): (String, Option<Uuid>) =
        sqlx::query_as("SELECT task_status, assigned_actor_id FROM research_tasks WHERE id = $1")
            .bind(task_id.parse::<Uuid>().expect("task UUID"))
            .fetch_one(&database)
            .await
            .expect("suspension should release active agent tasks");
    assert_eq!(task_status, "queued");
    assert!(task_assignee.is_none());

    let reinstate = send_admin_json_request(
        &app,
        &lifecycle_path,
        &trust_safety_token,
        json!({"action":"reinstate", "reason":"The investigation is complete."}),
    )
    .await;
    assert_eq!(reinstate.status(), StatusCode::NO_CONTENT);
    let reinstated_status: String =
        sqlx::query_scalar("SELECT review_status FROM agent_actor_identities WHERE actor_id = $1")
            .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
            .fetch_one(&database)
            .await
            .expect("reinstatement should persist");
    assert_eq!(reinstated_status, "approved");

    let replacement_credential = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/agents/{agent_id}/credentials"),
        &operator.session_token,
        json!({"name":"Replacement credential", "capabilities":["research"]}),
    )
    .await;
    assert_eq!(replacement_credential.status(), StatusCode::CREATED);
    let revoke = send_admin_json_request(
        &app,
        &lifecycle_path,
        &trust_safety_token,
        json!({"action":"revoke", "reason":"This agent is no longer authorized."}),
    )
    .await;
    assert_eq!(revoke.status(), StatusCode::NO_CONTENT);
    let revoked_status: String =
        sqlx::query_scalar("SELECT review_status FROM agent_actor_identities WHERE actor_id = $1")
            .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
            .fetch_one(&database)
            .await
            .expect("revocation should persist");
    assert_eq!(revoked_status, "revoked");
    let active_credential_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_machine_credentials
        WHERE agent_actor_id = $1 AND revoked_at IS NULL",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("revoked agent credentials should be counted");
    assert_eq!(active_credential_count, 0);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs
        WHERE target_id = $1 AND action LIKE 'agents.lifecycle.%'",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("lifecycle decisions should be audited");
    assert_eq!(audit_count, 3);
}

async fn create_administrator_session(database: &PgPool, role_key: &str) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
         VALUES ($1, $2, 'integration-test-hash', 'test-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("administrator fixture should persist");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(role_key)
    .bind(
        peerfora_api::constants::admin_auth::SEEDED_ROOT_ADMINISTRATOR_ID
            .parse::<Uuid>()
            .expect("seeded root ID should be valid"),
    )
    .execute(database)
    .await
    .expect("administrator role fixture should persist");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
         VALUES ($1, $2, $3, now() + interval '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(one_time_token::hash_one_time_token(&token))
    .execute(database)
    .await
    .expect("administrator session should persist");
    (administrator_id, token)
}

async fn send_admin_request(
    app: &axum::Router,
    path: &str,
    token: &str,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header(header::COOKIE, format!("peerfora_admin_session={token}"))
                .body(Body::empty())
                .expect("administrator request should build"),
        )
        .await
        .expect("router should return an administrator response")
}

async fn send_admin_json_request(
    app: &axum::Router,
    path: &str,
    token: &str,
    body: Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::COOKIE, format!("peerfora_admin_session={token}"))
                .header(header::ORIGIN, support::TEST_ORIGIN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&body).expect("JSON should serialize"),
                ))
                .expect("administrator review request should build"),
        )
        .await
        .expect("router should return an administrator response")
}
