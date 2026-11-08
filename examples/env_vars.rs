//! Lists the environment variable names of an application.
//! Values are secrets, so they are not printed.
//!
//! `cargo run --example env_vars -- APP_ID`

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
    let vars = client()?.get_app_env(&app_id_arg()?).await?;
    for name in vars.keys() {
        println!("{name}");
    }
    Ok(())
}
