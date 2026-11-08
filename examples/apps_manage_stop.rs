//! Stops an application.
//!
//! `cargo run --example apps_manage_stop -- APP_ID`

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
    use discloud_rs::AppStopError;

    match client()?.stop_app(&app_id_arg()?).await {
        Ok(()) => println!("Stopped"),
        Err(AppStopError::AlreadyOffline) => println!("The application is already offline"),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
