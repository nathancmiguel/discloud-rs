//! Moderators of your applications (`/app/{appID}/team`).

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Discloud, Error, client::app_id, wire::MessageResponse};

/// Permission that can be granted to an application moderator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TeamPerms {
    /// `start_app`
    #[serde(rename = "start_app")]
    Start,
    /// `stop_app`
    #[serde(rename = "stop_app")]
    Stop,
    /// `restart_app`
    #[serde(rename = "restart_app")]
    Restart,
    /// `logs_app`
    #[serde(rename = "logs_app")]
    Logs,
    /// `commit_app`
    #[serde(rename = "commit_app")]
    Commit,
    /// `edit_ram`
    #[serde(rename = "edit_ram")]
    Ram,
    /// `backup_app`
    #[serde(rename = "backup_app")]
    Backup,
    /// `status_app`
    #[serde(rename = "status_app")]
    Status,
    /// A permission this version of the crate does not know about.
    /// Never send it back to the API.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// A moderator of one of your applications.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamMember {
    /// User ID of the moderator.
    #[serde(rename = "modID")]
    pub mod_id: String,
    /// ID of the moderated application.
    #[serde(rename = "appID")]
    pub app_id: String,
    /// Granted permissions.
    pub perms: Vec<TeamPerms>,
    /// Username of the moderator, if known.
    pub username: Option<String>,
    /// Avatar URL of the moderator, if known.
    pub avatar: Option<String>,
}

/// Raw moderator entry as returned by `GET /app/{appID}/team`.
#[deprecated(since = "0.4.0", note = "use `TeamMember`, returned by `get_app_team`")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct APITeamMember {
    /// User ID of the moderator.
    #[serde(rename = "modID")]
    pub id: String,
    /// Granted permissions.
    pub perms: Vec<TeamPerms>,
}

#[derive(Deserialize)]
struct ModeratorEntry {
    #[serde(rename = "modID")]
    mod_id: String,
    #[serde(default)]
    perms: Vec<TeamPerms>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
}

#[derive(Deserialize)]
struct ListResponse {
    #[serde(default)]
    team: Vec<ModeratorEntry>,
}

#[derive(Deserialize)]
struct MemberResponse {
    app: TeamMember,
}

#[derive(Serialize)]
struct MemberBody<'a> {
    #[serde(rename = "modID")]
    mod_id: &'a str,
    perms: &'a [TeamPerms],
}

fn validate(mod_id: &str, perms: &[TeamPerms]) -> Result<(), Error> {
    if mod_id.trim().is_empty() {
        return Err(Error::invalid("the moderator ID must not be empty"));
    }
    if perms.contains(&TeamPerms::Unknown) {
        return Err(Error::invalid(
            "`TeamPerms::Unknown` cannot be sent to the API",
        ));
    }
    Ok(())
}

impl Discloud {
    /// Lists the moderators of an application. `GET /app/{appID}/team`
    pub async fn get_app_team(&self, id: &str) -> Result<Vec<TeamMember>, Error> {
        let id = app_id(id)?;
        let r: ListResponse = self.get(&["app", id, "team"]).await?;
        Ok(r.team
            .into_iter()
            .map(|m| TeamMember {
                mod_id: m.mod_id,
                app_id: id.to_owned(),
                perms: m.perms,
                username: m.username,
                avatar: m.avatar,
            })
            .collect())
    }

    /// Adds a moderator to an application. `POST /app/{appID}/team`
    pub async fn add_app_mod(
        &self,
        app: &str,
        mod_id: &str,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        self.write_mod(Method::POST, app, mod_id, &perms).await
    }

    /// Replaces the permissions of a moderator. `PUT /app/{appID}/team`
    pub async fn edit_app_mod(
        &self,
        app: &str,
        mod_id: &str,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        self.write_mod(Method::PUT, app, mod_id, &perms).await
    }

    async fn write_mod(
        &self,
        method: Method,
        app: &str,
        mod_id: &str,
        perms: &[TeamPerms],
    ) -> Result<TeamMember, Error> {
        let app = app_id(app)?;
        validate(mod_id, perms)?;
        let r: MemberResponse = self
            .send_json(method, &["app", app, "team"], &MemberBody { mod_id, perms })
            .await?;
        let mut member = r.app;
        if member.app_id.is_empty() {
            app.clone_into(&mut member.app_id);
        }
        Ok(member)
    }

    /// Removes a moderator from an application. `DELETE /app/{appID}/team/{modID}`
    pub async fn remove_app_mod(&self, app: &str, mod_id: &str) -> Result<(), Error> {
        let app = app_id(app)?;
        self.send::<MessageResponse>(Method::DELETE, &["app", app, "team", mod_id])
            .await
            .map(drop)
    }
}

impl TeamMember {
    /// Removes this moderator. See [`Discloud::remove_app_mod`].
    pub async fn remove(&self, client: &Discloud) -> Result<(), Error> {
        client.remove_app_mod(&self.app_id, &self.mod_id).await
    }

    /// Replaces the permissions and returns the updated moderator.
    pub async fn edit_perms(
        &self,
        client: &Discloud,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        client.edit_app_mod(&self.app_id, &self.mod_id, perms).await
    }

    /// Replaces the permissions and updates `self` in place.
    pub async fn edit_perms_mut(
        &mut self,
        client: &Discloud,
        perms: Vec<TeamPerms>,
    ) -> Result<(), Error> {
        *self = self.edit_perms(client, perms).await?;
        Ok(())
    }

    /// Grants additional permissions and returns the updated moderator.
    pub async fn add_perms(
        &self,
        client: &Discloud,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        let mut merged = self.perms.clone();
        merged.extend(perms);
        merged.retain(|p| *p != TeamPerms::Unknown);
        merged.sort_unstable();
        merged.dedup();
        self.edit_perms(client, merged).await
    }

    /// Grants additional permissions and updates `self` in place.
    pub async fn add_perms_mut(
        &mut self,
        client: &Discloud,
        perms: Vec<TeamPerms>,
    ) -> Result<(), Error> {
        *self = self.add_perms(client, perms).await?;
        Ok(())
    }
}

/// Moderator operations bound to one application.
#[deprecated(
    since = "0.4.0",
    note = "call the methods on `Discloud` or `App` directly"
)]
#[derive(Debug, Clone)]
pub struct TeamManager {
    /// ID of the managed application.
    pub app_id: String,
}

#[allow(deprecated)]
impl TeamManager {
    /// See [`Discloud::get_app_team`].
    pub async fn get_app_team(&self, client: &Discloud) -> Result<Vec<TeamMember>, Error> {
        client.get_app_team(&self.app_id).await
    }

    /// See [`Discloud::add_app_mod`].
    pub async fn add_app_mod(
        &self,
        client: &Discloud,
        mod_id: &str,
        perms: Vec<TeamPerms>,
    ) -> Result<TeamMember, Error> {
        client.add_app_mod(&self.app_id, mod_id, perms).await
    }
}
