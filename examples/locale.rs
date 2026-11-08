//! Changes the language of the API messages.
//!
//! `cargo run --example locale -- en-US`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let locale: discloud_rs::Locale = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "en-US".into())
        .parse()?;
    client()?.set_locale(locale).await?;
    println!("Locale set to {locale}");
    Ok(())
}
