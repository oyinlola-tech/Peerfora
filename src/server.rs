use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;

use crate::{app::build_app, configs::settings::load_settings};

pub async fn start_http_server() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    start_logging(&settings.log_filter);

    let address = SocketAddr::new(settings.host, settings.port);
    let listener = TcpListener::bind(address).await?;
    info!(address = %listener.local_addr()?, "Card Peerfora API is listening");

    axum::serve(listener, build_app()).await?;
    Ok(())
}

fn start_logging(log_filter: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_filter));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
