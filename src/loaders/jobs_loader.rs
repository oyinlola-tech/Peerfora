use tokio::time::{Duration, interval};

use crate::{
    constants::account_auth::EXPIRED_AUTH_TOKEN_CLEANUP_INTERVAL_SECONDS,
    jobs::cleanup_expired_auth_tokens_job::cleanup_expired_auth_tokens,
    models::app_state::AppState,
};

pub fn start_background_jobs(state: AppState) {
    let database = state.database.clone();
    tokio::spawn(async move {
        let mut cleanup_interval = interval(Duration::from_secs(
            EXPIRED_AUTH_TOKEN_CLEANUP_INTERVAL_SECONDS,
        ));
        loop {
            cleanup_interval.tick().await;
            cleanup_expired_auth_tokens(&database).await;
        }
    });
}
