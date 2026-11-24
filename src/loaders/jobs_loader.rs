use tokio::time::{Duration, interval};

use crate::{
    constants::account_auth::EXPIRED_AUTH_TOKEN_CLEANUP_INTERVAL_SECONDS,
    jobs::{
        cleanup_expired_auth_tokens_job::cleanup_expired_auth_tokens,
        phase_four_workers::{run_email_delivery_worker, run_media_processing_worker},
    },
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

    tokio::spawn(run_email_delivery_worker(
        state.database.clone(),
        state.email_sender.clone(),
        state.admin_mfa_encryption_key.clone(),
        "email-delivery-worker".to_owned(),
    ));

    tokio::spawn(run_media_processing_worker(
        state.database.clone(),
        "media-processing-worker".to_owned(),
    ));
}
