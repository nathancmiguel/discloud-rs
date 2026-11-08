# Changelog

All notable changes to this project are documented here. The project follows
[Semantic Versioning](https://semver.org/); before 1.0, minor versions may
contain breaking changes.

## [0.4.0] - Unreleased

### Added

- Every operation in `openapi.yaml` (97 routes) is now implemented. New groups:
  - application profile, APT packages, console and exec
  - file explorer
  - VS Code user
  - subdomains and custom domains
  - snapshots
  - environment variables
  - projects
  - databases
  - teams
- Upload and commit routes: `upload_app`, `upload_app_to_project`,
  `upload_app_bytes`, `commit_app_bytes` and `commit_team_app_bytes`.
- `Discloud::builder()`, which can set the base URL, timeouts, the user agent
  or an existing `reqwest::Client`.
- `ApiError`, which carries the HTTP status, the API message, the `code` and
  `statusCode` fields. `Error::api_error()` and `Error::http_status()` read it.
- Cargo features `rustls` (default) and `native-tls` select the TLS backend.
- `Locale` implements `FromStr`, `Serialize`, `Copy` and `Hash`.
- A mock-server test suite and a CI pipeline (fmt, clippy, tests on three
  operating systems, MSRV, docs, cargo-deny, OpenAPI lint and Nix).

### Fixed

- Every client now shares one connection pool. Previously each request built
  a new `reqwest::Client`.
- Error messages no longer leak memory (`String::leak` was called for every
  404 and 409 response).
- Multipart uploads now include the multipart boundary. A hard-coded
  `Content-Type` header used to remove it.
- Path parameters are percent-encoded, and `.`/`..` are rejected. IDs could
  previously change the requested route.
- Models no longer fail on fields the API omits or sends as `null`, such as
  `App::cluster_name`, `exitCode` and `mods`.
- `addedTimestamp` and `addedAtTimestamp` are both accepted.
- `AppStopAll::stopped` reads the API's `stoped` key.
- The token is marked sensitive and redacted from `Debug` output. Response
  bodies, which may hold secrets, are never logged.
- `Error` implements `std::error::Error`, and so do `AppStartError`,
  `AppStopError` and `AppRamError`.

### Changed (breaking)

| Before | After |
| --- | --- |
| `Error::InvalidToken`, `Ratelimited`, `ServerError`, `Unknown` | `Unauthorized(ApiError)`, `RateLimited(ApiError)`, `Server(ApiError)`, `Api(ApiError)` |
| `Error::NotFound(&'static str)`, `Conflict(&'static str)`, `Forbidden` | `NotFound(ApiError)`, `Conflict(ApiError)`, `Forbidden(ApiError)` (use `.message`) |
| `Error::InvalidRequest(&'static str)` | `Error::InvalidInput(String)` |
| `Error::Reqwest`, `Error::ReadFile` | `Error::Http`, `Error::Io` |
| `commit_app(...) -> Result<(), Error>` | `-> Result<AppCommit, Error>` (message and startup logs) |
| `App::commit(&client, id, path)` | `App::commit(&client, path)` (the unused `id` argument is gone) |
| `TeamApp::commit(path, &client)`, `TeamApp::set_ram(qty, &client)` | `commit(&client, path)`, `set_ram(&client, qty)` |
| `AppDeleteAll::removealled` | `AppDeleteAll::removed` |
| `App::ram: i32`, `App::avatar_url: String`, `App::cluster_name: String`, `exit_code: i32` | `u32`, `Option<String>`, `Option<String>`, `Option<i32>` |
| `Discloud::get_team_mods` (panicked with `todo!()`) | removed |

- `Error` and the response models are `#[non_exhaustive]`. New fields and
  variants are no longer breaking, but code that matches on them needs a
  wildcard arm.
- `TeamPerms` has an `Unknown` variant for permissions this version does not
  know about.

### Deprecated

- `TeamManager` and `APITeamMember`: call the methods on `Discloud` or `App`
  and use `TeamMember` instead.
- `get_team_all_apps_backup`: renamed to `get_all_team_apps_backup`.

### Migration

```rust
// 0.3
match client.get_app("id").await {
    Err(Error::NotFound(msg)) => eprintln!("{msg}"),
    Err(Error::InvalidToken) => eprintln!("bad token"),
    _ => {}
}

// 0.4
match client.get_app("id").await {
    Err(Error::NotFound(e)) => eprintln!("{}", e.message),
    Err(Error::Unauthorized(_)) => eprintln!("bad token"),
    _ => {}
}
```

## [0.3.0-alpha]

- Commit routes and the first team-app routes.
