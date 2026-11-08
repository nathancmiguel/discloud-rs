//! Lists the snapshot versions of an application and prints a download
//! link for the latest one.
//!
//! `cargo run --example snapshots -- APP_ID`

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
    let client = client()?;
    let id = app_id_arg()?;
    let versions = client.get_app_snapshots(&id).await?;
    for v in &versions {
        println!("{} ({} bytes)", v.version, v.size.unwrap_or_default());
    }
    if let Some(latest) = versions.iter().map(|v| v.version.as_str()).max() {
        let link = client.get_snapshot_download(&id, latest).await?;
        println!("Latest: {} (expires {})", link.url, link.expires_at);
    }
    Ok(())
}
