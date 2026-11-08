//! Uploading new applications (`POST /upload`).

use std::path::Path;

use reqwest::{Method, multipart::Form};
use serde::Deserialize;

use crate::{
    Discloud, Error,
    apps::{AppType, bytes_form, file_form},
};

/// An application created by an upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct UploadApp {
    /// Application ID.
    pub id: String,
    /// [`AppType`]: `0` = bot, `1` = site.
    pub r#type: AppType,
    /// Application name.
    pub name: String,
    /// Avatar URL.
    #[serde(rename = "avatarURL", alias = "avatarUrl")]
    pub avatar_url: Option<String>,
    /// Allocated RAM in MB.
    pub ram: u32,
    /// Entry point file.
    pub main_file: String,
    /// Programming language.
    pub lang: String,
    /// `VERSION` from `discloud.config`.
    pub version: Option<String>,
    /// Upload time in milliseconds since the Unix epoch.
    #[serde(alias = "addedTimestamp")]
    pub added_at_timestamp: u64,
    /// Whether the application restarts automatically.
    pub auto_restart: bool,
}

#[derive(Deserialize)]
struct UploadResponse {
    app: UploadApp,
}

impl Discloud {
    /// Uploads a new application from a `.zip` archive on disk (it must
    /// contain a `discloud.config`). `POST /upload`
    pub async fn upload_app(&self, path: impl AsRef<Path>) -> Result<UploadApp, Error> {
        self.upload_form(file_form(path.as_ref()).await?, None)
            .await
    }

    /// Uploads a new application into a project. `POST /upload`
    pub async fn upload_app_to_project(
        &self,
        path: impl AsRef<Path>,
        project_id: &str,
    ) -> Result<UploadApp, Error> {
        self.upload_form(file_form(path.as_ref()).await?, Some(project_id))
            .await
    }

    /// Uploads a new application from an in-memory `.zip` archive,
    /// optionally into a project. `POST /upload`
    pub async fn upload_app_bytes(
        &self,
        zip: Vec<u8>,
        file_name: &str,
        project_id: Option<&str>,
    ) -> Result<UploadApp, Error> {
        self.upload_form(bytes_form(zip, file_name)?, project_id)
            .await
    }

    async fn upload_form(
        &self,
        mut form: Form,
        project_id: Option<&str>,
    ) -> Result<UploadApp, Error> {
        if let Some(project) = project_id {
            if project.trim().is_empty() {
                return Err(Error::invalid("the project ID must not be empty"));
            }
            form = form.text("projectId", project.to_owned());
        }
        let request = self
            .request(Method::POST, &["upload"], &[])?
            .multipart(form);
        let r: UploadResponse = self.execute(request).await?;
        Ok(r.app)
    }
}
