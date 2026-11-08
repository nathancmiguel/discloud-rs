//! Applications of other users that you moderate (`GET /team`).

use std::path::Path;

use serde::Deserialize;

use crate::{
    AppBackup, AppCommit, AppLogs, AppStartStatus, AppStatus, Discloud, Error, TeamPerms,
    apps::AppType,
    error::{AppRamError, AppStartError, AppStopError},
};

/// Owner of a moderated application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamAppOwner {
    /// Owner's name.
    pub name: String,
    /// Owner's avatar URL.
    pub avatar: Option<String>,
}

/// An application you moderate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct TeamApp {
    /// Application ID.
    pub id: String,
    /// Application name.
    pub name: String,
    /// [`AppType`]: `0` = bot, `1` = site.
    pub r#type: AppType,
    /// Whether the application is online.
    pub online: bool,
    /// Allocated RAM in MB.
    pub ram: u32,
    /// Whether the application was killed for exceeding its RAM.
    pub ram_killed: bool,
    /// Exit code of the last run, if it stopped.
    pub exit_code: Option<i32>,
    /// Programming language.
    pub lang: String,
    /// Owner of the application.
    pub owner: Option<TeamAppOwner>,
    /// Your permissions on this application.
    pub perms: Vec<TeamPerms>,
}

#[derive(Deserialize)]
struct TeamAppsResponse {
    #[serde(default)]
    apps: Vec<TeamApp>,
}

impl Discloud {
    /// Lists the applications you moderate. `GET /team`
    pub async fn get_team_apps(&self) -> Result<Vec<TeamApp>, Error> {
        let r: TeamAppsResponse = self.get(&["team"]).await?;
        Ok(r.apps)
    }
}

impl TeamApp {
    /// See [`Discloud::start_team_app`].
    pub async fn start(&self, client: &Discloud) -> Result<AppStartStatus, AppStartError> {
        client.start_team_app(&self.id).await
    }

    /// See [`Discloud::restart_team_app`].
    pub async fn restart(&self, client: &Discloud) -> Result<(), Error> {
        client.restart_team_app(&self.id).await
    }

    /// See [`Discloud::stop_team_app`].
    pub async fn stop(&self, client: &Discloud) -> Result<(), AppStopError> {
        client.stop_team_app(&self.id).await
    }

    /// See [`Discloud::commit_team_app`].
    pub async fn commit(
        &self,
        client: &Discloud,
        path: impl AsRef<Path>,
    ) -> Result<AppCommit, Error> {
        client.commit_team_app(&self.id, path).await
    }

    /// See [`Discloud::get_team_app_backup`].
    pub async fn get_backup(&self, client: &Discloud) -> Result<AppBackup, Error> {
        client.get_team_app_backup(&self.id).await
    }

    /// See [`Discloud::get_team_app_logs`].
    pub async fn get_logs(&self, client: &Discloud) -> Result<AppLogs, Error> {
        client.get_team_app_logs(&self.id).await
    }

    /// See [`Discloud::set_team_app_ram`].
    pub async fn set_ram(&self, client: &Discloud, quantity: u32) -> Result<(), AppRamError> {
        client.set_team_app_ram(&self.id, quantity).await
    }

    /// See [`Discloud::get_team_app_status`].
    pub async fn get_status(&self, client: &Discloud) -> Result<AppStatus, Error> {
        client.get_team_app_status(&self.id).await
    }
}
