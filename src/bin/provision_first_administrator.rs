use peerfora_api::{
    configs::settings::load_settings, database::connect_database,
    services::admin_auth::commands::provision_first_administrator_command::provision_first_administrator,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    if let Err(error) = provision_initial_administrator().await {
        eprintln!("Administrator provisioning failed: {error}");
        std::process::exit(1);
    }
}

async fn provision_initial_administrator() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    let email = read_administrator_email()?;
    let password = rpassword::prompt_password("Administrator password: ")?;
    let database = connect_database(&settings.database_url).await?;
    let administrator_id = provision_first_administrator(&database, &email, password).await?;
    println!("Provisioned the initial administrator: {administrator_id}");
    println!(
        "Sign in and complete mandatory authenticator-app enrollment before using admin APIs."
    );
    Ok(())
}

fn read_administrator_email() -> Result<String, io::Error> {
    print!("Administrator email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    Ok(email.trim().to_owned())
}
