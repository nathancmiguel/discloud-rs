//! Prints the account information of the token owner.
//!
//! `cargo run --example client`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let user = client()?.get_user_info().await?;
    println!(
        "{} on plan {}: {}/{} MB of RAM used, {} apps",
        user.user_id,
        user.plan,
        user.used_ram,
        user.total_ram,
        user.apps.len()
    );
    Ok(())
}
