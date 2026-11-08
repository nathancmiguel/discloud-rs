//! Read-only smoke tests against the real API.
//!
//! Ignored by default. Run with a token:
//! `DISCLOUD_TOKEN=... cargo test --test live -- --ignored`

use discloud_rs::Discloud;

fn client() -> Discloud {
    dotenvy::dotenv().ok();
    let token = std::env::var("DISCLOUD_TOKEN").expect("set DISCLOUD_TOKEN to run live tests");
    Discloud::new(token)
}

#[tokio::test]
#[ignore = "requires DISCLOUD_TOKEN and network access"]
async fn live_user_info() {
    let user = client().get_user_info().await.expect("user info");
    assert!(!user.user_id.is_empty());
}

#[tokio::test]
#[ignore = "requires DISCLOUD_TOKEN and network access"]
async fn live_list_apps() {
    let client = client();
    for app in client.get_all_apps().await.expect("apps") {
        assert!(!app.id.is_empty());
    }
}
