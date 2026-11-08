//! Removes a moderator from an application.
//!
//! `cargo run --example apps_team_manage_remove -- APP_ID MOD_USER_ID`

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
    let id = app_id_arg()?;
    let mod_id = std::env::args()
        .nth(2)
        .ok_or("pass the moderator's user ID")?;
    client()?.remove_app_mod(&id, &mod_id).await?;
    println!("Removed {mod_id}");
    Ok(())
}
