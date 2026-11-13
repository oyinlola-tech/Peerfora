use peerfora_api::{
    configs::settings::load_settings, database::connect_database,
    services::admin_auth::commands::provision_first_administrator_command::provision_first_administrator,
};

#[tokio::main]
async fn main() {
    if let Err(error) = provision_initial_administrator().await {
        eprintln!("Administrator provisioning failed: {error}");
        std::process::exit(1);
    }
}

async fn provision_initial_administrator() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    let email = std::env::var("PEERFORA_BOOTSTRAP_ADMIN_EMAIL")?;
    let password = std::env::var("PEERFORA_BOOTSTRAP_ADMIN_PASSWORD")?;
    let database = connect_database(&settings.database_url).await?;
    let administrator_id = provision_first_administrator(&database, &email, password).await?;
    println!("Provisioned the initial administrator: {administrator_id}");
    println!(
        "Sign in and complete mandatory authenticator-app enrollment before using admin APIs."
    );
    Ok(())
}
