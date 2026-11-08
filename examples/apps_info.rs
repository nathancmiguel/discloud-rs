//! Lists your applications, or shows one when an ID is given.
//!
//! `cargo run --example apps_info [-- APP_ID]`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = client()?;
    match std::env::args().nth(1) {
        Some(id) => println!("{:#?}", client.get_app(&id).await?),
        None => {
            for app in client.get_all_apps().await? {
                let state = if app.online { "online" } else { "offline" };
                println!("{:<16} {:<30} {state:<8} {} MB", app.id, app.name, app.ram);
            }
        }
    }
    Ok(())
}
