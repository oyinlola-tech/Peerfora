use sqlx::PgPool;
use tracing::{error, info};

use crate::{
    constants::phase_four::{
        EMAIL_FALLBACK_SUBJECT, EMAIL_OUTBOX_INTERVAL_SECONDS, EMAIL_OUTBOX_MAX_ATTEMPTS,
        MEDIA_PROCESSING_INTERVAL_SECONDS, MEDIA_PROCESSING_MAX_ATTEMPTS, OUTBOX_CLAIM_SECONDS,
    },
    interfaces::email_sender::{EmailMessage, EmailSender},
    repositories::communication_repository::{media_repository, outbox_repository},
    utils::encrypted_secret::decrypt_secret,
};

pub async fn run_email_delivery_worker(
    pool: PgPool,
    email_sender: Option<std::sync::Arc<dyn EmailSender>>,
    encryption_key: Option<String>,
    worker_id: String,
) {
    if email_sender.is_none() {
        info!("Email delivery worker starting without an email sender; deliveries remain queued");
    }
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
        EMAIL_OUTBOX_INTERVAL_SECONDS,
    ));
    loop {
        interval.tick().await;
        if email_sender.is_none() || encryption_key.is_none() {
            continue;
        }
        process_email_deliveries(
            &pool,
            email_sender.as_ref().expect("email sender is available"),
            encryption_key.as_ref().unwrap(),
            &worker_id,
        )
        .await;
    }
}

async fn process_email_deliveries(
    pool: &PgPool,
    email_sender: &std::sync::Arc<dyn EmailSender>,
    encryption_key: &str,
    worker_id: &str,
) {
    while let Some((delivery_id, recipient, encrypted_payload)) =
        outbox_repository::claim_email_delivery(pool, worker_id, OUTBOX_CLAIM_SECONDS)
            .await
            .unwrap_or(None)
    {
        let delivered = match decrypt_payload(&encrypted_payload, encryption_key).await {
            Ok(message) => email_sender.send_email(&recipient, &message).await.is_ok(),
            Err(error) => {
                error!(delivery_id = %delivery_id, error = %error, "Email delivery payload could not be decrypted");
                outbox_repository::mark_email_delivery_failed(
                    pool,
                    delivery_id,
                    "Encrypted payload could not be decrypted",
                    OUTBOX_CLAIM_SECONDS,
                    EMAIL_OUTBOX_MAX_ATTEMPTS,
                )
                .await
                .ok();
                continue;
            }
        };
        if delivered {
            outbox_repository::mark_email_delivery_processed(pool, delivery_id, worker_id)
                .await
                .ok();
        } else {
            outbox_repository::mark_email_delivery_failed(
                pool,
                delivery_id,
                "Email provider rejected the delivery",
                OUTBOX_CLAIM_SECONDS,
                EMAIL_OUTBOX_MAX_ATTEMPTS,
            )
            .await
            .ok();
        }
    }
}

async fn decrypt_payload(
    encrypted_payload: &str,
    encryption_key: &str,
) -> Result<EmailMessage, String> {
    let bytes =
        decrypt_secret(encrypted_payload, encryption_key).map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok(EmailMessage {
        subject: payload
            .get("subject")
            .and_then(|value| value.as_str())
            .unwrap_or(EMAIL_FALLBACK_SUBJECT)
            .to_owned(),
        plain_text: payload
            .get("plain_text")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
        html: payload
            .get("html")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
    })
}

pub async fn run_media_processing_worker(pool: PgPool, worker_id: String) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
        MEDIA_PROCESSING_INTERVAL_SECONDS,
    ));
    loop {
        interval.tick().await;
        process_media_jobs(&pool, &worker_id).await;
    }
}

async fn process_media_jobs(pool: &PgPool, worker_id: &str) {
    while let Some(job_id) = media_repository::claim_processing_job(pool, worker_id)
        .await
        .unwrap_or(None)
    {
        match media_repository::complete_processing_job(pool, job_id).await {
            Ok(()) => {}
            Err(error) => {
                error!(job_id = %job_id, error = %error, "Media processing job failed");
                media_repository::fail_processing_job(
                    pool,
                    job_id,
                    "Media processing failed",
                    MEDIA_PROCESSING_MAX_ATTEMPTS,
                )
                .await
                .ok();
            }
        }
    }
}
