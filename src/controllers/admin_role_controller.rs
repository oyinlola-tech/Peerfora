use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dtos::admin_role::{
        AdministrativePermissionItem, AdministrativePermissionsResponse, AdministrativeRoleItem,
        AdministrativeRolesResponse, AdministratorPermissionKeysResponse,
        CreateAdministrativeRoleRequest, ReplaceAdministrativeRolePermissionsRequest,
        ReplaceAdministratorRolesRequest, UpdateAdministrativeRoleRequest,
    },
    errors::app_error::AppError,
    models::{
        admin_identity::AdministratorSessionIdentity, admin_role::AdministrativeRoleRecord,
        app_state::AppState,
    },
    services::access_control::{
        commands::{
            create_administrative_role_command::create_administrative_role,
            delete_administrative_role_command::delete_administrative_role,
            replace_administrative_role_permissions_command::replace_administrative_role_permissions,
            replace_administrator_roles_command::replace_administrator_roles as replace_administrator_role_assignments,
            update_administrative_role_command::update_administrative_role,
        },
        queries::{
            find_administrative_role_query::find_administrative_role,
            list_administrative_permissions_query::list_administrative_permissions,
            list_administrative_roles_query::list_administrative_roles,
            list_administrator_role_assignments_query::{
                list_administrator_permission_keys, list_administrator_role_assignments,
            },
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn list_roles(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let roles = list_administrative_roles(&state.database).await?;
    Ok(Json(AdministrativeRolesResponse {
        items: roles.into_iter().map(role_item).collect(),
    }))
}

pub async fn show_role(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let role = find_administrative_role(&state.database, role_id).await?;
    Ok(Json(role_item(role)))
}

pub async fn create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Json(request): Json<CreateAdministrativeRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role_id = create_administrative_role(
        &state.database,
        actor.administrator_id,
        &request.role_key,
        &request.display_name,
        &request.description,
        &request.permission_keys,
        get_request_correlation_id(&headers),
    )
    .await?;
    let role = find_administrative_role(&state.database, role_id).await?;
    Ok((StatusCode::CREATED, Json(role_item(role))))
}

pub async fn update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(role_id): Path<Uuid>,
    Json(request): Json<UpdateAdministrativeRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    update_administrative_role(
        &state.database,
        actor.administrator_id,
        role_id,
        request.display_name,
        request.description,
        request.is_active,
        get_request_correlation_id(&headers),
    )
    .await?;
    let role = find_administrative_role(&state.database, role_id).await?;
    Ok(Json(role_item(role)))
}

pub async fn delete_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    delete_administrative_role(
        &state.database,
        actor.administrator_id,
        role_id,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let permissions = list_administrative_permissions(&state.database).await?;
    Ok(Json(AdministrativePermissionsResponse {
        items: permissions
            .into_iter()
            .map(|permission| AdministrativePermissionItem {
                permission_key: permission.permission_key,
                description: permission.description,
                is_assignable: permission.is_assignable,
                is_sensitive: permission.is_sensitive,
            })
            .collect(),
    }))
}

pub async fn replace_role_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(role_id): Path<Uuid>,
    Json(request): Json<ReplaceAdministrativeRolePermissionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    replace_administrative_role_permissions(
        &state.database,
        actor.administrator_id,
        role_id,
        &request.permission_keys,
        get_request_correlation_id(&headers),
    )
    .await?;
    let role = find_administrative_role(&state.database, role_id).await?;
    Ok(Json(role_item(role)))
}

pub async fn list_administrator_roles(
    State(state): State<AppState>,
    Path(administrator_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let roles = list_administrator_role_assignments(&state.database, administrator_id).await?;
    Ok(Json(AdministrativeRolesResponse {
        items: roles.into_iter().map(role_item).collect(),
    }))
}

pub async fn replace_administrator_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(administrator_id): Path<Uuid>,
    Json(request): Json<ReplaceAdministratorRolesRequest>,
) -> Result<impl IntoResponse, AppError> {
    replace_administrator_role_assignments(
        &state.database,
        actor.administrator_id,
        administrator_id,
        &request.role_ids,
        get_request_correlation_id(&headers),
    )
    .await?;
    let roles = list_administrator_role_assignments(&state.database, administrator_id).await?;
    Ok(Json(AdministrativeRolesResponse {
        items: roles.into_iter().map(role_item).collect(),
    }))
}

pub async fn list_administrator_permissions(
    State(state): State<AppState>,
    Path(administrator_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let permission_keys =
        list_administrator_permission_keys(&state.database, administrator_id).await?;
    Ok(Json(AdministratorPermissionKeysResponse {
        administrator_id,
        permission_keys,
    }))
}

fn role_item(role: AdministrativeRoleRecord) -> AdministrativeRoleItem {
    AdministrativeRoleItem {
        id: role.id,
        role_key: role.role_key,
        display_name: role.display_name,
        description: role.description,
        is_system_role: role.is_system_role,
        is_active: role.is_active,
        permissions: role.permissions,
    }
}
