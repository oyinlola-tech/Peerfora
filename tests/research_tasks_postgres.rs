#[path = "research_services_postgres/mod.rs"]
mod research_services_postgres;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    response::Response,
};
use peerfora_api::{app::build_app, database::connect_database};
use research_services_postgres::{
    create_account_fixture, database_url_from_environment, test_app_state,
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const RESEARCH_PROJECTS_PATH: &str = "/api/v1/research/projects";

#[tokio::test]
async fn research_task_lifecycle_enforces_membership_assignment_and_review() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .try_init();
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "task_owner").await;
    let assignee = create_account_fixture(&database, "task_assignee").await;
    let reviewer = create_account_fixture(&database, "task_reviewer").await;
    let outsider = create_account_fixture(&database, "task_outsider").await;
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));
    let project_id = create_private_project(&app, &owner.session_token).await;
    invite_member(
        &app,
        project_id,
        &owner.session_token,
        &assignee.username,
        "researcher",
    )
    .await;
    invite_member(
        &app,
        project_id,
        &owner.session_token,
        &reviewer.username,
        "reviewer",
    )
    .await;
    accept_project_invitation(&app, project_id, &assignee.session_token).await;
    accept_project_invitation(&app, project_id, &reviewer.session_token).await;

    let invalid_task = send_json_request(
        &app,
        "POST",
        &project_tasks_path(project_id),
        &owner.session_token,
        json!({
            "title": "Water access comparison",
            "objective": "Compare interventions",
            "scope": "Use the permitted public reports",
            "expectedDeliverables": []
        }),
    )
    .await;
    assert_eq!(invalid_task.status(), StatusCode::BAD_REQUEST);

    let task_id = create_task(&app, project_id, &owner.session_token).await;
    assert_task_hidden_from_nonmember(&app, task_id, &outsider.session_token).await;
    assert_nonmember_cannot_create_task(&app, project_id, &outsider.session_token).await;
    queue_task(&app, task_id, &owner.session_token).await;
    assign_task(&app, task_id, &owner.session_token, &assignee.username).await;
    assert_task_cannot_start_before_acceptance(&app, task_id, &assignee.session_token).await;
    accept_and_start_task(&app, task_id, &assignee.session_token).await;

    let result = submit_result(&app, task_id, &assignee.session_token, "comparison result").await;
    assert_eq!(result["status"], "awaiting_review");
    assert_eq!(result["latestSubmissionNumber"], 1);
    let self_review = review_task(&app, task_id, &assignee.session_token, "approved").await;
    assert_eq!(self_review.status(), StatusCode::FORBIDDEN);

    review_task(&app, task_id, &reviewer.session_token, "revision_requested").await;
    let reopened = get_task(&app, task_id, &owner.session_token).await;
    assert_eq!(reopened["status"], "assigned");
    assert_eq!(reopened["latestReviewStatus"], "revision_requested");
    assert_task_cannot_start_before_acceptance(&app, task_id, &assignee.session_token).await;
    accept_and_start_task(&app, task_id, &assignee.session_token).await;
    let result = submit_result(&app, task_id, &assignee.session_token, "revised result").await;
    assert_eq!(result["latestSubmissionNumber"], 2);
    let reviewed = review_task(&app, task_id, &reviewer.session_token, "approved").await;
    assert_eq!(reviewed.status(), StatusCode::NO_CONTENT);
    let completed = get_task(&app, task_id, &owner.session_token).await;
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["latestReviewStatus"], "approved");
    assert_eq!(
        cancel_task(&app, task_id, &owner.session_token)
            .await
            .status(),
        StatusCode::CONFLICT
    );

    let task_to_cancel = create_task(&app, project_id, &owner.session_token).await;
    queue_task(&app, task_to_cancel, &owner.session_token).await;
    assign_task(
        &app,
        task_to_cancel,
        &owner.session_token,
        &assignee.username,
    )
    .await;
    remove_member(&app, project_id, &owner.session_token, &assignee.username).await;
    let cancelled = get_task(&app, task_to_cancel, &owner.session_token).await;
    assert_eq!(cancelled["status"], "cancelled");
    assert_task_hidden_from_nonmember(&app, task_to_cancel, &assignee.session_token).await;
}

