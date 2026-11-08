//! File explorer of an application (`/app/{appID}/explorer`).

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Discloud, Error, client::app_id};

/// Kind of a file system entry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FileKind {
    /// A regular file.
    #[default]
    File,
    /// A directory.
    Directory,
}

/// An entry of a directory listing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct FileItem {
    /// File or directory name.
    pub name: String,
    /// Entry kind.
    #[serde(rename = "type")]
    pub kind: FileKind,
    /// Size in bytes (files only).
    pub size: Option<u64>,
}

/// Contents of a directory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct DirectoryListing {
    /// Listed directory.
    pub path: String,
    /// Entries of the directory.
    #[serde(rename = "dir")]
    pub entries: Vec<FileItem>,
}

/// Contents of a file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct FileContent {
    /// Full path of the file.
    pub path: String,
    /// File name.
    pub name: String,
    /// File content.
    pub content: String,
}

#[derive(Deserialize)]
struct PathResponse {
    #[serde(default)]
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateBody<'a> {
    type_file: FileKind,
    c_path: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditBody<'a> {
    file_content: &'a str,
    c_path: &'a str,
}

fn non_empty_path(path: &str) -> Result<&str, Error> {
    if path.trim().is_empty() {
        Err(Error::invalid("the file path must not be empty"))
    } else {
        Ok(path)
    }
}

impl Discloud {
    /// Lists a directory of an application. `None` lists the working
    /// directory. `GET /app/{appID}/explorer`
    pub async fn list_files(
        &self,
        app: &str,
        path: Option<&str>,
    ) -> Result<DirectoryListing, Error> {
        let app = app_id(app)?;
        let query: Vec<_> = path.map(|p| ("cPath", p.to_owned())).into_iter().collect();
        self.execute(self.request(Method::GET, &["app", app, "explorer"], &query)?)
            .await
    }

    /// Reads a file of an application. `GET /app/{appID}/explorer/open`
    pub async fn read_file(&self, app: &str, path: &str) -> Result<FileContent, Error> {
        let app = app_id(app)?;
        let query = [("cPath", non_empty_path(path)?.to_owned())];
        self.execute(self.request(Method::GET, &["app", app, "explorer", "open"], &query)?)
            .await
    }

    /// Creates an empty file or a directory. Returns the created path.
    /// `POST /app/{appID}/explorer`
    pub async fn create_file(
        &self,
        app: &str,
        path: &str,
        kind: FileKind,
    ) -> Result<String, Error> {
        let app = app_id(app)?;
        let body = CreateBody {
            type_file: kind,
            c_path: non_empty_path(path)?,
        };
        let r: PathResponse = self
            .send_json(Method::POST, &["app", app, "explorer"], &body)
            .await?;
        Ok(r.path)
    }

    /// Replaces the content of a file. Returns the edited path.
    /// `PUT /app/{appID}/explorer/edit`
    pub async fn edit_file(&self, app: &str, path: &str, content: &str) -> Result<String, Error> {
        let app = app_id(app)?;
        let body = EditBody {
            file_content: content,
            c_path: non_empty_path(path)?,
        };
        let r: PathResponse = self
            .send_json(Method::PUT, &["app", app, "explorer", "edit"], &body)
            .await?;
        Ok(r.path)
    }
}
