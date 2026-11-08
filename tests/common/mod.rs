//! Shared helpers for the mock-server tests. Response bodies are taken from
//! the examples in `openapi.yaml` so the models stay in sync with the spec.
#![allow(dead_code, unreachable_pub)]

use discloud_rs::Discloud;
use serde_json::Value;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

pub const TOKEN: &str = "test-token";

pub async fn setup() -> (MockServer, Discloud) {
    let server = MockServer::start().await;
    let client = Discloud::builder(TOKEN)
        .base_url(server.uri())
        .build()
        .expect("client");
    (server, client)
}

/// Mounts a mock that must be hit exactly once with the token header.
pub async fn mock(server: &MockServer, verb: &str, route: &str, status: u16, body: Value) {
    Mock::given(method(verb))
        .and(path(route))
        .and(header("api-token", TOKEN))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .expect(1)
        .mount(server)
        .await;
}
