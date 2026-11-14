use super::support::*;
use axum::http::StatusCode;

pub async fn verify_administrative_role_management(
    context: &TestContext,
    root_administrator: &AdministratorAuthState,
) {
    let cookie = format!(
        "peerfora_admin_session={}",
        root_administrator.administrator_cookie
    );
    let app = context.app.clone();
    let roles_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/roles",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(roles_response.status(), StatusCode::OK);
    let roles_json = response_json(roles_response).await;
    assert!(roles_json["items"].as_array().is_some_and(|roles| {
        roles
            .iter()
            .any(|role| role["roleKey"] == "super_administrator")
    }));

    let catalog_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/permissions",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(catalog_response.status(), StatusCode::OK);
    let catalog_json = response_json(catalog_response).await;
    assert!(catalog_json["items"].as_array().is_some_and(|permissions| {
        permissions.iter().any(|permission| {
            permission["permissionKey"] == "accounts.read" && permission["isAssignable"] == true
        })
    }));

    let create_response = send_admin_json(
        app.clone(),
        "POST",
        "/api/v1/admin/roles",
        serde_json::json!({
            "roleKey": "student_support_helper",
            "displayName": "Student Support Helper",
            "description": "Read account records for authorized student support.",
            "permissionKeys": ["accounts.read"]
        }),
        &cookie,
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created_role = response_json(create_response).await;
    let role_id = created_role["id"]
        .as_str()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .expect("custom role should have a stable identifier");
    assert_eq!(created_role["permissions"][0], "accounts.read");

    let restricted_permission = send_admin_json(
        app.clone(),
        "PUT",
        &format!("/api/v1/admin/roles/{role_id}/permissions"),
        serde_json::json!({"permissionKeys": ["verification.read"]}),
        &cookie,
    )
    .await;
    assert_eq!(restricted_permission.status(), StatusCode::BAD_REQUEST);

    let invited_id =
        administrator_id_by_email(&context.database, "new-administrator@example.test").await;
    let assignment_response = send_admin_json(
        app.clone(),
        "PUT",
        &format!("/api/v1/admin/administrators/{invited_id}/roles"),
        serde_json::json!({"roleIds": [role_id]}),
        &cookie,
    )
    .await;
    assert_eq!(assignment_response.status(), StatusCode::OK);
    let assignment_json = response_json(assignment_response).await;
    assert_eq!(
        assignment_json["items"][0]["roleKey"],
        "student_support_helper"
    );

    let effective_permissions = send_request(
        app.clone(),
        "GET",
        &format!("/api/v1/admin/administrators/{invited_id}/permissions"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(effective_permissions.status(), StatusCode::OK);
    let effective_permissions_json = response_json(effective_permissions).await;
    assert_eq!(
        effective_permissions_json["permissionKeys"][0],
        "accounts.read"
    );
    let delete_in_use_role = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/admin/roles/{role_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(delete_in_use_role.status(), StatusCode::CONFLICT);

    let delegator_role = send_admin_json(
        app.clone(),
        "POST",
        "/api/v1/admin/roles",
        serde_json::json!({
            "roleKey": "limited_role_delegator",
            "displayName": "Limited Role Delegator",
            "description": "Can delegate only the permissions assigned to this role.",
            "permissionKeys": [
                "accounts.read",
                "admin.roles.read",
                "admin.roles.create",
                "admin.roles.assign",
                "admin.permissions.assign"
            ]
        }),
        &cookie,
    )
    .await;
    assert_eq!(delegator_role.status(), StatusCode::CREATED);
    let delegator_role_json = response_json(delegator_role).await;
    let delegator_role_id = delegator_role_json["id"]
        .as_str()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .expect("delegation role should have a stable identifier");
    let replace_with_delegator = send_admin_json(
        app.clone(),
        "PUT",
        &format!("/api/v1/admin/administrators/{invited_id}/roles"),
        serde_json::json!({"roleIds": [delegator_role_id]}),
        &cookie,
    )
    .await;
    assert_eq!(replace_with_delegator.status(), StatusCode::OK);
    let delete_unassigned_role = send_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/admin/roles/{role_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(delete_unassigned_role.status(), StatusCode::NO_CONTENT);

    let delegated_cookie =
        complete_invited_administrator_mfa(context, "new-administrator@example.test").await;
    let delegated_cookie = format!("peerfora_admin_session={delegated_cookie}");
    let unauthorized_role_creation = send_admin_json(
        app.clone(),
        "POST",
        "/api/v1/admin/roles",
        serde_json::json!({
            "roleKey": "delegation_escalation_attempt",
            "displayName": "Escalation Attempt",
            "description": "This must be rejected because the creator lacks this permission.",
            "permissionKeys": ["admin.overview.read"]
        }),
        &delegated_cookie,
    )
    .await;
    assert_eq!(unauthorized_role_creation.status(), StatusCode::BAD_REQUEST);
    let delegated_role_creation = send_admin_json(
        app.clone(),
        "POST",
        "/api/v1/admin/roles",
        serde_json::json!({
            "roleKey": "delegated_account_reader",
            "displayName": "Delegated Account Reader",
            "description": "The delegated actor may grant only its own accounts.read permission.",
            "permissionKeys": ["accounts.read"]
        }),
        &delegated_cookie,
    )
    .await;
    assert_eq!(delegated_role_creation.status(), StatusCode::CREATED);
    let delegated_role_json = response_json(delegated_role_creation).await;
    let delegated_role_id = delegated_role_json["id"]
        .as_str()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .expect("delegated role should have a stable identifier");
    let delegated_permission_update = send_admin_json(
        app.clone(),
        "PUT",
        &format!("/api/v1/admin/roles/{delegated_role_id}/permissions"),
        serde_json::json!({"permissionKeys": ["accounts.read"]}),
        &delegated_cookie,
    )
    .await;
    assert_eq!(delegated_permission_update.status(), StatusCode::OK);
    let unauthorized_super_assignment = send_admin_json(
        app.clone(),
        "PUT",
        &format!("/api/v1/admin/administrators/{invited_id}/roles"),
        serde_json::json!({"roleIds": [root_role_id(&context.database).await]}),
        &delegated_cookie,
    )
    .await;
    assert_eq!(
        unauthorized_super_assignment.status(),
        StatusCode::BAD_REQUEST
    );

    let root_role = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM roles WHERE role_key = 'super_administrator'",
    )
    .fetch_one(&context.database)
    .await
    .expect("seeded Super Admin role should be present");
    let root_assignment = send_admin_json(
        app.clone(),
        "PUT",
        &format!(
            "/api/v1/admin/administrators/{}/roles",
            context.administrator_id
        ),
        serde_json::json!({"roleIds": []}),
        &cookie,
    )
    .await;
    assert_eq!(root_assignment.status(), StatusCode::FORBIDDEN);
    let reserved_role_update = send_admin_json(
        app.clone(),
        "PATCH",
        &format!("/api/v1/admin/roles/{root_role}"),
        serde_json::json!({"displayName": "Changed root role"}),
        &cookie,
    )
    .await;
    assert_eq!(reserved_role_update.status(), StatusCode::FORBIDDEN);
    let seeded_root_marker_update =
        sqlx::query("UPDATE administrators SET is_seeded_root = FALSE WHERE id = $1")
            .bind(context.administrator_id)
            .execute(&context.database)
            .await;
    assert!(seeded_root_marker_update.is_err());
    let seeded_root_deletion = sqlx::query("DELETE FROM administrators WHERE id = $1")
        .bind(context.administrator_id)
        .execute(&context.database)
        .await;
    assert!(seeded_root_deletion.is_err());
    let seeded_role_assignment_removal = sqlx::query(
        "DELETE FROM administrator_roles
        WHERE administrator_id = $1 AND role_key = 'super_administrator'",
    )
    .bind(context.administrator_id)
    .execute(&context.database)
    .await;
    assert!(seeded_role_assignment_removal.is_err());
    let reserved_role_deletion =
        sqlx::query("DELETE FROM roles WHERE role_key = 'super_administrator'")
            .execute(&context.database)
            .await;
    assert!(reserved_role_deletion.is_err());
    let reserved_permission_removal = sqlx::query(
        "DELETE FROM role_permissions
        WHERE role_key = 'super_administrator' AND permission_key = 'admin.roles.read'",
    )
    .execute(&context.database)
    .await;
    assert!(reserved_permission_removal.is_err());

    let invited_custom_role = send_admin_json(
        context.app.clone(),
        "POST",
        "/api/v1/admin/roles",
        serde_json::json!({
            "roleKey": "custom_invitation_role",
            "displayName": "Custom Invitation Role",
            "description": "A role selected from the active RBAC catalog.",
            "permissionKeys": ["accounts.read"]
        }),
        &cookie,
    )
    .await;
    assert_eq!(invited_custom_role.status(), StatusCode::CREATED);
    let custom_invitation = send_request(
        context.app.clone(),
        "POST",
        "/api/v1/admin/invitations",
        Some(&cookie),
        Some(
            serde_json::json!({
                "email": "custom-role-invite@example.test",
                "roleKey": "custom_invitation_role"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(custom_invitation.status(), StatusCode::ACCEPTED);
}

async fn complete_invited_administrator_mfa(context: &TestContext, email: &str) -> String {
    let login = send_json(
        context.app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({
            "email": email,
            "password": "new-admin-password-with-sixteen-bytes"
        }),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let challenge = response_json(login).await["challengeToken"]
        .as_str()
        .expect("invited administrator enrollment challenge")
        .to_owned();
    let enrollment = send_json(
        context.app.clone(),
        "/api/v1/admin/auth/mfa/enroll",
        serde_json::json!({"challengeToken": challenge}),
    )
    .await;
    assert_eq!(enrollment.status(), StatusCode::OK);
    let provisioning_uri = response_json(enrollment).await["provisioningUri"]
        .as_str()
        .expect("authenticator setup URI")
        .to_owned();
    let confirmation = send_json(
        context.app.clone(),
        "/api/v1/admin/auth/mfa/confirm",
        serde_json::json!({
            "challengeToken": challenge,
            "code": authenticator_code_from_uri(&provisioning_uri)
        }),
    )
    .await;
    assert_eq!(confirmation.status(), StatusCode::OK);
    cookie_value(&confirmation)
}

async fn root_role_id(database: &sqlx::PgPool) -> uuid::Uuid {
    sqlx::query_scalar("SELECT id FROM roles WHERE role_key = 'super_administrator'")
        .fetch_one(database)
        .await
        .expect("reserved Super Admin role should exist")
}

async fn administrator_id_by_email(database: &sqlx::PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT id FROM administrators WHERE email = $1")
        .bind(email)
        .fetch_one(database)
        .await
        .expect("invited test administrator should exist")
}

async fn send_admin_json(
    app: axum::Router,
    method: &str,
    path: &str,
    body: serde_json::Value,
    cookie: &str,
) -> axum::response::Response {
    send_request(
        app,
        method,
        path,
        Some(cookie),
        Some(serde_json::to_string(&body).expect("admin JSON request body")),
    )
    .await
}
