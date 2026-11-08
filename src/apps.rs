//! Applications: information, status, logs, backups and lifecycle management.
//!
//! Every operation exists for your own applications (`/app/...`) and for
//! applications you moderate (`/team/...`); both share the implementation
//! below through [`Scope`].

use std::{fmt, path::Path};

use reqwest::{
    Method,
    multipart::{Form, Part},
};
use serde::{Deserialize, Serialize};

use crate::{
    Discloud, Error, TeamMember, TeamPerms,
    client::{Scope, app_id, single_id},
    error::{ApiError, AppRamError, AppStartError, AppStopError},
    wire::{MessageResponse, null_as_default},
};

/// Application type: `0` is a bot, `1` is a website.
pub type AppType = i32;

/// Last commit of a Git-synchronized application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct LastCommit {
    /// Commit message.
    pub message: String,
    /// Commit hash.
    pub sha: String,
}

/// Git synchronization settings of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct SyncGit {
    /// Whether pushes are deployed automatically.
    pub auto_deploy: bool,
    /// GitHub repository ID.
    #[serde(rename = "repositoryID")]
    pub repository_id: u64,
    /// Synchronized branch.
    pub branch_name: String,
    /// Last deployed commit.
    pub last_commit: Option<LastCommit>,
}

/// An application you own.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct App {
    /// Application ID.
    pub id: String,
    /// Application name.
    pub name: String,
    /// Avatar URL.
    #[serde(rename = "avatarURL")]
    pub avatar_url: Option<String>,
    /// [`AppType`]: `0` = bot, `1` = site.
    pub r#type: AppType,
    /// Whether the application is online.
    pub online: bool,
    /// Whether the application was killed for exceeding its RAM.
    pub ram_killed: bool,
    /// Exit code of the last run, if it stopped.
    pub exit_code: Option<i32>,
    /// Allocated RAM in MB.
    pub ram: u32,
    /// Entry point file.
    pub main_file: String,
    /// Programming language.
    pub lang: String,
    /// Git auto-deploy setting (for example `"no"`).
    pub auto_deploy_git: Option<String>,
    /// Whether the application restarts automatically after a crash.
    pub auto_restart: bool,
    /// IDs of the application's moderators.
    #[serde(deserialize_with = "null_as_default")]
    pub mods: Vec<String>,
    /// Installed APT packages.
    #[serde(deserialize_with = "null_as_default")]
    pub apts: Vec<String>,
    /// Upload time in milliseconds since the Unix epoch.
    #[serde(alias = "addedTimestamp")]
    pub added_at_timestamp: u64,
    /// Cluster hosting the application.
    pub cluster_name: Option<String>,
    /// Git synchronization settings, if the application is linked to a repository.
    pub sync_git: Option<SyncGit>,
}

/// Network usage of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct NetIO {
    /// Downloaded data (human readable, e.g. `"5MB"`).
    pub down: String,
    /// Uploaded data (human readable).
    pub up: String,
}

/// Resource usage of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppStatus {
    /// Application ID.
    pub id: String,
    /// Container state (e.g. `"Online"`).
    pub container: String,
    /// CPU usage (e.g. `"0.00%"`).
    pub cpu: String,
    /// Memory usage and limit (e.g. `"39.9MB/100MB"`).
    pub memory: String,
    /// Disk usage.
    pub ssd: String,
    /// Network usage.
    #[serde(rename = "netIO")]
    pub net_io: NetIO,
    /// Time since the last restart (e.g. `"30 minutes"`).
    pub last_restart: String,
    /// Start time (RFC 3339).
    #[serde(rename = "startedAt")]
    pub started_at: String,
}

/// Terminal output of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppLogsTerminal {
    /// Complete logs.
    pub big: Option<String>,
    /// Condensed logs.
    pub small: Option<String>,
    /// URL to view the logs online.
    pub url: String,
}

/// Logs of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppLogs {
    /// Application ID.
    pub id: String,
    /// Terminal output.
    pub terminal: AppLogsTerminal,
}

