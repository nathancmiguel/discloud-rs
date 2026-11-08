//! Grants the restart permission to every moderator of an application.
//!
//! `cargo run --example apps_team_manage_edit -- APP_ID`

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
    use discloud_rs::TeamPerms;

    let client = client()?;
    for mut member in client.get_app_team(&app_id_arg()?).await? {
        member
            .add_perms_mut(&client, vec![TeamPerms::Restart])
            .await?;
        println!("{} now has {:?}", member.mod_id, member.perms);
    }
    Ok(())
}
