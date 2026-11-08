//! Changes the RAM of an application.
//!
//! `cargo run --example apps_manage_ram -- APP_ID 256`

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
    use discloud_rs::AppRamError;

    let id = app_id_arg()?;
    let ram: u32 = std::env::args()
        .nth(2)
        .ok_or("pass the RAM in MB as the second argument")?
        .parse()?;

    match client()?.set_app_ram(&id, ram).await {
        Ok(()) => println!("RAM set to {ram} MB"),
        Err(AppRamError::ForbiddenQuantity(reason)) => println!("Not allowed: {reason}"),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
