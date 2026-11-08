//! Error mapping and transport behavior.

mod common;

use common::{mock, setup};
use discloud_rs::{AppRamError, AppStartError, AppStopError, Discloud, Error};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

type Check = fn(&Error) -> bool;

#[tokio::test]
async fn status_codes_map_to_variants() {
    let cases: &[(u16, Check)] = &[
        (400, |e| matches!(e, Error::BadRequest(_))),
        (401, |e| matches!(e, Error::Unauthorized(_))),
        (403, |e| matches!(e, Error::Forbidden(_))),
        (404, |e| matches!(e, Error::NotFound(_))),
        (409, |e| matches!(e, Error::Conflict(_))),
        (429, |e| matches!(e, Error::RateLimited(_))),
        (500, |e| matches!(e, Error::Server(_))),
        (418, |e| matches!(e, Error::Api(_))),
    ];
    for (status, check) in cases {
        let (server, client) = setup().await;
        mock(
            &server,
            "GET",
            "/user",
            *status,
            json!({"status": "error", "message": "boom"}),
        )
        .await;
        let err = client.get_user_info().await.unwrap_err();
        assert!(check(&err), "HTTP {status} mapped to {err:?}");
        let api = err.api_error().expect("api error");
        assert_eq!(api.http_status, *status);
        assert_eq!(api.message, "boom");
        assert_eq!(err.http_status(), Some(*status));
    }
}

#[tokio::test]
async fn string_and_numeric_codes_are_kept() {
    let (server, client) = setup().await;
    mock(
        &server,
        "POST",
        "/snapshot/123",
        409,
        json!({"code": "SNAPSHOT:APP_BUSY", "message": "Another snapshot is running"}),
    )
    .await;
    let err = client.create_snapshot("123").await.unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.code.as_deref(), Some("SNAPSHOT:APP_BUSY"));
    assert!(err.to_string().contains("Another snapshot is running"));

    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/user",
        401,
        json!({"code": 401, "message": "Block"}),
    )
    .await;
    let err = client.get_user_info().await.unwrap_err();
    assert_eq!(err.api_error().unwrap().code.as_deref(), Some("401"));
}

#[tokio::test]
async fn missing_message_falls_back_to_reason() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>bad gateway</html>"))
        .mount(&server)
        .await;
    let err = client.get_user_info().await.unwrap_err();
    assert!(matches!(&err, Error::Server(e) if e.message == "Bad Gateway"));
}

#[tokio::test]
async fn in_body_errors_with_http_200() {
    let (server, client) = setup().await;
    mock(
        &server,
        "POST",
        "/upload",
        200,
        json!({"status": "error", "statusCode": 93, "message": "The discloud.config file was not found within .zip sent."}),
    )
    .await;
    let err = client
        .upload_app_bytes(b"zip".to_vec(), "app.zip", None)
        .await
        .unwrap_err();
    let api = err.api_error().unwrap();
    assert!(matches!(err, Error::Api(_)));
    assert_eq!((api.http_status, api.status_code), (200, Some(93)));
}

#[tokio::test]
async fn start_stop_ram_specific_errors() {
    let (server, client) = setup().await;
    mock(
        &server,
        "PUT",
        "/app/1/start",
        200,
        json!({"status": "error", "message": "Your application is already online!", "appStatus": {"online": true, "ramKilled": false, "exitCode": 0}}),
    )
    .await;
    mock(
        &server,
        "PUT",
        "/team/1/stop",
        200,
        json!({"status": "error", "message": "already offline"}),
    )
    .await;
    mock(
        &server,
        "PUT",
        "/app/1/ram",
        200,
        json!({"status": "error", "message": "You do not have enough RAM available."}),
    )
    .await;

    assert!(matches!(
        client.start_app("1").await,
        Err(AppStartError::AlreadyOnline)
    ));
    assert!(matches!(
        client.stop_team_app("1").await,
        Err(AppStopError::AlreadyOffline)
    ));
    assert!(
        matches!(client.set_app_ram("1", 200).await, Err(AppRamError::ForbiddenQuantity(m)) if m.contains("enough RAM"))
    );
}

#[tokio::test]
async fn http_errors_stay_generic_for_start() {
    let (server, client) = setup().await;
    mock(
        &server,
        "PUT",
        "/app/1/start",
        404,
        json!({"status": "error", "message": "App not found"}),
    )
    .await;
    assert!(matches!(
        client.start_app("1").await,
        Err(AppStartError::Other(Error::NotFound(_)))
    ));
}

#[tokio::test]
async fn invalid_input_is_rejected_locally() {
    // No mock server: these must fail before any request is sent.
    let client = Discloud::builder("t")
        .base_url("http://127.0.0.1:9")
        .build()
        .unwrap();
    assert!(matches!(
        client.get_app("all").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.get_app("..").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.commit_app_bytes("all", vec![], "a.zip").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.set_app_ram("1", 50).await,
        Err(AppRamError::ForbiddenQuantity(_))
    ));
    assert!(matches!(
        client.create_subdomain("No").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.get_snapshot_download("1", "bad").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.install_apt("1", &[]).await,
        Err(Error::InvalidInput(_))
    ));
}

#[tokio::test]
async fn path_parameters_are_percent_encoded() {
    let (server, client) = setup().await;
    mock(
        &server,
        "DELETE",
        "/app/1/team/a%2Fb",
        200,
        json!({"status": "ok", "message": "removed"}),
    )
    .await;
    client.remove_app_mod("1", "a/b").await.unwrap();
}

#[tokio::test]
async fn decode_errors_are_reported() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/user",
        200,
        json!({"status": "ok", "user": "not an object"}),
    )
    .await;
    assert!(matches!(
        client.get_user_info().await,
        Err(Error::Decode(_))
    ));
}

#[tokio::test]
async fn connection_failures_are_http_errors() {
    let client = Discloud::builder("t")
        .base_url("http://127.0.0.1:9")
        .build()
        .unwrap();
    assert!(matches!(client.get_user_info().await, Err(Error::Http(_))));
}
