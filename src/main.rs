#[tokio::main]
async fn main() {
    if let Err(error) = card_peerfora_api::index::start_http_server().await {
        eprintln!("Card Peerfora could not start: {error}");
        std::process::exit(1);
    }
}
