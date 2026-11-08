![discloud and ferris](https://github.com/jackskelt/discloud-rs/assets/86922268/42249ccb-4e20-4c60-a463-7dfd06466c32)

# discloud-rs

[![crates.io](https://img.shields.io/crates/v/discloud-rs.svg)](https://crates.io/crates/discloud-rs)
[![docs.rs](https://docs.rs/discloud-rs/badge.svg)](https://docs.rs/discloud-rs)
[![CI](https://github.com/jackskelt/discloud-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jackskelt/discloud-rs/actions/workflows/ci.yml)

An async Rust client for the [Discloud](https://discloud.com) hosting API (v2).

- Covers every route of the API specification in [`openapi.yaml`](openapi.yaml):
  applications, moderators, uploads, file explorer, domains, snapshots,
  environment variables, projects, databases and teams.
- Typed models and errors that carry the API's message and error code.
- One pooled HTTP client; cheap to clone and share between tasks.
- Small footprint: `reqwest`, `serde`, `serde_json`, `thiserror` and
  `tracing`. No `unsafe` code.

## Installation

```toml
[dependencies]
discloud-rs = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The client works on any Tokio runtime.

**Minimum supported Rust version:** 1.85.

### Cargo features

| Feature | Default | Description |
| --- | --- | --- |
| `rustls` | yes | TLS through rustls, with the platform certificate verifier. |
| `native-tls` | no | TLS through the OS stack (OpenSSL on Linux, so it needs `pkg-config` and `libssl-dev`). |

To use native TLS only: `discloud-rs = { version = "0.4", default-features = false, features = ["native-tls"] }`.

## Quick start

Get an API token from the Discloud dashboard. Then:

```rust,no_run
use discloud_rs::{AppStartError, Discloud};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Discloud::new(std::env::var("DISCLOUD_TOKEN")?);

    let user = client.get_user_info().await?;
    println!("{} MB of {} MB used", user.used_ram, user.total_ram);

    for app in client.get_all_apps().await? {
        println!("{} ({}) online={}", app.name, app.id, app.online);
    }

    match client.start_app("APP_ID").await {
        Ok(status) => println!("started, online={}", status.online),
        Err(AppStartError::AlreadyOnline) => println!("already online"),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
```

Most models also have convenience methods, for example `app.restart(&client)`
or `member.add_perms(&client, perms)`.

### Configuration

```rust,no_run
use std::time::Duration;

# fn main() -> Result<(), discloud_rs::Error> {
let client = discloud_rs::Discloud::builder("TOKEN")
    .timeout(Duration::from_secs(30))      // total per request (none by default)
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(100)) // default; uploads and commits answer slowly
    .user_agent("my-bot/1.0")
    .build()?;
# Ok(()) }
```

`.base_url(...)` points the client at another server, such as a proxy or a
mock server in tests. `.http_client(...)` reuses an existing `reqwest::Client`.

## Error handling

Every method returns `Result<T, discloud_rs::Error>`. Start, stop and RAM
changes return a small wrapper with the expected outcomes instead:
`AppStartError::AlreadyOnline`, `AppStopError::AlreadyOffline` and
`AppRamError::ForbiddenQuantity`.

| Variant | When |
| --- | --- |
| `Unauthorized`, `Forbidden`, `NotFound`, `Conflict`, `BadRequest`, `RateLimited`, `Server` | The API answered with HTTP 401, 403, 404, 409, 400, 429 or 5xx. |
| `Api` | Any other API failure, including HTTP 200 responses with `"status": "error"` (the API reports some failures this way). |
| `Http` | Connection, TLS or timeout failure. |
| `Decode` | The response did not match the expected model. |
| `Io` | A local file could not be read (uploads, commits). |
| `InvalidInput` | The arguments were rejected before sending anything. Examples: `"all"` passed to a single-app method, an invalid subdomain name, or RAM below 100 MB. |

API variants carry an `ApiError` with `http_status`, `message`, `code` (for
example `SNAPSHOT:APP_BUSY` or `CLUSTER_MISMATCH`) and `status_code`:

```rust,no_run
# async fn run(client: discloud_rs::Discloud) {
use discloud_rs::Error;

match client.create_snapshot("APP_ID").await {
    Ok(snapshot) => println!("snapshot {} ready at {}", snapshot.version, snapshot.url),
    Err(Error::Conflict(e)) if e.code.as_deref() == Some("SNAPSHOT:APP_BUSY") => {
        println!("try again later: {}", e.message)
    }
    Err(e) => eprintln!("{e}"),
}
# }
```

## Supported routes

Every route is implemented and covered by mock-server tests built from the
examples in `openapi.yaml`. The tests check the request method, path, headers
and body, and the response decoding. They were **not** run against the live
API; only the read-only smoke tests in `tests/live.rs` do that, and only when
you run them with a token.

| Area | Methods |
| --- | --- |
| User | `get_user_info`, `get_vscode_user`, `set_locale` |
| Apps: info | `get_app`, `get_all_apps`, `get_app_status`, `get_all_apps_status`, `get_app_logs`, `get_all_apps_logs`, `get_app_backup`, `get_all_apps_backup` |
| Apps: manage | `start_app`, `start_all_apps`, `stop_app`, `stop_all_apps`, `restart_app`, `restart_all_apps`, `set_app_ram`, `commit_app`, `commit_app_bytes`, `delete_app`, `delete_all_apps` |
| Apps: extras | `update_app_profile`, `install_apt`, `uninstall_apt`, `run_console_command`, `exec_command` |
| Upload | `upload_app`, `upload_app_to_project`, `upload_app_bytes` |
| File explorer | `list_files`, `read_file`, `create_file`, `edit_file` |
| App moderators | `get_app_team`, `add_app_mod`, `edit_app_mod`, `remove_app_mod` |
| Moderated apps (`/team`) | `get_team_apps`, plus `*_team_app` / `*_all_team_apps` variants of status, logs, backup, start, stop, restart, RAM and commit |
| Subdomains | `get_subdomains`, `get_subdomain`, `create_subdomain`, `delete_subdomain` |
| Custom domains | `get_custom_domains`, `get_custom_domain`, `create_custom_domain`, `edit_custom_domain`, `remove_custom_domain`, `verify_custom_domain` |
| Snapshots | `get_snapshots`, `get_app_snapshots`, `create_snapshot`, `get_snapshot_download`, `delete_snapshot` |
| Environment variables | `get_/set_/delete_` + `account_env`, `project_env`, `app_env` |
| Projects | `create_project`, `get_projects`, `get_project`, `update_project`, `delete_project`, `set_project_team`, `get_project_available_apps`, `add_app_to_project`, `remove_app_from_project` |
| Databases | `get_database_presets`, `get_random_database_names`, `get_databases`, `create_database`, `get_database`, `update_database`, `delete_database`, `start_/stop_/restart_database`, `backup_database`, `get_database_backups`, `get_database_backup_download`, `restore_database`, `get_database_logs`, `delete_database_logs` |
| Teams | `create_team`, `get_teams`, `get_teams_apps`, `get_team_role_permissions`, `get_team`, `update_team`, `delete_team`, `add_team_member`, `get_team_members`, `update_team_member`, `remove_team_member`, `leave_team` |

### Known API quirks and limitations

- Some failures arrive as HTTP 200 with `"status": "error"`. They are reported
  as `Error::Api` with `http_status == 200`.
- Wire typos are mapped to correctly named fields: `stoped` becomes
  `AppStopAll::stopped`, and `removealled` becomes `AppDeleteAll::removed`.
- A few payloads are undocumented, so they are exposed as raw
  `serde_json::Value`: `SnapshotList::backups`, `ProjectDetails::team` and
  `Database::perms`.
- `get_all_team_apps_status` calls `/team/all/status`, which the
  specification only documents for single IDs.
- Team roles and database engines are plain strings, so new values on the
  API side don't break deserialization.

## Examples

Runnable examples live in [`examples/`](examples). Put the token in
`DISCLOUD_TOKEN` (or in a `.env` file; see `.env.example`) and run:

```bash
cargo run --example client
cargo run --example apps_info
cargo run --example apps_manage_restart -- APP_ID
cargo run --example upload -- path/to/app.zip
cargo run --example apps_manage_delete -- APP_ID --yes   # destructive
```

## Design notes

- `Discloud` wraps an `Arc` around one `reqwest::Client`, the token and the base URL.
- Each resource module (`src/apps.rs`, `src/databases.rs`, ...) holds its
  models and its own `impl Discloud` block.
- `/app/...` and `/team/...` routes share one implementation; a private
  `Scope` selects the path prefix.
- A single request pipeline (`src/client.rs`) builds percent-encoded URLs,
  maps errors and decodes bodies.
- Response models default missing fields, so API additions or omissions
  don't break deserialization. They are `#[non_exhaustive]`, so new fields are
  not breaking changes.

## Development

With Nix (flakes): `nix develop`, or `direnv allow` (the repository ships an
`.envrc`). The dev shell provides the toolchain from `rust-toolchain.toml`,
rust-analyzer, cargo-deny, pkg-config and OpenSSL.

Without Nix, install Rust with [rustup](https://rustup.rs); `rust-toolchain.toml`
selects stable with clippy and rustfmt.

Run the same checks as CI:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo deny check
npx @redocly/cli@2.69.0 lint --config .redocly.yaml openapi.yaml
```

Live smoke tests (read-only, need a token and network access):

```bash
DISCLOUD_TOKEN=... cargo test --test live -- --ignored
```

## Contributing

Issues and pull requests are welcome. When adding or changing a route, keep
three things in sync:

1. the operation in `openapi.yaml`,
2. the method and models in `src/`,
3. a mock-server test in `tests/` that uses the spec example.

CI must pass (formatting, clippy with `-D warnings`, tests, docs, MSRV,
cargo-deny and the OpenAPI lint).

## License

Licensed under the [Apache License 2.0](LICENSE).
