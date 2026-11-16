use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{
    create_account_fixture, create_administrator_session, send_admin_json_request,
    send_json_request, support,
};
use peerfora_api::{app::build_app, database::connect_database};

#[tokio::test]
async fn approved_agent_profiles_disclose_identity_and_only_approved_capabilities() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply migrations");
    let operator = create_account_fixture(&database, "agent_public_profile").await;
    let app = build_app(support::test_app_state(database.clone(), database_url));
    let registration = send_json_request(
        &app,
        "POST",
        "/api/v1/agents",
        &operator.session_token,
        json!({
            "handle":"profile_research_agent",
            "displayName":"Profile Research Agent",
            "description":"A test agent profile with disclosed model metadata.",
            "modelDisclosure":"Externally operated test model",
            "modelName":"Peerfora Test Model",
            "modelVersion":"0.1.0",
            "supportedLanguages":["en-GB", "fr"],
            "requestedCapabilities":["research"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);

    let hidden_until_approved = request_public_profile(&app, "profile_research_agent").await;
    assert_eq!(hidden_until_approved.status(), StatusCode::NOT_FOUND);

    let agent_id: String =
        sqlx::query_scalar("SELECT actor_id::text FROM agent_actor_identities WHERE handle = $1")
            .bind("profile_research_agent")
            .fetch_one(&database)
            .await
            .expect("registered agent actor should exist");
    let (_, admin_token) = create_administrator_session(&database, "agent_reviewer").await;
    let review = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/agents/{agent_id}/review"),
        &admin_token,
        json!({
            "decision":"approve",
            "approvedCapabilities":["research"],
            "reason":"The research capability is within the supported agent workflow."
        }),
    )
    .await;
    assert_eq!(review.status(), StatusCode::NO_CONTENT);

    let response = request_public_profile(&app, "profile_research_agent").await;
    assert_eq!(response.status(), StatusCode::OK);
    let profile: Value = support::response_json(response).await;
    assert_eq!(profile["modelName"], "Peerfora Test Model");
    assert_eq!(profile["modelVersion"], "0.1.0");
    assert_eq!(profile["supportedLanguages"], json!(["en-GB", "fr"]));
    assert_eq!(profile["approvedCapabilities"], json!(["research"]));
    assert_eq!(profile["operationalStatus"], "approved");
    assert_eq!(
        profile["operatorVerificationStatus"],
        "email_claimed_by_active_verified_account"
    );
    assert!(profile["operatorUsername"].as_str().is_some());
    assert!(profile.get("operatorEmail").is_none());
    assert!(profile.get("operatorAccountId").is_none());
    assert_eq!(profile["publishedContributionCount"], 0);
    assert_eq!(profile["recentContributions"], json!([]));
}

async fn request_public_profile(app: &axum::Router, handle: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/agents/{handle}"))
                .body(Body::empty())
                .expect("public agent profile request should build"),
        )
        .await
        .expect("router should return a public profile response")
}
