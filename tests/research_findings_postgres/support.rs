use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

pub const RESEARCH_PROJECTS_PATH: &str = "/api/v1/research/projects";

pub async fn create_public_project(app: &axum::Router, token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        RESEARCH_PROJECTS_PATH,
        token,
        json!({"title": "Published finding project", "visibility": "public"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("project should include an ID")
        .parse()
        .expect("project ID should be a UUID")
}

pub async fn create_public_community(app: &axum::Router, token: &str) -> String {
    let response = send_json_request(
        app,
        "POST",
        "/api/v1/communities",
        token,
        json!({
            "slug": format!("research-{}", Uuid::new_v4().simple()),
            "displayName": "Research discussion",
            "category": "general",
            "visibility": "public"
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["slug"]
        .as_str()
        .expect("community response should include a slug")
        .to_owned()
}

pub async fn create_community_post(app: &axum::Router, community_slug: &str, token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        &format!("/api/v1/communities/{community_slug}/posts"),
        token,
        json!({
            "kind": "discussion",
            "title": "Discussion of published research",
            "body": "This discussion references an approved public finding."
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("post response should include an ID")
        .parse()
        .expect("post ID should be a UUID")
}

pub async fn invite_reviewer(app: &axum::Router, project_id: Uuid, owner: &str, username: &str) {
    let response = send_json_request(
        app,
        "POST",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}/invitations"),
        owner,
        json!({"username": username, "role": "reviewer"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

pub async fn accept_project_invitation(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_request(
        app,
        "POST",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}/invitations/accept"),
        Some(token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

pub async fn create_doi_source(app: &axum::Router, project_id: Uuid, token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        &project_sources_path(project_id),
        token,
        doi_source_body(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("source should include an ID")
        .parse()
        .expect("source ID should be a UUID")
}

pub fn doi_source_body() -> Value {
    json!({
        "sourceType": "doi",
        "title": "Water access intervention study",
        "doi": "10.9999/peerfora-test-source",
        "authors": ["Example Researcher"],
        "publicationYear": 2025,
        "provenance": "Manually entered by a project member",
        "license": "CC BY 4.0"
    })
}

pub async fn create_finding(
    app: &axum::Router,
    project_id: Uuid,
    token: &str,
    source_id: Uuid,
) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        &project_findings_path(project_id),
        token,
        finding_body(source_id, "The tested intervention improved access."),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let finding = response_json(response).await;
    assert_eq!(finding["status"], "draft");
    assert_eq!(finding["citations"].as_array().map(Vec::len), Some(1));
    finding["id"]
        .as_str()
        .expect("finding should include an ID")
        .parse()
        .expect("finding ID should be a UUID")
}

pub fn finding_body(source_id: Uuid, statement: &str) -> Value {
    json!({
        "title": "Water access intervention",
        "statement": statement,
        "details": "Observed in the synthetic test scenario.",
        "citations": [{"sourceId": source_id, "locator": "Table 2", "context": "Outcome comparison"}]
    })
}

pub async fn submit_finding(app: &axum::Router, finding_id: Uuid, token: &str) -> StatusCode {
    send_request(
        app,
        "POST",
        &format!("{}/submit", finding_path(finding_id)),
        Some(token),
    )
    .await
    .status()
}

pub async fn review_finding(
    app: &axum::Router,
    finding_id: Uuid,
    token: &str,
) -> axum::response::Response {
    send_json_request(
        app,
        "POST",
        &format!("{}/review", finding_path(finding_id)),
        token,
        json!({"decision": "approve", "notes": "Source and finding reviewed."}),
    )
    .await
}

pub async fn publish_finding(
    app: &axum::Router,
    finding_id: Uuid,
    token: &str,
) -> axum::response::Response {
    send_request(
        app,
        "POST",
        &format!("{}/publish", finding_path(finding_id)),
        Some(token),
    )
    .await
}

pub async fn retract_finding(
    app: &axum::Router,
    finding_id: Uuid,
    token: &str,
) -> axum::response::Response {
    send_request(
        app,
        "POST",
        &format!("{}/retract", finding_path(finding_id)),
        Some(token),
    )
    .await
}

pub async fn find_finding(
    app: &axum::Router,
    finding_id: Uuid,
    token: Option<&str>,
) -> axum::response::Response {
    send_request(app, "GET", &finding_path(finding_id), token).await
}

pub async fn assert_finding_hidden(app: &axum::Router, finding_id: Uuid, token: Option<&str>) {
    assert_eq!(
        find_finding(app, finding_id, token).await.status(),
        StatusCode::NOT_FOUND
    );
}

pub async fn list_revisions(app: &axum::Router, finding_id: Uuid, token: &str) -> Vec<Value> {
    let response = send_request(
        app,
        "GET",
        &format!("{}/revisions", finding_path(finding_id)),
        Some(token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response)
        .await
        .as_array()
        .cloned()
        .unwrap_or_default()
}

pub fn project_sources_path(project_id: Uuid) -> String {
    format!("{RESEARCH_PROJECTS_PATH}/{project_id}/sources")
}

pub fn project_findings_path(project_id: Uuid) -> String {
    format!("{RESEARCH_PROJECTS_PATH}/{project_id}/findings")
}

pub fn finding_path(finding_id: Uuid) -> String {
    format!("/api/v1/research/findings/{finding_id}")
}

pub async fn send_json_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    value: Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .expect("request should be valid");
    app.clone()
        .oneshot(request)
        .await
        .expect("router should handle request")
}

pub async fn send_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.clone()
        .oneshot(
            request
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("router should handle request")
}

pub async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 2_000_000)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain JSON")
}
