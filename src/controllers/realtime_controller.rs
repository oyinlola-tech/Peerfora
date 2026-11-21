use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use futures_util::Stream;
use uuid::Uuid;

use crate::{
    constants::{
        account_auth::SESSION_COOKIE_NAME,
        phase_four::{REALTIME_HEARTBEAT_INTERVAL_SECONDS, REALTIME_SSE_PING_TEXT},
    },
    controllers::auth_cookie,
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::accounts::queries::account_session_queries,
    services::realtime::hub::RealtimeEvent,
};

pub async fn realtime_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let receiver = state.realtime_hub.subscribe();
    let stream = filtered_event_stream(receiver, account_id);
    Ok(axum::response::sse::Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(
                REALTIME_HEARTBEAT_INTERVAL_SECONDS,
            ))
            .text(REALTIME_SSE_PING_TEXT),
    ))
}

fn filtered_event_stream(
    receiver: tokio::sync::broadcast::Receiver<RealtimeEvent>,
    account_id: Uuid,
) -> impl Stream<Item = Result<axum::response::sse::Event, AppError>> + 'static {
    futures_util::stream::unfold(receiver, move |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) if event.account_id == account_id => {
                    let data = match serde_json::to_string(&event) {
                        Ok(data) => data,
                        Err(_) => continue,
                    };
                    let event = axum::response::sse::Event::default()
                        .event(event.event_type.clone())
                        .data(data);
                    return Some((Ok(event), receiver));
                }
                Ok(_) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
}

async fn require_account_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Err(AccountAuthError::SessionInvalid.into());
    };
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(account.id)
}
