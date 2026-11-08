//! Lists your subdomains and custom domains.
//!
//! `cargo run --example subdomains`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = client()?;
    for sub in client.get_subdomains().await? {
        let state = if sub.in_use() { "in use" } else { "available" };
        println!("{}.discloud.app ({state})", sub.id);
    }
    for domain in client.get_custom_domains().await? {
        let status = domain.dns_status.as_deref().unwrap_or("unknown");
        println!("{} ({status})", domain.id);
    }
    Ok(())
}
