#[path = "research_services_postgres/mod.rs"]
mod research_services_postgres;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use peerfora_api::{app::build_app, database::connect_database};
use research_services_postgres::{
    create_account_fixture, database_url_from_environment, test_app_state,
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn private_research_projects_enforce_owner_and_member_access() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "owner").await;
    let collaborator = create_account_fixture(&database, "collab").await;
    let subject_id: Uuid =
        sqlx::query_scalar("SELECT id FROM academic_subjects WHERE is_active = TRUE LIMIT 1")
            .fetch_one(&database)
            .await
            .expect("starter subject should be available");
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));

    let project_response = send_json_request(
        app.clone(),
        "POST",
        "/api/v1/research/projects",
        Some(&owner.session_token),
        json!({
            "title": "Global water access study",
            "abstract": "Initial abstract",
            "description": "A private research workspace.",
            "objectives": ["Compare regional access"],
            "researchQuestions": ["What improves access?"],
            "subjectIds": [subject_id]
        }),
    )
    .await;
    assert_eq!(project_response.status(), StatusCode::CREATED);
    let project_json: serde_json::Value = response_json(project_response).await;
    assert_eq!(project_json["visibility"], "private");
    assert_eq!(project_json["currentMemberRole"], "owner");
    assert_eq!(project_json["ownerUsername"], owner.username);
    let project_id = project_json["id"]
        .as_str()
        .expect("project response should have an id")
        .parse::<Uuid>()
        .expect("project id should be a UUID");

    assert_project_not_visible(&app, project_id, None).await;
    assert_project_not_visible(&app, project_id, Some(&collaborator.session_token)).await;
    assert_project_is_absent_from_nonmember_list(&app, project_id, &collaborator.session_token)
        .await;
    invite_collaborator(
        &app,
        project_id,
        &owner.session_token,
        &collaborator.username,
    )
    .await;
    accept_invitation(&app, project_id, &collaborator.session_token).await;
    assert_collaborator_can_read(&app, project_id, &collaborator.session_token).await;
    assert_collaborator_cannot_edit(&app, project_id, &collaborator.session_token).await;
    assert_collaborator_cannot_invite(&app, project_id, &collaborator.session_token).await;
    update_and_clear_project_abstract(&app, project_id, &owner.session_token).await;
    assert_owner_cannot_leave_or_remove_self(
        &app,
        project_id,
        &owner.session_token,
        &owner.username,
    )
    .await;
    remove_collaborator(
        &app,
        project_id,
        &owner.session_token,
        &collaborator.username,
    )
    .await;
    assert_project_not_visible(&app, project_id, Some(&collaborator.session_token)).await;
    archive_project(&app, project_id, &owner.session_token).await;
    assert_project_not_visible(&app, project_id, Some(&owner.session_token)).await;
}

async fn assert_project_not_visible(
    app: &axum::Router,
    project_id: Uuid,
    session_token: Option<&str>,
) {
    let response = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/research/projects/{project_id}"),
        session_token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn assert_project_is_absent_from_nonmember_list(
    app: &axum::Router,
    project_id: Uuid,
    session_token: &str,
) {
    let response = send_request(
        app.clone(),
        "GET",
        "/api/v1/research/projects",
        Some(session_token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let projects: serde_json::Value = response_json(response).await;
    assert!(projects.as_array().is_some_and(|projects| {
        !projects
            .iter()
            .any(|project| project["id"] == project_id.to_string())
    }));
}

async fn invite_collaborator(
    app: &axum::Router,
    project_id: Uuid,
    owner_token: &str,
    invitee_username: &str,
) {
    let response = send_json_request(
        app.clone(),
        "POST",
        &format!("/api/v1/research/projects/{project_id}/invitations"),
        Some(owner_token),
        json!({"username": invitee_username, "role": "researcher"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

async fn accept_invitation(app: &axum::Router, project_id: Uuid, collaborator_token: &str) {
    let invitations = send_request(
        app.clone(),
        "GET",
        "/api/v1/research/invitations",
        Some(collaborator_token),
    )
    .await;
    assert_eq!(invitations.status(), StatusCode::OK);
    let invitations_json: serde_json::Value = response_json(invitations).await;
    assert_eq!(invitations_json.as_array().map(Vec::len), Some(1));

    let accepted = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/research/projects/{project_id}/invitations/accept"),
        Some(collaborator_token),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::NO_CONTENT);
}

async fn assert_collaborator_can_read(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/research/projects/{project_id}"),
        Some(token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let project_json: serde_json::Value = response_json(response).await;
    assert_eq!(project_json["currentMemberRole"], "researcher");
}

async fn assert_collaborator_cannot_edit(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_json_request(
        app.clone(),
        "PATCH",
        &format!("/api/v1/research/projects/{project_id}"),
        Some(token),
        json!({"title": "Collaborator edit"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn assert_collaborator_cannot_invite(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_json_request(
        app.clone(),
        "POST",
        &format!("/api/v1/research/projects/{project_id}/invitations"),
        Some(token),
        json!({"username": "some_other_user", "role": "observer"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn assert_owner_cannot_leave_or_remove_self(
    app: &axum::Router,
    project_id: Uuid,
    owner_token: &str,
    owner_username: &str,
) {
    let leave_response = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/research/projects/{project_id}/membership"),
        Some(owner_token),
    )
    .await;
    assert_eq!(leave_response.status(), StatusCode::CONFLICT);

    let removal_response = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/research/projects/{project_id}/members/{owner_username}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(removal_response.status(), StatusCode::CONFLICT);
}

async fn update_and_clear_project_abstract(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_json_request(
        app.clone(),
        "PATCH",
        &format!("/api/v1/research/projects/{project_id}"),
        Some(token),
        json!({"abstract": null, "description": "Updated project description"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let project_json: serde_json::Value = response_json(response).await;
    assert!(project_json["abstract"].is_null());
    assert_eq!(project_json["description"], "Updated project description");
}

async fn remove_collaborator(
    app: &axum::Router,
    project_id: Uuid,
    owner_token: &str,
    member_username: &str,
) {
    let removed = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/research/projects/{project_id}/members/{member_username}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::NO_CONTENT);
}

async fn archive_project(app: &axum::Router, project_id: Uuid, owner_token: &str) {
    let response = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/research/projects/{project_id}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

async fn send_json_request(
    app: axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.oneshot(
        request
            .body(Body::from(body.to_string()))
            .expect("HTTP request should be valid"),
    )
    .await
    .expect("research route should respond")
}

async fn send_request(
    app: axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.oneshot(
        request
            .body(Body::empty())
            .expect("HTTP request should be valid"),
    )
    .await
    .expect("research route should respond")
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain JSON")
}
