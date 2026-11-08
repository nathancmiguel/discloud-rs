//! Environment variables at account, project and application level (`/env`).
//!
//! Values are secrets: the client never logs request or response bodies.

use std::collections::BTreeMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Discloud, Error, client::app_id, wire::DataResponse};

/// Environment variables, sorted by name.
pub type EnvVars = BTreeMap<String, String>;

#[derive(Deserialize)]
struct VarsData {
    #[serde(default)]
    vars: EnvVars,
}

#[derive(Deserialize)]
struct RemovedData {
    #[serde(default)]
    removed: u64,
}

#[derive(Serialize)]
struct VarsBody<'a> {
    vars: &'a EnvVars,
}

fn collect<I, K, V>(vars: I) -> EnvVars
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    vars.into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
}

fn project(id: &str) -> Result<&str, Error> {
    if id.trim().is_empty() {
        Err(Error::invalid("the project ID must not be empty"))
    } else {
        Ok(id)
    }
}

impl Discloud {
    async fn env_get(&self, path: &[&str]) -> Result<EnvVars, Error> {
        let r: DataResponse<VarsData> = self.get(path).await?;
        Ok(r.data.vars)
    }

    async fn env_set(&self, path: &[&str], vars: EnvVars) -> Result<EnvVars, Error> {
        let r: DataResponse<VarsData> = self
            .send_json(Method::PUT, path, &VarsBody { vars: &vars })
            .await?;
        Ok(r.data.vars)
    }

    async fn env_delete(&self, path: &[&str]) -> Result<u64, Error> {
        let r: DataResponse<RemovedData> = self.send(Method::DELETE, path).await?;
        Ok(r.data.removed)
    }

    /// Returns the account-wide variables. `GET /env/account`
    pub async fn get_account_env(&self) -> Result<EnvVars, Error> {
        self.env_get(&["env", "account"]).await
    }

    /// Replaces all account-wide variables and returns the stored set.
    /// `PUT /env/account`
    pub async fn set_account_env<I, K, V>(&self, vars: I) -> Result<EnvVars, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.env_set(&["env", "account"], collect(vars)).await
    }

    /// Removes all account-wide variables and returns how many were removed.
    /// `DELETE /env/account`
    pub async fn delete_account_env(&self) -> Result<u64, Error> {
        self.env_delete(&["env", "account"]).await
    }

    /// Returns the variables of a project. `GET /env/project/{projectID}`
    pub async fn get_project_env(&self, project_id: &str) -> Result<EnvVars, Error> {
        self.env_get(&["env", "project", project(project_id)?])
            .await
    }

    /// Replaces the variables of a project. `PUT /env/project/{projectID}`
    pub async fn set_project_env<I, K, V>(
        &self,
        project_id: &str,
        vars: I,
    ) -> Result<EnvVars, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.env_set(&["env", "project", project(project_id)?], collect(vars))
            .await
    }

    /// Removes the variables of a project. `DELETE /env/project/{projectID}`
    pub async fn delete_project_env(&self, project_id: &str) -> Result<u64, Error> {
        self.env_delete(&["env", "project", project(project_id)?])
            .await
    }

    /// Returns the variables of an application. `GET /env/app/{appID}`
    pub async fn get_app_env(&self, app: &str) -> Result<EnvVars, Error> {
        self.env_get(&["env", "app", app_id(app)?]).await
    }

    /// Replaces the variables of an application. `PUT /env/app/{appID}`
    pub async fn set_app_env<I, K, V>(&self, app: &str, vars: I) -> Result<EnvVars, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.env_set(&["env", "app", app_id(app)?], collect(vars))
            .await
    }

    /// Removes the variables of an application. `DELETE /env/app/{appID}`
    pub async fn delete_app_env(&self, app: &str) -> Result<u64, Error> {
        self.env_delete(&["env", "app", app_id(app)?]).await
    }
}
