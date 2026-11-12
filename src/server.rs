use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::info;

use crate::{
    app::build_app, configs::settings::load_settings, database::connect_database,
    errors::app_error::AppError, integrations::email::smtp_email_sender::SmtpEmailSender,
    interfaces::email_sender::EmailSender, models::app_state::AppState,
};

pub async fn start_http_server() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    start_logging(&settings.log_filter);
    let database = connect_database(&settings.database_url).await?;
    let email_sender: Option<Arc<dyn EmailSender>> = SmtpEmailSender::from_settings(&settings)
        .map_err(AppError::from)?
        .map(|sender| Arc::new(sender) as Arc<dyn EmailSender>);
    let state = AppState::from_settings(database, &settings, email_sender);

    let address = SocketAddr::new(settings.host, settings.port);
    let listener = TcpListener::bind(address).await?;
    info!(address = %listener.local_addr()?, "Peerfora API is listening");

    axum::serve(listener, build_app(state)).await?;
    Ok(())
}

fn start_logging(log_filter: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_filter));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
