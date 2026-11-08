//! Lists the available database engines and your databases.
//!
//! `cargo run --example databases`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = client()?;
    for preset in client.get_database_presets().await? {
        println!("{}: versions {}", preset.r#type, preset.versions.join(", "));
    }
    for db in client.get_databases().await? {
        println!(
            "{} ({} {}) - {} MB",
            db.display_name, db.db.r#type, db.db.version, db.total_ram
        );
    }
    Ok(())
}
