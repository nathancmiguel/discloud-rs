//! Uploads a new application, or commits new code to an existing one.
//!
//! `cargo run --example upload -- path/to/app.zip [APP_ID]`

use std::error::Error;

use discloud_rs::Discloud;

fn client() -> Result<Discloud, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    Ok(Discloud::new(std::env::var("DISCLOUD_TOKEN")?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let zip = args.next().ok_or("pass the path of a .zip archive")?;
    let client = client()?;

    match args.next() {
        Some(id) => {
            let commit = client.commit_app(&id, &zip).await?;
            println!("{}", commit.message);
            if let Some(logs) = commit.logs {
                println!("Startup logs:\n{logs}");
            }
        }
        None => {
            let app = client.upload_app(&zip).await?;
            println!("Uploaded {} ({}) with {} MB", app.name, app.id, app.ram);
        }
    }
    Ok(())
}
