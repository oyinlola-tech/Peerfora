#[path = "developer_applications_postgres/support.rs"]
mod support;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use peerfora_api::{
    app::build_app,
    database::connect_database,
    enums::account_auth_error::AccountAuthError,
    interfaces::email_sender::{EmailMessage, EmailSender},
    utils::one_time_token,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    create_account_fixture, response_json, send_json_request, test_app_state_with_email_sender,
};
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone, Default)]
struct RecordingEmailSender {
    messages: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl EmailSender for RecordingEmailSender {
    async fn send_email(
        &self,
        _recipient: &str,
        message: &EmailMessage,
    ) -> Result<(), AccountAuthError> {
        self.messages
            .lock()
            .expect("email message list should remain available")
            .push(message.plain_text.clone());
        Ok(())
    }
}

#[tokio::test]
async fn agent_registration_requires_verified_operator_claim_before_review() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply migrations");
    let operator_email = format!("agent-operator-{}@example.test", Uuid::new_v4());
    let operator = create_verified_account(&database, &operator_email).await;
    let unrelated_operator = create_account_fixture(&database, "unrelated_agent_operator").await;
    let email_sender = Arc::new(RecordingEmailSender::default());
    let app = build_app(test_app_state_with_email_sender(
        database.clone(),
        database_url.clone(),
        Some(email_sender.clone()),
    ));
    let app_without_email = build_app(support::test_app_state(database.clone(), database_url));
    let unavailable_delivery = send_public_json_request(
        &app_without_email,
        "/api/v1/agents/registrations",
        json!({
            "operatorEmail":format!("no-sender-{}@example.test", Uuid::new_v4()),
            "handle":"no_sender_agent",
            "displayName":"No Sender Agent",
            "requestedCapabilities":["research"]
        }),
    )
    .await;
    assert_eq!(
        unavailable_delivery.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );

    let registration = send_public_json_request(
        &app,
        "/api/v1/agents/registrations",
        json!({
            "operatorEmail":operator_email,
            "handle":"claimed_research_agent",
            "displayName":"Claimed Research Agent",
            "description":"A pending agent that needs an accountable operator.",
            "requestedCapabilities":["research", "cite_sources"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::ACCEPTED);
    let registration: Value = response_json(registration).await;
    let agent_id = registration["agentId"].as_str().expect("agent ID");
    assert_eq!(registration["reviewStatus"], "pending_claim");
    assert!(registration.get("token").is_none());
    assert!(registration.get("operatorEmail").is_none());

    let messages = email_sender
        .messages
        .lock()
        .expect("recorded email should be available")
        .clone();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("This one-time link expires in 60 minutes"));
    let claim_link = messages[0]
        .lines()
        .find(|line| line.contains("/claim-agent?token="))
        .expect("claim email should contain its configured claim link");
    let claim_token = claim_link
        .split_once("?token=")
        .map(|(_, token)| token)
        .expect("claim link should contain the token query parameter");

    let stored_claim: (Option<Uuid>, String, String) = sqlx::query_as(
        "SELECT agent.operator_account_id, agent.review_status, claim.operator_email_hash
         FROM agent_actor_identities agent
         JOIN agent_operator_claims claim ON claim.agent_actor_id = agent.actor_id
         WHERE agent.actor_id = $1",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("pending claim should persist");
    assert_eq!(stored_claim.0, None);
    assert_eq!(stored_claim.1, "pending_claim");
    assert_ne!(stored_claim.2, operator_email);
    let stored_token_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM agent_operator_claims WHERE agent_actor_id = $1",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("claim token hash should persist");
    assert_ne!(stored_token_hash, claim_token);

    let unrelated_claim = send_json_request(
        &app,
        "POST",
        "/api/v1/agents/claims/complete",
        &unrelated_operator.session_token,
        json!({"token":claim_token}),
    )
    .await;
    assert_eq!(unrelated_claim.status(), StatusCode::FORBIDDEN);

    let claimed = send_json_request(
        &app,
        "POST",
        "/api/v1/agents/claims/complete",
        &operator.session_token,
        json!({"token":claim_token}),
    )
    .await;
    assert_eq!(claimed.status(), StatusCode::OK);
    let claimed: Value = response_json(claimed).await;
    assert_eq!(claimed["agentId"], agent_id);
    assert_eq!(claimed["reviewStatus"], "pending_review");

    let operator_agents = send_json_request(
        &app,
        "GET",
        "/api/v1/agents/me",
        &operator.session_token,
        json!({}),
    )
    .await;
    assert_eq!(operator_agents.status(), StatusCode::OK);
    let operator_agents: Value = response_json(operator_agents).await;
    assert_eq!(operator_agents[0]["id"], agent_id);
    assert_eq!(operator_agents[0]["reviewStatus"], "pending_review");

    let reused_claim = send_json_request(
        &app,
        "POST",
        "/api/v1/agents/claims/complete",
        &operator.session_token,
        json!({"token":claim_token}),
    )
    .await;
    assert_eq!(reused_claim.status(), StatusCode::BAD_REQUEST);
    let operator_account: Uuid = sqlx::query_scalar(
        "SELECT operator_account_id FROM agent_actor_identities WHERE actor_id = $1",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .fetch_one(&database)
    .await
    .expect("claimed operator should persist");
    assert_eq!(operator_account, operator.account_id);
}

struct AccountWithEmail {
    account_id: Uuid,
    session_token: String,
}

async fn create_verified_account(database: &PgPool, email: &str) -> AccountWithEmail {
    let account_id = Uuid::new_v4();
    let country_code: String = sqlx::query_scalar(
        "SELECT country_code::text FROM countries WHERE is_active = TRUE LIMIT 1",
    )
    .fetch_one(database)
    .await
    .expect("country reference data should exist");
    let username = format!("operator_{}", &account_id.simple().to_string()[..16]);
    sqlx::query(
        "INSERT INTO accounts (
            id, email, username, password_hash, eligibility_country_code,
            age_policy_version, terms_accepted_at, email_verified_at
        ) VALUES ($1, $2, $3, 'test-hash', $4, 'test-policy-v1', now(), now())",
    )
    .bind(account_id)
    .bind(email)
    .bind(username)
    .bind(country_code)
    .execute(database)
    .await
    .expect("verified operator account should persist");
    let session_token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at)
         VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(account_id)
    .bind(one_time_token::hash_one_time_token(&session_token))
    .execute(database)
    .await
    .expect("operator session should persist");
    AccountWithEmail {
        account_id,
        session_token,
    }
}

async fn send_public_json_request(
    app: &axum::Router,
    path: &str,
    body: Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&body).expect("JSON should serialize"),
                ))
                .expect("public request should build"),
        )
        .await
        .expect("router should return a response")
}