/// Backup download link of an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppBackup {
    /// Application ID.
    pub id: String,
    /// Download URL of the backup archive.
    pub url: String,
    /// Backup status (only present for bulk requests).
    pub status: Option<String>,
}

/// State of an application after a start request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppStartStatus {
    /// Exit code of the process, if it exited.
    pub exit_code: Option<i32>,
    /// Whether the application is online.
    pub online: bool,
    /// Whether the application was killed for exceeding its RAM.
    pub ram_killed: bool,
}

/// Result of starting every application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppStartAll {
    /// Applications that were started.
    pub started: Vec<String>,
    /// Applications that were already online.
    pub already_online: Vec<String>,
    /// Applications with another operation in progress.
    pub already_in_process: Vec<String>,
}

/// Result of stopping every application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppStopAll {
    /// Applications that were stopped (sent as `stoped` by the API).
    #[serde(rename = "stoped", alias = "stopped")]
    pub stopped: Vec<String>,
    /// Applications that were already offline.
    pub already_offline: Vec<String>,
    /// Applications with another operation in progress.
    pub already_in_process: Vec<String>,
}

/// Result of restarting every application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppRestartAll {
    /// Applications that were restarted.
    pub restarted: Vec<String>,
    /// Applications that were offline.
    pub already_offline: Vec<String>,
    /// Applications with another operation in progress.
    pub already_in_process: Vec<String>,
}

/// Result of deleting every application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppDeleteAll {
    /// Applications that were removed (sent as `removealled` by the API).
    #[serde(rename = "removealled", alias = "removed")]
    pub removed: Vec<String>,
    /// Applications that were offline.
    pub already_offline: Vec<String>,
    /// Applications with another operation in progress.
    pub already_in_process: Vec<String>,
}

/// Result of a commit (code update).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppCommit {
    /// Message returned by the API.
    pub message: String,
    /// Application logs, present when the application failed to start.
    pub logs: Option<String>,
}

/// Fields to change with [`Discloud::update_app_profile`]. Unset fields are
/// left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AppProfileUpdate {
    /// New name (at most 30 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New avatar URL.
    #[serde(rename = "avatarURL", skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// APT packages that can be installed on an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AptPackage {
    /// `canvas`
    Canvas,
    /// `ffmpeg`
    Ffmpeg,
    /// `java`
    Java,
    /// `libgl`
    Libgl,
    /// `openssl`
    Openssl,
    /// `puppeteer`
    Puppeteer,
    /// `tools`
    Tools,
}

impl AptPackage {
    /// The package name used by the API.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::Ffmpeg => "ffmpeg",
            Self::Java => "java",
            Self::Libgl => "libgl",
            Self::Openssl => "openssl",
            Self::Puppeteer => "puppeteer",
            Self::Tools => "tools",
        }
    }
}

impl fmt::Display for AptPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Output of a terminal command sent with [`Discloud::run_console_command`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ConsoleOutput {
    /// Whether the application is online.
    pub online: bool,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// Output of a command run with [`Discloud::exec_command`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExecOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code of the command.
    pub exit_code: Option<i32>,
}

#[derive(Deserialize)]
struct AppsResponse<T> {
    apps: T,
}

#[derive(Deserialize)]
struct BackupsResponse<T> {
    backups: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartResponse {
    app_status: Option<AppStartStatus>,
}

#[derive(Serialize)]
struct RamBody {
    #[serde(rename = "ramMB")]
    ram: u32,
}

#[derive(Serialize)]
struct AptBody {
    apt: String,
}

#[derive(Serialize)]
struct ConsoleBody<'a> {
    command: &'a str,
}

#[derive(Deserialize)]
struct ConsoleApp {
    #[serde(default)]
    shell: ConsoleOutput,
}

#[derive(Serialize)]
struct ExecBody<'a> {
    cmd: &'a str,
}

#[derive(Deserialize)]
struct ExecResponse {
    #[serde(default)]
    exec: Option<ExecOutput>,
}

