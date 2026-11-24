#[path = "phase_four_communication_postgres/support.rs"]
mod support;

use axum::http::StatusCode;
use peerfora_api::{app::build_app, database::connect_database};
use serde_json::json;
use support::{
    create_account_fixture, phase_four_test_app, response_json, send_json_request, test_app_state,
};
use uuid::Uuid;

#[tokio::test]
async fn phase_four_communication_is_member_scoped_and_durable() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "phase4_owner").await;
    let member = create_account_fixture(&database, "phase4_member").await;
    let outsider = create_account_fixture(&database, "phase4_outsider").await;
    let state = test_app_state(database.clone(), database_url.clone());
    let production_app = build_app(state.clone());
    let disabled_route = send_json_request(
        &production_app,
        "GET",
        "/api/v1/conversations",
        &owner.session_token,
        json!({}),
    )
    .await;
    assert_eq!(disabled_route.status(), StatusCode::NOT_FOUND);
    let app = phase_four_test_app(state);

    let preferences = send_json_request(
        &app,
        "GET",
        "/api/v1/notification-preferences",
        &owner.session_token,
        json!({}),
    )
    .await;
    assert_eq!(preferences.status(), StatusCode::OK);
    assert_eq!(response_json(preferences).await["inAppEnabled"], true);

    let conversation = send_json_request(
        &app,
        "POST",
        "/api/v1/conversations",
        &owner.session_token,
        json!({
            "conversationType": "group",
            "title": "Phase four test conversation",
            "memberAccountIds": [member.account_id.to_string()]
        }),
    )
    .await;
    assert_eq!(conversation.status(), StatusCode::CREATED);
    let conversation_id = response_json(conversation).await["id"]
        .as_str()
        .expect("conversation includes an id")
        .parse::<Uuid>()
        .expect("conversation id should parse");

    let outsider_list = send_json_request(
        &app,
        "GET",
        "/api/v1/conversations",
        &outsider.session_token,
        json!({}),
    )
    .await;
    assert_eq!(outsider_list.status(), StatusCode::OK);
    assert_eq!(response_json(outsider_list).await["totalCount"], 0);

    let message = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/conversations/{conversation_id}/messages"),
        &owner.session_token,
        json!({"body": "private phase four message"}),
    )
    .await;
    assert_eq!(message.status(), StatusCode::CREATED);
    let message_id = response_json(message).await["id"]
        .as_str()
        .expect("message includes an id")
        .parse::<Uuid>()
        .expect("message id should parse");

    let member_messages = send_json_request(
        &app,
        "GET",
        &format!("/api/v1/conversations/{conversation_id}/messages"),
        &member.session_token,
        json!({}),
    )
    .await;
    assert_eq!(member_messages.status(), StatusCode::OK);
    assert_eq!(
        response_json(member_messages)
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let unauthorized_message = send_json_request(
        &app,
        "GET",
        &format!("/api/v1/messages/{message_id}"),
        &outsider.session_token,
        json!({}),
    )
    .await;
    assert_eq!(unauthorized_message.status(), StatusCode::FORBIDDEN);

    let outsider_send = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/conversations/{conversation_id}/messages"),
        &outsider.session_token,
        json!({"body": "should be rejected"}),
    )
    .await;
    assert_eq!(outsider_send.status(), StatusCode::FORBIDDEN);

    let notifications = send_json_request(
        &app,
        "GET",
        "/api/v1/notifications",
        &member.session_token,
        json!({}),
    )
    .await;
    assert_eq!(notifications.status(), StatusCode::OK);
    let notification_page = response_json(notifications).await;
    assert!(notification_page["totalCount"].as_i64().unwrap() >= 1);
    let notification_body = notification_page["notifications"][0]["body"]
        .as_str()
        .expect("notification body should be present");
    assert!(!notification_body.contains("private phase four message"));

    let unread = send_json_request(
        &app,
        "GET",
        "/api/v1/notifications/unread-count",
        &member.session_token,
        json!({}),
    )
    .await;
    assert_eq!(unread.status(), StatusCode::OK);
    assert!(response_json(unread).await["unreadCount"].as_i64().unwrap() >= 1);

    let notification_id = notification_page["notifications"][0]["id"]
        .as_str()
        .expect("notification id should be a string")
        .to_string();
    let read_notification = send_json_request(
        &app,
        "PATCH",
        &format!("/api/v1/notifications/{notification_id}/read"),
        &member.session_token,
        json!({}),
    )
    .await;
    assert_eq!(read_notification.status(), StatusCode::NO_CONTENT);

    let device = send_json_request(
        &app,
        "POST",
        "/api/v1/devices/push",
        &member.session_token,
        json!({"platform": "apns", "token": "phase-four-test-token-value"}),
    )
    .await;
    assert_eq!(device.status(), StatusCode::CREATED);
    assert_eq!(response_json(device).await["platform"], "apns");

    let checksum = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let upload = send_json_request(
        &app,
        "POST",
        "/api/v1/media/uploads",
        &member.session_token,
        json!({
            "resourceType": "message",
            "resourceId": message_id.to_string(),
            "originalFilename": "note.txt",
            "mediaType": "text/plain",
            "sizeBytes": 5,
            "checksum": checksum
        }),
    )
    .await;
    assert_eq!(upload.status(), StatusCode::CREATED);
    let upload_body = response_json(upload).await;
    let session_id = upload_body["uploadSessionId"]
        .as_str()
        .expect("upload session includes an id")
        .parse::<Uuid>()
        .expect("upload session id should parse");
    let upload_token = upload_body["uploadToken"]
        .as_str()
        .expect("upload token present");

    let bytes = send_json_request(
        &app,
        "PUT",
        &format!("/api/v1/media/uploads/{session_id}/bytes"),
        &member.session_token,
        json!({"content": "hello"}),
    )
    .await;
    let _ = (bytes, upload_token);

    let media_object_id = upload_body["mediaObjectId"]
        .as_str()
        .expect("media object id should be a string")
        .to_string();
    let download = send_json_request(
        &app,
        "GET",
        &format!("/api/v1/media/{media_object_id}/download"),
        &member.session_token,
        json!({}),
    )
    .await;
    assert!(
        download.status() == StatusCode::OK || download.status() == StatusCode::CONFLICT,
        "download returns OK when ready or conflict while processing"
    );

    let report = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/conversations/{conversation_id}/reports"),
        &member.session_token,
        json!({"category": "harassment", "description": "Conversation content report."}),
    )
    .await;
    assert_eq!(report.status(), StatusCode::CREATED);

    let message_report = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/messages/{message_id}/reports"),
        &member.session_token,
        json!({"category": "harassment", "description": "Message content report."}),
    )
    .await;
    assert_eq!(message_report.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn research_project_conversations_reject_nonmembers() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "phase4_research_owner").await;
    let member = create_account_fixture(&database, "phase4_research_member").await;
    let outsider = create_account_fixture(&database, "phase4_research_outsider").await;
    let owner_actor_id: Uuid =
        sqlx::query_scalar("SELECT actor_id FROM human_actor_identities WHERE account_id = $1")
            .bind(owner.account_id)
            .fetch_one(&database)
            .await
            .expect("project owner human identity should exist");
    let member_actor_id: Uuid =
        sqlx::query_scalar("SELECT actor_id FROM human_actor_identities WHERE account_id = $1")
            .bind(member.account_id)
            .fetch_one(&database)
            .await
            .expect("project member human identity should exist");
    let mut project_transaction = database
        .begin()
        .await
        .expect("project transaction should begin");
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO research_projects (owner_actor_id, title) \
         VALUES ($1, 'Phase four private project') RETURNING id",
    )
    .bind(owner_actor_id)
    .fetch_one(&mut *project_transaction)
    .await
    .expect("private research project should be created");
    sqlx::query(
        "INSERT INTO research_project_members \
         (project_id, actor_id, member_role, membership_status, joined_at) \
         VALUES ($1, $2, 'owner', 'active', now()), \
                ($1, $3, 'researcher', 'active', now())",
    )
    .bind(project_id)
    .bind(owner_actor_id)
    .bind(member_actor_id)
    .execute(&mut *project_transaction)
    .await
    .expect("project memberships should be created");
    project_transaction
        .commit()
        .await
        .expect("project and memberships should commit");

    let app = phase_four_test_app(test_app_state(database.clone(), database_url));
    let member_conversation = send_json_request(
        &app,
        "POST",
        "/api/v1/conversations",
        &owner.session_token,
        json!({
            "conversationType": "research_project",
            "researchProjectId": project_id,
            "title": "Project members only",
            "memberAccountIds": [member.account_id]
        }),
    )
    .await;
    assert_eq!(member_conversation.status(), StatusCode::CREATED);

    let outsider_conversation = send_json_request(
        &app,
        "POST",
        "/api/v1/conversations",
        &owner.session_token,
        json!({
            "conversationType": "research_project",
            "researchProjectId": project_id,
            "title": "Must not include outsider",
            "memberAccountIds": [outsider.account_id]
        }),
    )
    .await;
    assert_ne!(outsider_conversation.status(), StatusCode::CREATED);
    let conversation_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM conversations WHERE research_project_id = $1")
            .bind(project_id)
            .fetch_one(&database)
            .await
            .expect("project conversation count should be readable");
    assert_eq!(conversation_count, 1);
}
