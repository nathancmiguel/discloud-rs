//! Deletes an application permanently. This cannot be undone.
//!
//! `cargo run --example apps_manage_delete -- APP_ID --yes`

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
    if std::env::args().nth(2).as_deref() != Some("--yes") {
        return Err(format!("this deletes {id} permanently; re-run with --yes to confirm").into());
    }
    client()?.delete_app(&id).await?;
    println!("Deleted {id}");
    Ok(())
}