/// Minimum RAM accepted by the API, in MB (bots; sites need 512 MB).
pub(crate) const MIN_RAM_MB: u32 = 100;

/// Builds the multipart form for a zip archive on disk. The file is streamed,
/// not loaded into memory.
pub(crate) async fn file_form(path: &Path) -> Result<Form, Error> {
    let part = Part::file(path).await?.mime_str("application/zip")?;
    Ok(Form::new().part("file", part))
}

/// Builds the multipart form for an in-memory zip archive.
pub(crate) fn bytes_form(bytes: Vec<u8>, file_name: &str) -> Result<Form, Error> {
    let part = Part::bytes(bytes)
        .file_name(file_name.to_owned())
        .mime_str("application/zip")?;
    Ok(Form::new().part("file", part))
}

/// Operations shared by `/app/...` and `/team/...`.
impl Discloud {
    async fn scoped_get<T: serde::de::DeserializeOwned>(
        &self,
        scope: Scope,
        id: &str,
        action: &str,
    ) -> Result<T, Error> {
        self.get(&[scope.prefix(), id, action]).await
    }

    async fn status_one(&self, scope: Scope, id: &str, all: &str) -> Result<AppStatus, Error> {
        let id = single_id(id, all)?;
        let r: AppsResponse<_> = self.scoped_get(scope, id, "status").await?;
        Ok(r.apps)
    }

    async fn status_all(&self, scope: Scope) -> Result<Vec<AppStatus>, Error> {
        let r: AppsResponse<_> = self.scoped_get(scope, "all", "status").await?;
        Ok(r.apps)
    }

    async fn logs_one(&self, scope: Scope, id: &str, all: &str) -> Result<AppLogs, Error> {
        let id = single_id(id, all)?;
        let r: AppsResponse<_> = self.scoped_get(scope, id, "logs").await?;
        Ok(r.apps)
    }

    async fn logs_all(&self, scope: Scope) -> Result<Vec<AppLogs>, Error> {
        let r: AppsResponse<_> = self.scoped_get(scope, "all", "logs").await?;
        Ok(r.apps)
    }

    async fn backup_one(&self, scope: Scope, id: &str, all: &str) -> Result<AppBackup, Error> {
        let id = single_id(id, all)?;
        let r: BackupsResponse<_> = self.scoped_get(scope, id, "backup").await?;
        Ok(r.backups)
    }

    async fn backup_all(&self, scope: Scope) -> Result<Vec<AppBackup>, Error> {
        let r: BackupsResponse<_> = self.scoped_get(scope, "all", "backup").await?;
        Ok(r.backups)
    }

    async fn start_one(
        &self,
        scope: Scope,
        id: &str,
        all: &str,
    ) -> Result<AppStartStatus, AppStartError> {
        let id = single_id(id, all)?;
        match self
            .send::<StartResponse>(Method::PUT, &[scope.prefix(), id, "start"])
            .await
        {
            Ok(r) => r.app_status.ok_or_else(|| {
                AppStartError::Other(Error::Api(ApiError {
                    http_status: 200,
                    code: None,
                    status_code: None,
                    message: "the response did not include the application status".into(),
                }))
            }),
            Err(e) if e.is_in_body_error() => Err(AppStartError::AlreadyOnline),
            Err(e) => Err(e.into()),
        }
    }

