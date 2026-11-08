//! Prints a backup download link for an application.
//!
//! `cargo run --example apps_backup -- APP_ID`

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
    let backup = client()?.get_app_backup(&app_id_arg()?).await?;
    println!("Download the backup of {} at {}", backup.id, backup.url);
    Ok(())
}
