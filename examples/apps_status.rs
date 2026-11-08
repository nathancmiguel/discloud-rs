//! Shows the resource usage of an application.
//!
//! `cargo run --example apps_status -- APP_ID`

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
    let status = client()?.get_app_status(&app_id_arg()?).await?;
    println!(
        "{}: CPU {}, memory {}, disk {}, network down {} / up {}, last restart {}",
        status.container,
        status.cpu,
        status.memory,
        status.ssd,
        status.net_io.down,
        status.net_io.up,
        status.last_restart
    );
    Ok(())
}
