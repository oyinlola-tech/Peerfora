use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::{
    constants::admin_auth::ADMIN_SESSION_COOKIE_NAME,
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    repositories::admin_auth_repository,
    utils::one_time_token::hash_one_time_token,
    utils::request_context::get_request_correlation_id,
};

#[derive(Clone)]
pub struct AdministratorPermissionGuard {
    pub database: PgPool,
    pub permission_key: &'static str,
}

impl AdministratorPermissionGuard {
    pub fn new(database: PgPool, permission_key: &'static str) -> Self {
        Self {
            database,
            permission_key,
        }
    }
}

pub async fn require_active_administrator_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = read_administrator_session_token(&request) else {
        return AppError::from(AdminAuthError::InvalidSession).into_response();
    };
    let identity = match admin_auth_repository::find_active_administrator_session(
        &state.database,
        &hash_one_time_token(&token),
    )
    .await
    {
        Ok(Some(identity)) => identity,
        Ok(None) => return AppError::from(AdminAuthError::InvalidSession).into_response(),
        Err(_) => {
            return AppError::from(AdminAuthError::AuthenticationUnavailable).into_response();
        }
    };
    request.extensions_mut().insert(identity);
    next.run(request).await
}

pub async fn require_administrator_permission(
    State(guard): State<AdministratorPermissionGuard>,
    request: Request,
    next: Next,
) -> Response {
    let Some(identity) = request
        .extensions()
        .get::<AdministratorSessionIdentity>()
        .copied()
    else {
        return AppError::from(AdminAuthError::InvalidSession).into_response();
    };
    match admin_auth_repository::administrator_has_permission(
        &guard.database,
        identity.administrator_id,
        guard.permission_key,
    )
    .await
    {
        Ok(true) => next.run(request).await,
        Ok(false) => {
            if admin_auth_repository::record_administrator_permission_denial(
                &guard.database,
                identity.administrator_id,
                guard.permission_key,
                get_request_correlation_id(request.headers()),
            )
            .await
            .is_err()
            {
                return AppError::from(AdminAuthError::AuthenticationUnavailable).into_response();
            }
            AppError::from(AdminAuthError::PermissionDenied).into_response()
        }
        Err(_) => AppError::from(AdminAuthError::AuthenticationUnavailable).into_response(),
    }
}

fn read_administrator_session_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            (name == ADMIN_SESSION_COOKIE_NAME && !value.is_empty()).then(|| value.to_owned())
        })
}
