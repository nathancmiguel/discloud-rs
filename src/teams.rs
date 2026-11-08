//! Teams and their members (`/teams`).
//!
//! Not to be confused with [`Discloud::get_team_apps`] (`GET /team`), which
//! lists applications you moderate.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    Discloud, Error,
    wire::{DataResponse, MessageResponse},
};

/// A team.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct Team {
    /// Team ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Team name.
    pub name: String,
    /// Owner's user ID.
    #[serde(rename = "ownerID")]
    pub owner_id: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Whether you own the team (listings only).
    pub is_owner: Option<bool>,
}

/// A member of a team.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamMemberInfo {
    /// User ID.
    #[serde(rename = "userID")]
    pub user_id: String,
    /// Role (`admin`, `developer`, `viewer`, ...).
    pub role: String,
    /// Username, when returned.
    pub username: Option<String>,
}

/// A project linked to a team.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamProject {
    /// Project ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Project name.
    pub name: String,
}

/// Result of [`Discloud::get_team`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamDetails {
    /// The team.
    pub team: Team,
    /// Members of the team.
    pub members: Vec<TeamMemberInfo>,
    /// Projects linked to the team.
    pub projects: Vec<TeamProject>,
}

/// An application reachable through one of your teams.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct TeamsApp {
    /// Application ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Application name tag.
    #[serde(rename = "nameTAG")]
    pub name_tag: String,
    /// Team granting access.
    #[serde(rename = "teamID")]
    pub team_id: String,
}

/// Permissions of a team role.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct RolePermissions {
    /// Role name.
    pub role: String,
    /// Permissions granted by the role (e.g. `config_app`, `view_logs`).
    pub permissions: Vec<String>,
}

/// Fields of a new team.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CreateTeam {
    /// Team name (at most 50 characters).
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Fields to change on a team. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UpdateTeam {
    /// New name (at most 50 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize)]
struct AddMemberBody<'a> {
    #[serde(rename = "userID")]
    user_id: &'a str,
    role: &'a str,
}

#[derive(Serialize)]
struct RoleBody<'a> {
    role: &'a str,
}

fn non_empty<'a>(what: &str, value: &'a str) -> Result<&'a str, Error> {
    if value.trim().is_empty() {
        Err(Error::invalid(format!("the {what} must not be empty")))
    } else {
        Ok(value)
    }
}

fn check_name(name: &str) -> Result<(), Error> {
    let len = name.chars().count();
    if len == 0 || len > 50 {
        Err(Error::invalid(
            "the team name must have between 1 and 50 characters",
        ))
    } else {
        Ok(())
    }
}

impl Discloud {
    /// Creates a team. `POST /teams`
    pub async fn create_team(&self, new: &CreateTeam) -> Result<Team, Error> {
        check_name(&new.name)?;
        let r: DataResponse<_> = self.send_json(Method::POST, &["teams"], new).await?;
        Ok(r.data)
    }

    /// Lists the teams you belong to. `GET /teams`
    pub async fn get_teams(&self) -> Result<Vec<Team>, Error> {
        let r: DataResponse<_> = self.get(&["teams"]).await?;
        Ok(r.data)
    }

    /// Lists the applications reachable through your teams. `GET /teams/apps`
    pub async fn get_teams_apps(&self) -> Result<Vec<TeamsApp>, Error> {
        let r: DataResponse<_> = self.get(&["teams", "apps"]).await?;
        Ok(r.data)
    }

    /// Returns the permissions of a role. `GET /teams/roles/{role}/permissions`
    pub async fn get_team_role_permissions(&self, role: &str) -> Result<RolePermissions, Error> {
        let r: DataResponse<_> = self
            .get(&["teams", "roles", non_empty("role", role)?, "permissions"])
            .await?;
        Ok(r.data)
    }

    /// Returns a team with its members and projects. `GET /teams/{teamID}`
    pub async fn get_team(&self, team_id: &str) -> Result<TeamDetails, Error> {
        let r: DataResponse<_> = self.get(&["teams", non_empty("team ID", team_id)?]).await?;
        Ok(r.data)
    }

    /// Updates a team. `PATCH /teams/{teamID}`
    pub async fn update_team(&self, team_id: &str, update: &UpdateTeam) -> Result<Team, Error> {
        if let Some(name) = &update.name {
            check_name(name)?;
        }
        let r: DataResponse<_> = self
            .send_json(
                Method::PATCH,
                &["teams", non_empty("team ID", team_id)?],
                update,
            )
            .await?;
        Ok(r.data)
    }

    /// Deletes a team. `DELETE /teams/{teamID}`
    pub async fn delete_team(&self, team_id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["teams", non_empty("team ID", team_id)?])
            .await
            .map(drop)
    }

    /// Adds a member to a team. `POST /teams/{teamID}/members`
    pub async fn add_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TeamMemberInfo, Error> {
        let body = AddMemberBody {
            user_id: non_empty("user ID", user_id)?,
            role: non_empty("role", role)?,
        };
        let r: DataResponse<_> = self
            .send_json(
                Method::POST,
                &["teams", non_empty("team ID", team_id)?, "members"],
                &body,
            )
            .await?;
        Ok(r.data)
    }

    /// Lists the members of a team. `GET /teams/{teamID}/members`
    pub async fn get_team_members(&self, team_id: &str) -> Result<Vec<TeamMemberInfo>, Error> {
        let r: DataResponse<_> = self
            .get(&["teams", non_empty("team ID", team_id)?, "members"])
            .await?;
        Ok(r.data)
    }

    /// Changes the role of a member. `PATCH /teams/{teamID}/members/{memberUserID}`
    pub async fn update_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TeamMemberInfo, Error> {
        let r: DataResponse<_> = self
            .send_json(
                Method::PATCH,
                &["teams", non_empty("team ID", team_id)?, "members", user_id],
                &RoleBody {
                    role: non_empty("role", role)?,
                },
            )
            .await?;
        Ok(r.data)
    }

    /// Removes a member from a team. `DELETE /teams/{teamID}/members/{memberUserID}`
    pub async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(
            Method::DELETE,
            &["teams", non_empty("team ID", team_id)?, "members", user_id],
        )
        .await
        .map(drop)
    }

    /// Leaves a team (the owner cannot leave). `POST /teams/{teamID}/leave`
    pub async fn leave_team(&self, team_id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(
            Method::POST,
            &["teams", non_empty("team ID", team_id)?, "leave"],
        )
        .await
        .map(drop)
    }
}