    async fn stop_one(&self, scope: Scope, id: &str, all: &str) -> Result<(), AppStopError> {
        let id = single_id(id, all)?;
        match self
            .send::<MessageResponse>(Method::PUT, &[scope.prefix(), id, "stop"])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if e.is_in_body_error() => Err(AppStopError::AlreadyOffline),
            Err(e) => Err(e.into()),
        }
    }

    async fn restart_one(&self, scope: Scope, id: &str, all: &str) -> Result<(), Error> {
        let id = single_id(id, all)?;
        self.send::<MessageResponse>(Method::PUT, &[scope.prefix(), id, "restart"])
            .await
            .map(drop)
    }

    async fn bulk<T: serde::de::DeserializeOwned>(
        &self,
        scope: Scope,
        method: Method,
        action: &str,
    ) -> Result<T, Error> {
        let r: AppsResponse<T> = self.send(method, &[scope.prefix(), "all", action]).await?;
        Ok(r.apps)
    }

    async fn set_ram(&self, scope: Scope, id: &str, quantity: u32) -> Result<(), AppRamError> {
        let id = app_id(id)?;
        if quantity < MIN_RAM_MB {
            return Err(AppRamError::ForbiddenQuantity(format!(
                "at least {MIN_RAM_MB} MB is required, got {quantity} MB"
            )));
        }
        match self
            .send_json::<MessageResponse, _>(
                Method::PUT,
                &[scope.prefix(), id, "ram"],
                &RamBody { ram: quantity },
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(Error::Api(e)) if (200..300).contains(&e.http_status) => {
                Err(AppRamError::ForbiddenQuantity(e.message))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn commit_form(&self, scope: Scope, id: &str, form: Form) -> Result<AppCommit, Error> {
        let request = self
            .request(Method::PUT, &[scope.prefix(), id, "commit"], &[])?
            .multipart(form);
        self.execute(request).await
    }
}

/// Your applications (`/app/...`).
impl Discloud {
    /// Returns an application you own. `GET /app/{appID}`
    pub async fn get_app(&self, id: &str) -> Result<App, Error> {
        let id = single_id(id, "get_all_apps")?;
        let r: AppsResponse<_> = self.get(&["app", id]).await?;
        Ok(r.apps)
    }

    /// Returns all of your applications. `GET /app/all`
    pub async fn get_all_apps(&self) -> Result<Vec<App>, Error> {
        let r: AppsResponse<_> = self.get(&["app", "all"]).await?;
        Ok(r.apps)
    }

    /// Returns the resource usage of an application. `GET /app/{appID}/status`
    pub async fn get_app_status(&self, id: &str) -> Result<AppStatus, Error> {
        self.status_one(Scope::App, id, "get_all_apps_status").await
    }

    /// Returns the resource usage of all of your applications. `GET /app/all/status`
    pub async fn get_all_apps_status(&self) -> Result<Vec<AppStatus>, Error> {
        self.status_all(Scope::App).await
    }

    /// Returns the logs of an application. `GET /app/{appID}/logs`
    pub async fn get_app_logs(&self, id: &str) -> Result<AppLogs, Error> {
        self.logs_one(Scope::App, id, "get_all_apps_logs").await
    }

    /// Returns the logs of all of your applications. `GET /app/all/logs`
    pub async fn get_all_apps_logs(&self) -> Result<Vec<AppLogs>, Error> {
        self.logs_all(Scope::App).await
    }

    /// Returns a backup download link for an application. `GET /app/{appID}/backup`
    pub async fn get_app_backup(&self, id: &str) -> Result<AppBackup, Error> {
        self.backup_one(Scope::App, id, "get_all_apps_backup").await
    }

    /// Returns backup links for all of your applications (slow). `GET /app/all/backup`
    pub async fn get_all_apps_backup(&self) -> Result<Vec<AppBackup>, Error> {
        self.backup_all(Scope::App).await
    }

    /// Starts an application. `PUT /app/{appID}/start`
    pub async fn start_app(&self, id: &str) -> Result<AppStartStatus, AppStartError> {
        self.start_one(Scope::App, id, "start_all_apps").await
    }

    /// Starts all of your applications. `PUT /app/all/start`
    pub async fn start_all_apps(&self) -> Result<AppStartAll, Error> {
        self.bulk(Scope::App, Method::PUT, "start").await
    }

    /// Stops an application. `PUT /app/{appID}/stop`
    pub async fn stop_app(&self, id: &str) -> Result<(), AppStopError> {
        self.stop_one(Scope::App, id, "stop_all_apps").await
    }

    /// Stops all of your applications. `PUT /app/all/stop`
    pub async fn stop_all_apps(&self) -> Result<AppStopAll, Error> {
        self.bulk(Scope::App, Method::PUT, "stop").await
    }

    /// Restarts an application. `PUT /app/{appID}/restart`
    pub async fn restart_app(&self, id: &str) -> Result<(), Error> {
        self.restart_one(Scope::App, id, "restart_all_apps").await
    }

    /// Restarts all of your applications. `PUT /app/all/restart`
    pub async fn restart_all_apps(&self) -> Result<AppRestartAll, Error> {
        self.bulk(Scope::App, Method::PUT, "restart").await
    }

    /// Changes the RAM (in MB) allocated to an application. `PUT /app/{appID}/ram`
    pub async fn set_app_ram(&self, id: &str, quantity: u32) -> Result<(), AppRamError> {
        self.set_ram(Scope::App, id, quantity).await
    }

    /// Replaces the code of an application with a `.zip` archive on disk.
    /// `PUT /app/{appID}/commit`
    ///
    /// The API may take a long time to answer; the default read timeout is
    /// 100 seconds.
    pub async fn commit_app(&self, id: &str, path: impl AsRef<Path>) -> Result<AppCommit, Error> {
        let id = app_id(id)?;
        let form = file_form(path.as_ref()).await?;
        self.commit_form(Scope::App, id, form).await
    }

    /// Like [`commit_app`](Self::commit_app), with an in-memory `.zip` archive.
    pub async fn commit_app_bytes(
        &self,
        id: &str,
        zip: Vec<u8>,
        file_name: &str,
    ) -> Result<AppCommit, Error> {
        let id = app_id(id)?;
        self.commit_form(Scope::App, id, bytes_form(zip, file_name)?)
            .await
    }

    /// Deletes an application permanently. `DELETE /app/{appID}/delete`
    pub async fn delete_app(&self, id: &str) -> Result<(), Error> {
        let id = single_id(id, "delete_all_apps")?;
        self.send::<MessageResponse>(Method::DELETE, &["app", id, "delete"])
            .await
            .map(drop)
    }

    /// Deletes all of your applications permanently. `DELETE /app/all/delete`
    pub async fn delete_all_apps(&self) -> Result<AppDeleteAll, Error> {
        self.bulk(Scope::App, Method::DELETE, "delete").await
    }

    /// Changes the name and/or avatar of an application. `PUT /app/{appID}/profile`
    pub async fn update_app_profile(
        &self,
        id: &str,
        update: &AppProfileUpdate,
    ) -> Result<(), Error> {
        let id = app_id(id)?;
        if update.name.is_none() && update.avatar_url.is_none() {
            return Err(Error::invalid("set at least one of `name` or `avatar_url`"));
        }
        if let Some(name) = &update.name {
            let len = name.chars().count();
            if len == 0 || len > 30 {
                return Err(Error::invalid(
                    "the application name must have between 1 and 30 characters",
                ));
            }
        }
        self.send_json::<MessageResponse, _>(Method::PUT, &["app", id, "profile"], update)
            .await
            .map(drop)
    }

    /// Installs APT packages on an application. `PUT /app/{appID}/apt`
    ///
    /// Returns the API message.
    pub async fn install_apt(&self, id: &str, packages: &[AptPackage]) -> Result<String, Error> {
        self.apt(Method::PUT, id, packages).await
    }

    /// Uninstalls APT packages from an application. `DELETE /app/{appID}/apt`
    ///
    /// Returns the API message.
    pub async fn uninstall_apt(&self, id: &str, packages: &[AptPackage]) -> Result<String, Error> {
        self.apt(Method::DELETE, id, packages).await
    }

    async fn apt(
        &self,
        method: Method,
        id: &str,
        packages: &[AptPackage],
    ) -> Result<String, Error> {
        let id = app_id(id)?;
        if packages.is_empty() {
            return Err(Error::invalid("at least one APT package is required"));
        }
        let apt = packages
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let r: MessageResponse = self
            .send_json(method, &["app", id, "apt"], &AptBody { apt })
            .await?;
        Ok(r.message)
    }

    /// Sends a command to the application's terminal. `PUT /app/{appID}/console`
    pub async fn run_console_command(
        &self,
        id: &str,
        command: &str,
    ) -> Result<ConsoleOutput, Error> {
        let id = app_id(id)?;
        if command.trim().is_empty() {
            return Err(Error::invalid("the command must not be empty"));
        }
        let r: AppsResponse<ConsoleApp> = self
            .send_json(
                Method::PUT,
                &["app", id, "console"],
                &ConsoleBody { command },
            )
            .await?;
        Ok(r.apps.shell)
    }

    /// Executes a command inside the application's container (owner only).
    /// `PUT /app/{appID}/exec`
    ///
    /// Returns `None` when the API returns no output.
    pub async fn exec_command(&self, id: &str, cmd: &str) -> Result<Option<ExecOutput>, Error> {
        let id = app_id(id)?;
        if cmd.trim().is_empty() {
            return Err(Error::invalid("the command must not be empty"));
        }
        let r: ExecResponse = self
            .send_json(Method::PUT, &["app", id, "exec"], &ExecBody { cmd })
            .await?;
        Ok(r.exec)
    }
}

/// Applications you moderate (`/team/...`).
impl Discloud {
    /// Returns the resource usage of a moderated application. `GET /team/{appID}/status`
    pub async fn get_team_app_status(&self, id: &str) -> Result<AppStatus, Error> {
        self.status_one(Scope::Team, id, "get_all_team_apps_status")
            .await
    }

    /// Returns the resource usage of all moderated applications. `GET /team/all/status`
    ///
    /// The API documentation only describes this route for single IDs.
    pub async fn get_all_team_apps_status(&self) -> Result<Vec<AppStatus>, Error> {
        self.status_all(Scope::Team).await
    }

    /// Returns the logs of a moderated application. `GET /team/{appID}/logs`
    pub async fn get_team_app_logs(&self, id: &str) -> Result<AppLogs, Error> {
        self.logs_one(Scope::Team, id, "get_all_team_apps_logs")
            .await
    }

    /// Returns the logs of all moderated applications. `GET /team/all/logs`
    pub async fn get_all_team_apps_logs(&self) -> Result<Vec<AppLogs>, Error> {
        self.logs_all(Scope::Team).await
    }

    /// Returns a backup link for a moderated application. `GET /team/{appID}/backup`
    pub async fn get_team_app_backup(&self, id: &str) -> Result<AppBackup, Error> {
        self.backup_one(Scope::Team, id, "get_all_team_apps_backup")
            .await
    }

    /// Returns backup links for all moderated applications. `GET /team/all/backup`
    pub async fn get_all_team_apps_backup(&self) -> Result<Vec<AppBackup>, Error> {
        self.backup_all(Scope::Team).await
    }

    /// Deprecated alias of [`get_all_team_apps_backup`](Self::get_all_team_apps_backup).
    #[deprecated(since = "0.4.0", note = "use `get_all_team_apps_backup`")]
    pub async fn get_team_all_apps_backup(&self) -> Result<Vec<AppBackup>, Error> {
        self.get_all_team_apps_backup().await
    }

    /// Starts a moderated application. `PUT /team/{appID}/start`
    pub async fn start_team_app(&self, id: &str) -> Result<AppStartStatus, AppStartError> {
        self.start_one(Scope::Team, id, "start_all_team_apps").await
    }

    /// Starts all moderated applications. `PUT /team/all/start`
    pub async fn start_all_team_apps(&self) -> Result<AppStartAll, Error> {
        self.bulk(Scope::Team, Method::PUT, "start").await
    }

    /// Stops a moderated application. `PUT /team/{appID}/stop`
    pub async fn stop_team_app(&self, id: &str) -> Result<(), AppStopError> {
        self.stop_one(Scope::Team, id, "stop_all_team_apps").await
    }

    /// Stops all moderated applications. `PUT /team/all/stop`
    pub async fn stop_all_team_apps(&self) -> Result<AppStopAll, Error> {
        self.bulk(Scope::Team, Method::PUT, "stop").await
    }

    /// Restarts a moderated application. `PUT /team/{appID}/restart`
    pub async fn restart_team_app(&self, id: &str) -> Result<(), Error> {
        self.restart_one(Scope::Team, id, "restart_all_team_apps")
            .await
    }

    /// Restarts all moderated applications. `PUT /team/all/restart`
    pub async fn restart_all_team_apps(&self) -> Result<AppRestartAll, Error> {
        self.bulk(Scope::Team, Method::PUT, "restart").await
    }

    /// Changes the RAM of a moderated application. `PUT /team/{appID}/ram`
    pub async fn set_team_app_ram(&self, id: &str, quantity: u32) -> Result<(), AppRamError> {
        self.set_ram(Scope::Team, id, quantity).await
    }

    /// Replaces the code of a moderated application. `PUT /team/{appID}/commit`
    pub async fn commit_team_app(
        &self,
        id: &str,
        path: impl AsRef<Path>,
    ) -> Result<AppCommit, Error> {
        let id = app_id(id)?;
        let form = file_form(path.as_ref()).await?;
        self.commit_form(Scope::Team, id, form).await
    }

    /// Like [`commit_team_app`](Self::commit_team_app), with an in-memory archive.
    pub async fn commit_team_app_bytes(
        &self,
        id: &str,
        zip: Vec<u8>,
        file_name: &str,
    ) -> Result<AppCommit, Error> {
        let id = app_id(id)?;
        self.commit_form(Scope::Team, id, bytes_form(zip, file_name)?)
            .await
    }
}

impl App {
    /// See [`Discloud::get_app_status`].
    pub async fn get_status(&self, client: &Discloud) -> Result<AppStatus, Error> {
        client.get_app_status(&self.id).await
    }

    /// See [`Discloud::get_app_logs`].
    pub async fn get_logs(&self, client: &Discloud) -> Result<AppLogs, Error> {
        client.get_app_logs(&self.id).await
    }

    /// See [`Discloud::get_app_backup`].
    pub async fn get_backup(&self, client: &Discloud) -> Result<AppBackup, Error> {
        client.get_app_backup(&self.id).await
    }

    /// See [`Discloud::start_app`].
    pub async fn start(&self, client: &Discloud) -> Result<AppStartStatus, AppStartError> {
        client.start_app(&self.id).await
    }

    /// See [`Discloud::stop_app`].
    pub async fn stop(&self, client: &Discloud) -> Result<(), AppStopError> {
        client.stop_app(&self.id).await
    }

    /// See [`Discloud::restart_app`].
    pub async fn restart(&self, client: &Discloud) -> Result<(), Error> {
        client.restart_app(&self.id).await
    }

    /// See [`Discloud::set_app_ram`].
    pub async fn set_ram(&self, client: &Discloud, quantity: u32) -> Result<(), AppRamError> {
        client.set_app_ram(&self.id, quantity).await
    }

    /// See [`Discloud::commit_app`].
    pub async fn commit(
        &self,
        client: &Discloud,
        path: impl AsRef<Path>,
    ) -> Result<AppCommit, Error> {
        client.commit_app(&self.id, path).await
    }

    /// See [`Discloud::delete_app`].
    pub async fn delete(&self, client: &Discloud) -> Result<(), Error> {
        client.delete_app(&self.id).await
    }

    /// See [`Discloud::get_app_team`].
    pub async fn get_team(&self, client: &Discloud) -> Result<Vec<TeamMember>, Error> {
        client.get_app_team(&self.id).await
    }

    /// See [`Discloud::add_app_mod`].
    pub async fn add_mod(
        &self,
        client: &Discloud,
        mod_id: &str,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        client.add_app_mod(&self.id, mod_id, perms).await
    }

    /// See [`Discloud::edit_app_mod`].
    pub async fn edit_mod(
        &self,
        client: &Discloud,
        mod_id: &str,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        client.edit_app_mod(&self.id, mod_id, perms).await
    }

    /// See [`Discloud::remove_app_mod`].
    pub async fn remove_mod(&self, client: &Discloud, mod_id: &str) -> Result<(), Error> {
        client.remove_app_mod(&self.id, mod_id).await
    }
}