async fn create_private_project(app: &axum::Router, owner_token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        RESEARCH_PROJECTS_PATH,
        owner_token,
        json!({"title": "Task lifecycle project"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("project response should include its ID")
        .parse()
        .expect("project ID should be a UUID")
}

async fn invite_member(
    app: &axum::Router,
    project_id: Uuid,
    owner_token: &str,
    username: &str,
    role: &str,
) {
    let response = send_json_request(
        app,
        "POST",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}/invitations"),
        owner_token,
        json!({"username": username, "role": role}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

async fn accept_project_invitation(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_request(
        app,
        "POST",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}/invitations/accept"),
        token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

async fn create_task(app: &axum::Router, project_id: Uuid, owner_token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        &project_tasks_path(project_id),
        owner_token,
        new_task_body("Compare water access models"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let task = response_json(response).await;
    assert_eq!(task["status"], "draft");
    task["id"]
        .as_str()
        .expect("task response should include its ID")
        .parse()
        .expect("task ID should be a UUID")
}

fn new_task_body(title: &str) -> Value {
    json!({
        "title": title,
        "objective": "Compare community-led approaches",
        "scope": "Use only the public reports named by the research team",
        "allowedDataSources": ["Regional public water reports"],
        "permittedTools": ["Manual literature review"],
        "expectedDeliverables": ["Comparison table", "Source notes"],
        "maxRuntimeSeconds": 900,
        "maxResultBytes": 200000
    })
}

async fn assert_task_hidden_from_nonmember(app: &axum::Router, task_id: Uuid, token: &str) {
    let response = send_request(
        app,
        "GET",
        &format!("/api/v1/research/tasks/{task_id}"),
        token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn assert_nonmember_cannot_create_task(app: &axum::Router, project_id: Uuid, token: &str) {
    let response = send_json_request(
        app,
        "POST",
        &project_tasks_path(project_id),
        token,
        new_task_body("Unauthorized task"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn assert_task_cannot_start_before_acceptance(
    app: &axum::Router,
    task_id: Uuid,
    assignee_token: &str,
) {
    let start_before_acceptance = send_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/start"),
        assignee_token,
    )
    .await;
    assert_eq!(start_before_acceptance.status(), StatusCode::CONFLICT);
}

async fn queue_task(app: &axum::Router, task_id: Uuid, token: &str) {
    let response = send_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/queue"),
        token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

async fn assign_task(app: &axum::Router, task_id: Uuid, token: &str, username: &str) {
    let response = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/assign"),
        token,
        json!({"assigneeUsername": username}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

async fn accept_and_start_task(app: &axum::Router, task_id: Uuid, token: &str) {
    for action in ["accept", "start"] {
        let response = send_request(
            app,
            "POST",
            &format!("/api/v1/research/tasks/{task_id}/{action}"),
            token,
        )
        .await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

async fn submit_result(app: &axum::Router, task_id: Uuid, token: &str, result: &str) -> Value {
    let response = send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/submissions"),
        token,
        json!({"result": result}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

async fn review_task(app: &axum::Router, task_id: Uuid, token: &str, decision: &str) -> Response {
    send_json_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/review"),
        token,
        json!({"decision": decision, "notes": "Review recorded"}),
    )
    .await
}

async fn cancel_task(app: &axum::Router, task_id: Uuid, token: &str) -> Response {
    send_request(
        app,
        "POST",
        &format!("/api/v1/research/tasks/{task_id}/cancel"),
        token,
    )
    .await
}

async fn get_task(app: &axum::Router, task_id: Uuid, token: &str) -> Value {
    let response = send_request(
        app,
        "GET",
        &format!("/api/v1/research/tasks/{task_id}"),
        token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

async fn remove_member(app: &axum::Router, project_id: Uuid, token: &str, username: &str) {
    let response = send_request(
        app,
        "DELETE",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}/members/{username}"),
        token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

fn project_tasks_path(project_id: Uuid) -> String {
    format!("{RESEARCH_PROJECTS_PATH}/{project_id}/tasks")
}

async fn send_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    session_token: &str,
) -> Response {
    send_optional_json_request(app, method, path, session_token, None).await
}

async fn send_json_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    session_token: &str,
    body: Value,
) -> Response {
    send_optional_json_request(app, method, path, session_token, Some(body)).await
}

async fn send_optional_json_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    session_token: &str,
    body: Option<Value>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {session_token}"));
    let body = match body {
        Some(body) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(request.body(body).expect("request should be valid"))
        .await
        .expect("router should return a response")
}

async fn response_json(response: Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain valid JSON")
}
