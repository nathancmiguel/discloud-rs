//! Prints the recent logs of an application.
//!
//! `cargo run --example apps_logs -- APP_ID`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

fn app_id_arg() -> Result<String, Box<dyn Error>> {
    std::env::args()
        .nth(1)
        .ok_or_else(|| "pass the application ID as the first argument".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let logs = client()?.get_app_logs(&app_id_arg()?).await?;
    println!("{}", logs.terminal.small.as_deref().unwrap_or("<no logs>"));
    println!("Full logs: {}", logs.terminal.url);
    Ok(())
}
