use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::{response_json, send_json_request};

pub async fn verify_approved_agent_finding_review_and_correction(
    app: &axum::Router,
    task_id: &str,
    operator_token: &str,
    reviewer_token: &str,
    operator_username: &str,
) {
    let edited_result = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/promote-finding"),
        operator_token,
        json!({
            "submissionNumber":1,
            "title":"Evidence summary for public learning access",
            "statement":"The human must not rewrite the agent-authored result during promotion."
        }),
    )
    .await;
    assert_eq!(edited_result.status(), StatusCode::BAD_REQUEST);

    let promoted = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/promote-finding"),
        operator_token,
        json!({
            "submissionNumber":1,
            "title":"Evidence summary for public learning access",
            "statement":"The submitted analysis references the source below for human evaluation."
        }),
    )
    .await;
    assert_eq!(promoted.status(), StatusCode::CREATED);
    let promoted: Value = response_json(promoted).await;
    let finding_id = promoted["id"].as_str().expect("finding ID");
    assert_eq!(promoted["authoredByAgentHandle"], "research_task_agent");
    assert_eq!(promoted["status"], "in_review");
    assert_eq!(
        promoted["citations"][0]["title"],
        "Synthetic integration fixture source"
    );

    let promoter_cannot_review = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/review"),
        operator_token,
        json!({"decision":"approve","notes":"The promoter must not review this agent-authored finding."}),
    )
    .await;
    assert_eq!(promoter_cannot_review.status(), StatusCode::FORBIDDEN);
    let finding_review = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/review"),
        reviewer_token,
        json!({"decision":"approve","notes":"Reviewed the agent-authored finding and its cited source."}),
    )
    .await;
    assert_eq!(finding_review.status(), StatusCode::NO_CONTENT);
    let publish = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/publish"),
        operator_token,
        json!({}),
    )
    .await;
    assert_eq!(publish.status(), StatusCode::NO_CONTENT);
    let correction = send_json_request(
        app,
        "PUT",
        &format!("/api/v1/research/findings/{finding_id}"),
        operator_token,
        json!({
            "title":"Corrected evidence summary for public learning access",
            "statement":"The reviewed comparison includes additional limitations described in the sources.",
            "details":"A human correction preserves the original published revision until review."
        }),
    )
    .await;
    assert_eq!(correction.status(), StatusCode::OK);
    let correction = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/submit"),
        operator_token,
        json!({}),
    )
    .await;
    assert_eq!(correction.status(), StatusCode::NO_CONTENT);
    let correction_review = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/review"),
        reviewer_token,
        json!({"decision":"approve","notes":"The correction was reviewed."}),
    )
    .await;
    assert_eq!(correction_review.status(), StatusCode::NO_CONTENT);
    let correction_publish = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/findings/{finding_id}/publish"),
        operator_token,
        json!({}),
    )
    .await;
    assert_eq!(correction_publish.status(), StatusCode::NO_CONTENT);
    let revisions = send_json_request(
        app,
        "GET",
        &format!("/api/v1/research/findings/{finding_id}/revisions"),
        reviewer_token,
        json!({}),
    )
    .await;
    assert_eq!(revisions.status(), StatusCode::OK);
    let revisions: Value = response_json(revisions).await;
    assert_eq!(revisions.as_array().map(Vec::len), Some(2));
    assert_eq!(revisions[0]["authoredByUsername"], operator_username);
    assert_eq!(revisions[1]["authoredByAgentHandle"], "research_task_agent");

    let public_profile = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/agents/research_task_agent")
                .body(axum::body::Body::empty())
                .expect("agent profile request should build"),
        )
        .await
        .expect("public agent profile should respond");
    assert_eq!(public_profile.status(), StatusCode::OK);
    let public_profile: Value = response_json(public_profile).await;
    assert_eq!(public_profile["operatorUsername"], operator_username);
    assert_eq!(public_profile["modelName"], "Peerfora Research Fixture");
    assert_eq!(public_profile["supportedLanguages"], json!(["en", "en-GB"]));
    assert_eq!(public_profile["publishedContributionCount"], 1);
    assert_eq!(
        public_profile["recentContributions"][0]["findingId"],
        finding_id
    );
    assert_eq!(
        public_profile["recentContributions"][0]["title"],
        "Corrected evidence summary for public learning access"
    );
}
