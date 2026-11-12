use std::{env, net::IpAddr};

pub struct Settings {
    pub host: IpAddr,
    pub port: u16,
    pub log_filter: String,
}

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let host = env::var("PEERFORA_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PEERFORA_PORT").unwrap_or_else(|_| "3000".to_owned());
    let log_filter = env::var("PEERFORA_LOG").unwrap_or_else(|_| "info".to_owned());

    Ok(Settings {
        host: host.parse()?,
        port: port.parse()?,
        log_filter,
    })
}
