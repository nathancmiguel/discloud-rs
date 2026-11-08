//! Projects grouping applications (`/projects`).

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    Discloud, Error,
    client::app_id,
    wire::{DataResponse, MessageResponse},
};

/// A project.
///
/// The API returns different subsets of these fields depending on the route,
/// so most of them are optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct Project {
    /// Project ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Project name.
    pub name: String,
    /// Owner's user ID.
    #[serde(rename = "ownerID")]
    pub owner_id: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Linked team, if any.
    #[serde(rename = "teamID")]
    pub team_id: Option<String>,
    /// Cluster hosting the project's applications.
    #[serde(rename = "clusterName")]
    pub cluster_name: Option<String>,
}

/// Summary of a project inside [`ProjectDetails`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ProjectSummary {
    /// Project ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Project name.
    pub name: String,
    /// Your role in the project (e.g. `owner`).
    #[serde(rename = "memberRole")]
    pub member_role: Option<String>,
}

/// An application as listed by the project routes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ProjectApp {
    /// Application ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Application name tag.
    #[serde(rename = "nameTAG")]
    pub name_tag: String,
    /// Cluster hosting the application.
    #[serde(rename = "clusterName")]
    pub cluster_name: Option<String>,
}

/// Result of [`Discloud::get_project`].
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ProjectDetails {
    /// The project.
    pub project: ProjectSummary,
    /// The linked team, if any. Its shape is not documented, so it is raw JSON.
    pub team: Option<serde_json::Value>,
    /// Applications in the project.
    pub apps: Vec<ProjectApp>,
}

/// Fields of a new project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CreateProject {
    /// Project name (at most 50 characters).
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Team to link.
    #[serde(rename = "teamID", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// Applications to move into the project.
    #[serde(rename = "fromAppIDs", skip_serializing_if = "Vec::is_empty")]
    pub from_app_ids: Vec<String>,
}

/// Fields to change on a project. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UpdateProject {
    /// New name (at most 50 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// An application linked to a project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ProjectAppLink {
    /// Application ID.
    #[serde(rename = "_id")]
    pub app_id: String,
    /// Project ID.
    #[serde(rename = "projectID")]
    pub project_id: String,
    /// Cluster hosting the application.
    #[serde(rename = "clusterName")]
    pub cluster_name: Option<String>,
}

/// A cluster migration started by [`Discloud::add_app_to_project`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProjectMigration {
    /// Application ID.
    #[serde(rename = "appID")]
    pub app_id: String,
    /// Project ID.
    #[serde(rename = "projectID")]
    pub project_id: String,
    /// Source cluster.
    pub from: String,
    /// Destination cluster.
    pub to: String,
    /// Estimated duration in seconds.
    pub estimated_sec: Option<u64>,
}

/// Result of [`Discloud::add_app_to_project`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AddAppToProject {
    /// The application was added (HTTP 200).
    Added(ProjectAppLink),
    /// The application is being migrated to the project's cluster first
    /// (HTTP 202).
    MigrationStarted(ProjectMigration),
}

#[derive(Serialize)]
struct TeamBody<'a> {
    #[serde(rename = "teamID")]
    team_id: Option<&'a str>,
}

#[derive(Serialize)]
struct MigrateBody {
    migrate: bool,
}

fn project(id: &str) -> Result<&str, Error> {
    if id.trim().is_empty() {
        Err(Error::invalid("the project ID must not be empty"))
    } else {
        Ok(id)
    }
}

fn check_name(name: &str) -> Result<(), Error> {
    let len = name.chars().count();
    if len == 0 || len > 50 {
        Err(Error::invalid(
            "the name must have between 1 and 50 characters",
        ))
    } else {
        Ok(())
    }
}

impl Discloud {
    /// Creates a project. `POST /projects`
    pub async fn create_project(&self, new: &CreateProject) -> Result<Project, Error> {
        check_name(&new.name)?;
        let r: DataResponse<_> = self.send_json(Method::POST, &["projects"], new).await?;
        Ok(r.data)
    }

    /// Lists your projects. `GET /projects`
    pub async fn get_projects(&self) -> Result<Vec<Project>, Error> {
        let r: DataResponse<_> = self.get(&["projects"]).await?;
        Ok(r.data)
    }

    /// Returns a project with its team and applications. `GET /projects/{projectID}`
    pub async fn get_project(&self, id: &str) -> Result<ProjectDetails, Error> {
        let r: DataResponse<_> = self.get(&["projects", project(id)?]).await?;
        Ok(r.data)
    }

    /// Updates a project. `PATCH /projects/{projectID}`
    pub async fn update_project(&self, id: &str, update: &UpdateProject) -> Result<Project, Error> {
        if let Some(name) = &update.name {
            check_name(name)?;
        }
        let r: DataResponse<_> = self
            .send_json(Method::PATCH, &["projects", project(id)?], update)
            .await?;
        Ok(r.data)
    }

    /// Deletes a project. `DELETE /projects/{projectID}`
    pub async fn delete_project(&self, id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["projects", project(id)?])
            .await
            .map(drop)
    }

    /// Links a team to a project, or unlinks it with `None`.
    /// `PATCH /projects/{projectID}/team`
    pub async fn set_project_team(
        &self,
        id: &str,
        team_id: Option<&str>,
    ) -> Result<Project, Error> {
        let r: DataResponse<_> = self
            .send_json(
                Method::PATCH,
                &["projects", project(id)?, "team"],
                &TeamBody { team_id },
            )
            .await?;
        Ok(r.data)
    }

    /// Lists your applications that can be added to a project.
    /// `GET /projects/{projectID}/apps`
    pub async fn get_project_available_apps(&self, id: &str) -> Result<Vec<ProjectApp>, Error> {
        let r: DataResponse<_> = self.get(&["projects", project(id)?, "apps"]).await?;
        Ok(r.data)
    }

    /// Adds an application to a project. With `migrate`, an application on
    /// another cluster is migrated first; without it, that case fails with
    /// [`Error::Conflict`] (code `CLUSTER_MISMATCH`).
    /// `PUT /projects/{projectID}/apps/{appID}`
    pub async fn add_app_to_project(
        &self,
        project_id: &str,
        app: &str,
        migrate: bool,
    ) -> Result<AddAppToProject, Error> {
        let request = self
            .request(
                Method::PUT,
                &["projects", project(project_id)?, "apps", app_id(app)?],
                &[],
            )?
            .json(&MigrateBody { migrate });
        let (status, r): (_, DataResponse<serde_json::Value>) =
            self.execute_with_status(request).await?;
        if status == StatusCode::ACCEPTED {
            serde_json::from_value(r.data)
                .map(AddAppToProject::MigrationStarted)
                .map_err(Error::Decode)
        } else {
            serde_json::from_value(r.data)
                .map(AddAppToProject::Added)
                .map_err(Error::Decode)
        }
    }

    /// Removes an application from a project.
    /// `DELETE /projects/{projectID}/apps/{appID}`
    pub async fn remove_app_from_project(&self, project_id: &str, app: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(
            Method::DELETE,
            &["projects", project(project_id)?, "apps", app_id(app)?],
        )
        .await
        .map(drop)
    }
}
