#[path = "agent_research_tasks_postgres/research_finding_flow.rs"]
mod research_finding_flow;
#[path = "agent_research_tasks_postgres/research_submission_quota.rs"]
mod research_submission_quota;
#[path = "developer_applications_postgres/support.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use peerfora_api::{
    app::build_app, constants::admin_auth, database::connect_database, utils::one_time_token,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{create_account_fixture, response_json, send_json_request, test_app_state};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn approved_agent_completes_assigned_research_task_with_reviewable_sources() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply all migrations");
    let operator = create_account_fixture(&database, "agent_research_owner").await;
    let reviewer = create_account_fixture(&database, "agent_research_reviewer").await;
    let operator_username: String = sqlx::query_scalar(
        "SELECT username FROM accounts WHERE username LIKE 'agent_research_owner_%'
        ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&database)
    .await
    .expect("operator username should exist");
    let reviewer_username: String = sqlx::query_scalar(
        "UPDATE accounts SET username = left(username, 24)
        WHERE id = (SELECT id FROM accounts
            WHERE username LIKE 'agent_research_reviewer_%'
            ORDER BY created_at DESC LIMIT 1)
        RETURNING username",
    )
    .fetch_one(&database)
    .await
    .expect("reviewer account should have a valid invitation username");
    let app = build_app(test_app_state(database.clone(), database_url));
    let registration = send_json_request(
        &app,
        "POST",
        "/api/v1/agents",
        &operator.session_token,
        json!({
            "handle":"research_task_agent",
            "displayName":"Research Task Agent",
            "modelName":"Peerfora Research Fixture",
            "modelVersion":"fixture-1",
            "supportedLanguages":["en", "en-GB"],
            "requestedCapabilities":["research"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);
    let agent: Value = response_json(registration).await;
    let agent_id = agent["id"].as_str().expect("agent actor ID");
    sqlx::query(
        "UPDATE agent_actor_identities SET research_submissions_per_24_hours = 2
        WHERE actor_id = $1",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .execute(&database)
    .await
    .expect("fixture quota should be set");
    let (_, admin_token) = create_agent_reviewer_session(&database).await;
    let approval = send_admin_request(
        &app,
        &format!("/api/v1/admin/agents/{agent_id}/review"),
        &admin_token,
        json!({
            "decision":"approve",
            "approvedCapabilities":["research"],
            "reason":"Research execution is limited to assigned tasks."
        }),
    )
    .await;
    assert_eq!(approval.status(), StatusCode::NO_CONTENT);
    let credential = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/agents/{agent_id}/credentials"),
        &operator.session_token,
        json!({"name":"Assigned task runner", "capabilities":["research"]}),
    )
    .await;
    assert_eq!(credential.status(), StatusCode::CREATED);
    let credential: Value = response_json(credential).await;
    let secret = credential["secret"].as_str().expect("one-time credential");

    let project = send_json_request(
        &app,
        "POST",
        "/api/v1/research/projects",
        &operator.session_token,
        json!({"title":"Agent source review project", "visibility":"public"}),
    )
    .await;
    assert_eq!(project.status(), StatusCode::CREATED);
    let project: Value = response_json(project).await;
    let project_id = project["id"].as_str().expect("project ID");
    let invitation = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/research/projects/{project_id}/invitations"),
        &operator.session_token,
        json!({"username":reviewer_username,"role":"reviewer"}),
    )
    .await;
    assert_eq!(invitation.status(), StatusCode::ACCEPTED);
    assert_eq!(
        send_json_request(
            &app,
            "POST",
            &format!("/api/v1/research/projects/{project_id}/invitations/accept"),
            &reviewer.session_token,
            json!({}),
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    let task = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/research/projects/{project_id}/tasks"),
        &operator.session_token,
        json!({
            "title":"Compare evidence on public learning access",
            "objective":"Summarize the evidence in the task scope",
            "scope":"Use only public and approved research sources",
            "allowedDataSources":["Public institutional reports"],
            "permittedTools":["Literature review"],
            "expectedDeliverables":["Evidence summary"]
        }),
    )
    .await;
    assert_eq!(task.status(), StatusCode::CREATED);
    let task: Value = response_json(task).await;
    let task_id = task["id"].as_str().expect("task ID");
    for action in ["queue", "assign-agent"] {
        let body = if action == "assign-agent" {
            json!({"agentId":agent_id})
        } else {
            json!({})
        };
        assert_eq!(
            send_json_request(
                &app,
                "POST",
                &format!("/api/v1/research/tasks/{task_id}/{action}"),
                &operator.session_token,
                body,
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );
    }
    let assigned = send_agent_request(
        &app,
        "GET",
        "/api/v1/agent-api/research/tasks",
        secret,
        None,
    )
    .await;
    assert_eq!(assigned.status(), StatusCode::OK);
    let assigned: Value = response_json(assigned).await;
    assert_eq!(assigned[0]["id"], task_id);
    let details = send_agent_request(
        &app,
        "GET",
        &format!("/api/v1/agent-api/research/tasks/{task_id}"),
        secret,
        None,
    )
    .await;
    assert_eq!(details.status(), StatusCode::OK);
    let details: Value = response_json(details).await;
    assert!(details.get("projectId").is_none());
    assert!(details.get("createdByUsername").is_none());
    assert_eq!(
        send_agent_request(
            &app,
            "GET",
            &format!("/api/v1/agent-api/research/tasks/{}", Uuid::new_v4()),
            secret,
            None,
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );

    for action in ["accept", "start"] {
        assert_eq!(
            send_agent_request(
                &app,
                "POST",
                &format!("/api/v1/agent-api/research/tasks/{task_id}/{action}"),
                secret,
                None,
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );
    }
    let rejected_http_source = send_agent_request(
        &app,
        "POST",
        &format!("/api/v1/agent-api/research/tasks/{task_id}/submissions"),
        secret,
        Some(json!({
            "result":"A valid result body.",
            "sources":[{"title":"Insecure source", "url":"http://research.example.org/source"}]
        })),
    )
    .await;
    assert_eq!(rejected_http_source.status(), StatusCode::BAD_REQUEST);
    let submission = send_agent_request(
        &app,
        "POST",
        &format!("/api/v1/agent-api/research/tasks/{task_id}/submissions"),
        secret,
        Some(json!({
            "result":"The submitted analysis references the source below for human evaluation.",
            "sources":[{
                "title":"Synthetic integration fixture source",
                "url":"https://research.example.org/test-fixture",
                "doi":"10.5555/peerfora.test"
            }]
        })),
    )
    .await;
    assert_eq!(submission.status(), StatusCode::CREATED);
    research_submission_quota::verify_agent_submission_quota_is_enforced(
        &app,
        project_id,
        agent_id,
        secret,
        &operator.session_token,
    )
    .await;
    let human_review = send_json_request(
        &app,
        "GET",
        &format!("/api/v1/research/tasks/{task_id}"),
        &reviewer.session_token,
        json!({}),
    )
    .await;
    assert_eq!(human_review.status(), StatusCode::OK);
    let human_review: Value = response_json(human_review).await;
    assert_eq!(
        human_review["latestSubmissionAgentHandle"],
        "research_task_agent"
    );
    assert_eq!(
        human_review["latestSubmissionSources"][0]["title"],
        "Synthetic integration fixture source"
    );
    assert_eq!(human_review["status"], "awaiting_review");
    let review = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/review"),
        &reviewer.session_token,
        json!({"decision":"approved","notes":"Reviewed the synthetic source fixture."}),
    )
    .await;
    assert_eq!(review.status(), StatusCode::NO_CONTENT);
    research_finding_flow::verify_approved_agent_finding_review_and_correction(
        &app,
        task_id,
        &operator.session_token,
        &reviewer.session_token,
        &operator_username,
    )
    .await;
}

async fn send_agent_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    credential: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {credential}"));
    if body.is_some() {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    app.clone()
        .oneshot(
            request
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .expect("agent research request should build"),
        )
        .await
        .expect("agent research API should respond")
}

async fn create_agent_reviewer_session(database: &PgPool) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let session_token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
         VALUES ($1, $2, 'integration-test-hash', 'test-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("agent reviewer should persist");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
         VALUES ($1, $2, 'agent_reviewer', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(
        admin_auth::SEEDED_ROOT_ADMINISTRATOR_ID
            .parse::<Uuid>()
            .expect("seed root administrator ID"),
    )
    .execute(database)
    .await
    .expect("agent reviewer role should persist");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
         VALUES ($1, $2, $3, now() + interval '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(one_time_token::hash_one_time_token(&session_token))
    .execute(database)
    .await
    .expect("agent reviewer session should persist");
    (administrator_id, session_token)
}

async fn send_admin_request(
    app: &axum::Router,
    path: &str,
    session_token: &str,
    body: Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header(
            header::COOKIE,
            format!("{}={session_token}", admin_auth::ADMIN_SESSION_COOKIE_NAME),
        )
        .header(header::ORIGIN, support::TEST_ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("administrator research review request should build");
    app.clone()
        .oneshot(request)
        .await
        .expect("administrator research review API should respond")
}
