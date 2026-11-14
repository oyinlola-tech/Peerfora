use super::support::*;
use axum::http::{StatusCode, header};

pub async fn verify_administrator_login_and_mfa(context: &TestContext) -> AdministratorAuthState {
    verify_administrator_login_rate_limit(context).await;
    let app = context.app.clone();
    let database = context.database.clone();
    let administrator_id = context.administrator_id;
    let allowed_origin_response = send_request(app.clone(), "GET", "/health", None, None).await;
    assert_eq!(
        allowed_origin_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(TEST_PUBLIC_URL)
    );
    assert_eq!(
        allowed_origin_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );
    let cors_preflight = send_cors_preflight_request(
        app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        TEST_PUBLIC_URL,
        "POST",
        "content-type",
    )
    .await;
    assert!(cors_preflight.status().is_success());
    assert_eq!(
        cors_preflight
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(TEST_PUBLIC_URL)
    );
    let rejected_origin_response = send_request_with_origin(
        app.clone(),
        "GET",
        "/health",
        None,
        Some("https://untrusted.example"),
        None,
    )
    .await;
    assert_eq!(rejected_origin_response.status(), StatusCode::FORBIDDEN);
    let missing_origin_cookie_response = send_request_with_origin(
        app.clone(),
        "POST",
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGOUT_ROUTE,
        Some("peerfora_admin_session=invalid"),
        None,
        None,
    )
    .await;
    assert_eq!(
        missing_origin_cookie_response.status(),
        StatusCode::FORBIDDEN
    );
    let liveness = send_request(app.clone(), "GET", "/health", None, None).await;
    assert_eq!(liveness.status(), StatusCode::OK);
    assert_eq!(
        liveness.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(liveness.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(liveness.headers().get("cache-control").unwrap(), "no-store");
    assert!(liveness.headers().get("content-security-policy").is_some());
    assert!(
        liveness
            .headers()
            .get(peerfora_api::constants::security::REQUEST_CORRELATION_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .is_some()
    );
    assert!(
        liveness
            .headers()
            .get("strict-transport-security")
            .is_none()
    );
    let readiness = send_request(app.clone(), "GET", "/health/ready", None, None).await;
    assert_eq!(readiness.status(), StatusCode::OK);

    let malformed_json = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/auth/login",
        None,
        Some("{".to_owned()),
    )
    .await;
    assert_eq!(malformed_json.status(), StatusCode::BAD_REQUEST);
    let oversized_json = format!(
        "{{\"email\":\"a@example.test\",\"password\":\"{}\"}}",
        "a".repeat(1_048_600)
    );
    let oversized_response = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/auth/login",
        None,
        Some(oversized_json),
    )
    .await;
    assert_eq!(oversized_response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let student_cookie_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some("peerfora_session=student-session-value"),
        None,
    )
    .await;
    assert_eq!(student_cookie_response.status(), StatusCode::UNAUTHORIZED);

    let student_bearer_response = send_request_with_bearer(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        "student-session-value",
    )
    .await;
    assert_eq!(student_bearer_response.status(), StatusCode::UNAUTHORIZED);
    let unauthenticated_account_search =
        send_request(app.clone(), "GET", "/api/v1/admin/accounts", None, None).await;
    assert_eq!(
        unauthenticated_account_search.status(),
        StatusCode::UNAUTHORIZED
    );

    let invalid_password = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": "incorrect-password" }),
    )
    .await;
    assert_eq!(invalid_password.status(), StatusCode::UNAUTHORIZED);

    let login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let login_json = response_json(login_response).await;
    assert_eq!(login_json["nextStep"], "enroll_mfa");
    let enrollment_token = login_json["challengeToken"]
        .as_str()
        .expect("MFA enrollment challenge token");

    let provisioning_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/enroll",
        serde_json::json!({ "challengeToken": enrollment_token }),
    )
    .await;
    assert_eq!(provisioning_response.status(), StatusCode::OK);
    let provisioning_json = response_json(provisioning_response).await;
    let provisioning_uri = provisioning_json["provisioningUri"]
        .as_str()
        .expect("authenticator provisioning URI");
    let totp_secret = provisioning_uri
        .split("secret=")
        .nth(1)
        .and_then(|value| value.split('&').next())
        .expect("TOTP secret in provisioning URI")
        .to_owned();
    let code = authenticator_code_from_uri(provisioning_uri);

    let enrollment_confirmation = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/confirm",
        serde_json::json!({ "challengeToken": enrollment_token, "code": code }),
    )
    .await;
    assert_eq!(enrollment_confirmation.status(), StatusCode::OK);
    let admin_cookie_header = enrollment_confirmation
        .headers()
        .get(header::SET_COOKIE)
        .expect("admin session cookie")
        .to_str()
        .expect("cookie header text");
    assert!(admin_cookie_header.starts_with("peerfora_admin_session="));
    assert!(admin_cookie_header.contains("Path=/api/v1/admin"));
    assert!(admin_cookie_header.contains("HttpOnly"));
    assert!(admin_cookie_header.contains("SameSite=Strict"));
    assert!(admin_cookie_header.contains("Secure"));
    assert!(admin_cookie_header.contains("Max-Age=28800"));
    let administrator_cookie = cookie_value(&enrollment_confirmation);
    let enrolled_profile = response_json(enrollment_confirmation).await;
    assert_eq!(
        enrolled_profile["administratorId"],
        administrator_id.to_string()
    );
    assert!(enrolled_profile.get("sessionToken").is_none());
    assert_eq!(enrolled_profile["roles"][0], "super_administrator");
    assert!(
        enrolled_profile["permissions"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    let recovery_codes = enrolled_profile["recoveryCodes"]
        .as_array()
        .expect("enrollment returns one-time recovery codes");
    assert_eq!(recovery_codes.len(), 10);
    let recovery_code = recovery_codes[0]
        .as_str()
        .expect("recovery code is returned as text")
        .to_owned();
    let stored_recovery_code: String = sqlx::query_scalar(
        "SELECT token_hash FROM administrator_recovery_codes
        WHERE administrator_id = $1 AND token_hash = $2",
    )
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &recovery_code,
    ))
    .fetch_one(&database)
    .await
    .expect("recovery code hash should persist");
    assert_eq!(
        stored_recovery_code,
        peerfora_api::utils::one_time_token::hash_one_time_token(&recovery_code)
    );

    let reused_enrollment_challenge = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/confirm",
        serde_json::json!({ "challengeToken": enrollment_token, "code": code }),
    )
    .await;
    assert_eq!(
        reused_enrollment_challenge.status(),
        StatusCode::UNAUTHORIZED
    );

    let current_profile = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(current_profile.status(), StatusCode::OK);
    let initial_overview = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/overview",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(initial_overview.status(), StatusCode::OK);
    let initial_overview_json = response_json(initial_overview).await;
    assert_eq!(initial_overview_json["registeredAccounts"], 0);
    assert_eq!(initial_overview_json["activeCommunities"], 0);

    AdministratorAuthState {
        administrator_cookie,
        totp_secret,
        code,
        recovery_code,
    }
}

async fn verify_administrator_login_rate_limit(context: &TestContext) {
    let email = format!("rate-limit-{}@example.test", uuid::Uuid::new_v4());
    for _ in 0..20 {
        let response = send_json(
            context.app.clone(),
            peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
            serde_json::json!({ "email": email, "password": "incorrect-password" }),
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    let limited_response = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        serde_json::json!({ "email": email, "password": "incorrect-password" }),
    )
    .await;
    assert_eq!(limited_response.status(), StatusCode::TOO_MANY_REQUESTS);
}
