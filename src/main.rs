#[tokio::main]
async fn main() {
    if let Err(error) = peerfora_api::index::start_http_server().await {
        eprintln!("Peerfora could not start: {error}");
        std::process::exit(1);
    }
}
