use super::support::*;
use axum::http::StatusCode;

pub async fn verify_account_and_community_administration(
    context: &TestContext,
    administrator: &AdministratorAuthState,
) -> AccountCommunityFixture {
    let app = context.app.clone();
    let database = context.database.clone();
    let administrator_cookie = administrator.administrator_cookie.clone();
    let student_account_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO accounts
        (id, email, username, display_name, password_hash, eligibility_country_code,
            age_policy_version, terms_accepted_at, email_verified_at)
    VALUES ($1, 'student-search@example.test', 'peerfora_student_search',
        'Private display name', 'test-only-hash', 'NG', 'global-18-v1', now(), now())",
    )
    .bind(student_account_id)
    .execute(&database)
    .await
    .expect("account fixture should persist");
    sqlx::query(
        "UPDATE user_profiles SET academic_level_id = 'secondary_school' WHERE account_id = $1",
    )
    .bind(student_account_id)
    .execute(&database)
    .await
    .expect("academic level fixture should persist");
    let student_session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at)
    VALUES ($1, $2, now() + interval '30 days')",
    )
    .bind(student_account_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &student_session_token,
    ))
    .execute(&database)
    .await
    .expect("active student session fixture should persist");
    let account_search_response = send_request(
    app.clone(),
    "GET",
    "/api/v1/admin/accounts?search=peerfora_student&status=active&countryCode=ng&academicLevelId=secondary_school&limit=10&offset=0",
    Some(&format!("peerfora_admin_session={administrator_cookie}")),
    None,
)
.await;
    assert_eq!(account_search_response.status(), StatusCode::OK);
    let account_search_json = response_json(account_search_response).await;
    assert_eq!(account_search_json["totalCount"], 1);
    assert_eq!(
        account_search_json["items"][0]["id"],
        student_account_id.to_string()
    );
    assert_eq!(account_search_json["items"][0]["emailVerified"], true);
    assert!(account_search_json["items"][0].get("dateOfBirth").is_none());
    assert!(
        account_search_json["items"][0]
            .get("eligibilityEvidence")
            .is_none()
    );
    assert!(
        account_search_json["items"][0]
            .get("passwordHash")
            .is_none()
    );
    let account_detail_response = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/admin/accounts/{student_account_id}"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(account_detail_response.status(), StatusCode::OK);
    let missing_account_response = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/admin/accounts/{}", uuid::Uuid::new_v4()),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(missing_account_response.status(), StatusCode::NOT_FOUND);
    let invalid_account_filter = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/accounts?status=admin",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(invalid_account_filter.status(), StatusCode::BAD_REQUEST);
    let active_student_session = send_request(
        app.clone(),
        "GET",
        "/api/v1/auth/me",
        Some(&format!("peerfora_session={student_session_token}")),
        None,
    )
    .await;
    assert_eq!(active_student_session.status(), StatusCode::OK);
    let suspend_student_account = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/accounts/{student_account_id}/suspend"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(serde_json::json!({ "reasonCode": "account_compromise" }).to_string()),
    )
    .await;
    assert_eq!(suspend_student_account.status(), StatusCode::NO_CONTENT);
    let suspended_student_account_status: String =
        sqlx::query_scalar("SELECT account_status FROM accounts WHERE id = $1")
            .bind(student_account_id)
            .fetch_one(&database)
            .await
            .expect("suspension should be persisted");
    assert_eq!(suspended_student_account_status, "suspended");
    let suspended_student_session_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM account_sessions WHERE account_id = $1")
            .bind(student_account_id)
            .fetch_one(&database)
            .await
            .expect("suspension should revoke student sessions");
    assert_eq!(suspended_student_session_count, 0);
    let rejected_suspended_student_session = send_request(
        app.clone(),
        "GET",
        "/api/v1/auth/me",
        Some(&format!("peerfora_session={student_session_token}")),
        None,
    )
    .await;
    assert_eq!(
        rejected_suspended_student_session.status(),
        StatusCode::UNAUTHORIZED
    );
    let reinstatement = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/accounts/{student_account_id}/reinstate"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(serde_json::json!({ "reasonCode": "other" }).to_string()),
    )
    .await;
    assert_eq!(reinstatement.status(), StatusCode::NO_CONTENT);
    let old_session_after_reinstatement = send_request(
        app.clone(),
        "GET",
        "/api/v1/auth/me",
        Some(&format!("peerfora_session={student_session_token}")),
        None,
    )
    .await;
    assert_eq!(
        old_session_after_reinstatement.status(),
        StatusCode::UNAUTHORIZED
    );

    let community_id = uuid::Uuid::new_v4();
    let mut community_transaction = database
        .begin()
        .await
        .expect("private community fixture transaction should start");
    sqlx::query(
        "INSERT INTO communities
        (id, slug, display_name, description, category, creator_account_id,
            owner_account_id, visibility)
    VALUES ($1, 'moderation-cohort', 'Moderation Cohort', 'Private test-only description',
        'general', $2, $2, 'private')",
    )
    .bind(community_id)
    .bind(student_account_id)
    .execute(&mut *community_transaction)
    .await
    .expect("private community fixture should persist");
    sqlx::query(
        "INSERT INTO community_members
        (community_id, account_id, membership_status, community_role, joined_at)
    VALUES ($1, $2, 'active', 'owner', now())",
    )
    .bind(community_id)
    .bind(student_account_id)
    .execute(&mut *community_transaction)
    .await
    .expect("private community owner membership should persist");
    community_transaction
        .commit()
        .await
        .expect("community owner consistency constraints should pass");
    let account_owner_suspension = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/accounts/{student_account_id}/suspend"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(serde_json::json!({ "reasonCode": "policy_violation" }).to_string()),
    )
    .await;
    assert_eq!(account_owner_suspension.status(), StatusCode::CONFLICT);
    let private_community_search = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/communities?search=moderation&category=general&status=active",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(private_community_search.status(), StatusCode::OK);
    let private_community_search_json = response_json(private_community_search).await;
    assert_eq!(private_community_search_json["totalCount"], 1);
    assert_eq!(
        private_community_search_json["items"][0]["visibility"],
        "private"
    );
    let private_community_details = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/admin/communities/{community_id}"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(private_community_details.status(), StatusCode::OK);
    let private_community_details_json = response_json(private_community_details).await;
    assert!(private_community_details_json.get("description").is_none());
    let suspend_community = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/communities/{community_id}/suspend"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(serde_json::json!({ "reasonCode": "policy_violation" }).to_string()),
    )
    .await;
    assert_eq!(suspend_community.status(), StatusCode::NO_CONTENT);
    let suspended_community_status: String =
        sqlx::query_scalar("SELECT lifecycle_status FROM communities WHERE id = $1")
            .bind(community_id)
            .fetch_one(&database)
            .await
            .expect("community suspension should persist");
    assert_eq!(suspended_community_status, "suspended");
    let restore_community = send_request(
        app.clone(),
        "POST",
        &format!("/api/v1/admin/communities/{community_id}/restore"),
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        Some(serde_json::json!({ "reasonCode": "other" }).to_string()),
    )
    .await;
    assert_eq!(restore_community.status(), StatusCode::NO_CONTENT);

    AccountCommunityFixture {
        student_account_id,
        community_id,
    }
}
