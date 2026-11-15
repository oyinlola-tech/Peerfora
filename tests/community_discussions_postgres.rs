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
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn community_posts_follow_membership_and_visibility_rules() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "post_owner").await;
    let member = create_account_fixture(&database, "post_member").await;
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));

    let community =
        create_community(&app, &owner.session_token, "public-post-space", "public").await;
    let public_post = create_post(
        &app,
        &owner.session_token,
        "public-post-space",
        "question",
        "How does memory allocation work?",
        "I am comparing stack and heap allocation.",
    )
    .await;
    assert_eq!(public_post["authorUsername"], owner.username);
    let post_id = public_post["id"]
        .as_str()
        .expect("post id should be present");
    let public_feed = get_posts(&app, "public-post-space", None, StatusCode::OK).await;
    assert_eq!(public_feed.as_array().map(Vec::len), Some(1));
    assert_eq!(public_feed[0]["id"], public_post["id"]);

    let nonmember_post = send_json_request(
        &app,
        "POST",
        &format!(
            "/api/v1/communities/{}/posts",
            community["slug"].as_str().unwrap()
        ),
        Some(&member.session_token),
        json!({"kind":"discussion","title":"Not joined yet","body":"Should be denied."}),
    )
    .await;
    assert_eq!(nonmember_post.status(), StatusCode::FORBIDDEN);
    let nonmember_comment =
        create_comment(&app, post_id, &member.session_token, "Not joined yet", None).await;
    assert_eq!(nonmember_comment.status(), StatusCode::FORBIDDEN);

    let joined = send_request(
        &app,
        "PUT",
        &format!(
            "/api/v1/communities/{}/membership",
            community["slug"].as_str().unwrap()
        ),
        Some(&member.session_token),
    )
    .await;
    assert_eq!(joined.status(), StatusCode::OK);
    let root_comment = create_comment(
        &app,
        post_id,
        &member.session_token,
        "The stack is typically automatic storage.",
        None,
    )
    .await;
    assert_eq!(root_comment.status(), StatusCode::CREATED);
    let root_comment_json = response_json(root_comment).await;
    assert_eq!(root_comment_json["parentCommentId"], Value::Null);
    let reply = create_comment(
        &app,
        post_id,
        &owner.session_token,
        "That is a useful distinction.",
        Some(
            root_comment_json["id"]
                .as_str()
                .expect("comment id should be present"),
        ),
    )
    .await;
    assert_eq!(reply.status(), StatusCode::CREATED);
    let reply_json = response_json(reply).await;
    assert_eq!(reply_json["parentCommentId"], root_comment_json["id"]);
    let comments = list_comments(&app, post_id, None, StatusCode::OK).await;
    assert_eq!(comments.as_array().map(Vec::len), Some(2));
    assert_eq!(comments[1]["parentCommentId"], comments[0]["id"]);
    let other_post = create_post(
        &app,
        &owner.session_token,
        "public-post-space",
        "text",
        "Another thread",
        "A parent comment must not cross post boundaries.",
    )
    .await;
    let cross_post_reply = create_comment(
        &app,
        other_post["id"]
            .as_str()
            .expect("post id should be present"),
        &owner.session_token,
        "This reply references a different thread.",
        Some(
            root_comment_json["id"]
                .as_str()
                .expect("comment id should be present"),
        ),
    )
    .await;
    assert_eq!(cross_post_reply.status(), StatusCode::NOT_FOUND);
    let member_post = create_post(
        &app,
        &member.session_token,
        "public-post-space",
        "discussion",
        "A joined member can post",
        "This contribution is visible to the public.",
    )
    .await;
    assert_eq!(member_post["authorUsername"], member.username);

    create_community(&app, &owner.session_token, "private-post-space", "private").await;
    let private_post = create_post(
        &app,
        &owner.session_token,
        "private-post-space",
        "text",
        "Private research group notes",
        "This body must not appear to a nonmember.",
    )
    .await;
    let private_post_id = private_post["id"]
        .as_str()
        .expect("post id should be present");
    let private_comments = list_comments(
        &app,
        private_post_id,
        Some(&member.session_token),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(private_comments["message"], "Post was not found");
    let private_feed = get_posts(
        &app,
        "private-post-space",
        Some(&member.session_token),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(private_feed["message"], "Community was not found");

    let invalid_post = send_json_request(
        &app,
        "POST",
        "/api/v1/communities/public-post-space/posts",
        Some(&owner.session_token),
        json!({"kind":"text","title":" ","body":"Missing title"}),
    )
    .await;
    assert_eq!(invalid_post.status(), StatusCode::BAD_REQUEST);
}

async fn create_community(app: &axum::Router, token: &str, slug: &str, visibility: &str) -> Value {
    let response = send_json_request(
        app,
        "POST",
        "/api/v1/communities",
        Some(token),
        json!({
            "slug": slug,
            "displayName": "Discussion Space",
            "category": "general",
            "visibility": visibility
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await
}

async fn create_post(
    app: &axum::Router,
    token: &str,
    slug: &str,
    kind: &str,
    title: &str,
    body: &str,
) -> Value {
    let response = send_json_request(
        app,
        "POST",
        &format!("/api/v1/communities/{slug}/posts"),
        Some(token),
        json!({"kind":kind,"title":title,"body":body}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await
}

async fn get_posts(
    app: &axum::Router,
    slug: &str,
    token: Option<&str>,
    expected_status: StatusCode,
) -> Value {
    let response = send_request(
        app,
        "GET",
        &format!("/api/v1/communities/{slug}/posts"),
        token,
    )
    .await;
    assert_eq!(response.status(), expected_status);
    response_json(response).await
}

async fn create_comment(
    app: &axum::Router,
    post_id: &str,
    token: &str,
    body: &str,
    parent_comment_id: Option<&str>,
) -> axum::response::Response {
    let request_body = json!({
        "body": body,
        "parentCommentId": parent_comment_id
    });
    send_json_request(
        app,
        "POST",
        &format!("/api/v1/posts/{post_id}/comments"),
        Some(token),
        request_body,
    )
    .await
}

async fn list_comments(
    app: &axum::Router,
    post_id: &str,
    token: Option<&str>,
    expected_status: StatusCode,
) -> Value {
    let response = send_request(
        app,
        "GET",
        &format!("/api/v1/posts/{post_id}/comments"),
        token,
    )
    .await;
    assert_eq!(response.status(), expected_status);
    response_json(response).await
}

async fn send_json_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.clone()
        .oneshot(
            request
                .body(Body::from(body.to_string()))
                .expect("request should be valid"),
        )
        .await
        .expect("community discussion route should respond")
}

async fn send_request(
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
        .expect("community route should respond")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain JSON")
}
