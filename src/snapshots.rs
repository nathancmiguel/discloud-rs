//! Versioned snapshots of application files (`/snapshot`).

use reqwest::Method;
use serde::Deserialize;

use crate::{Discloud, Error};

/// Filters for [`Discloud::get_snapshots`]. Unset fields use the API defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotQuery {
    /// Page number, starting at 1 (default 1).
    pub page: Option<u32>,
    /// Page size between 1 and 200 (default 50).
    pub limit: Option<u32>,
    /// Return a summarized listing.
    pub summary: Option<bool>,
    /// Only list the versions of this application.
    pub app_id: Option<String>,
}

/// A stored snapshot version.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct SnapshotVersion {
    /// Version identifier (`YYYYMMDD-HHMMSS`).
    pub version: String,
    /// Size in bytes.
    pub size: Option<u64>,
    /// Creation time (RFC 3339), returned by [`Discloud::get_snapshots`].
    pub created_at: Option<String>,
    /// Creation time in milliseconds, returned by [`Discloud::get_app_snapshots`].
    pub date: Option<u64>,
}

/// Result of [`Discloud::get_snapshots`].
///
/// Without an application filter the page fields and `backups` are set; with
/// one, `app_id` and `versions` are.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct SnapshotList {
    /// Current page.
    pub page: Option<u32>,
    /// Page size.
    pub limit: Option<u32>,
    /// Total number of entries.
    pub total: Option<u64>,
    /// Snapshot entries of all applications. Their shape is not documented,
    /// so they are returned as raw JSON.
    pub backups: Vec<serde_json::Value>,
    /// Application the versions belong to, when filtered.
    #[serde(rename = "app", deserialize_with = "app_ref")]
    pub app_id: Option<String>,
    /// Versions of the filtered application.
    pub versions: Vec<SnapshotVersion>,
}

/// A version listed right after creating a snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct CreatedSnapshotVersion {
    /// Version identifier.
    pub version: String,
    /// Human-readable size (e.g. `"2.5 MB"`).
    pub size: String,
    /// Creation time (RFC 3339).
    pub date: String,
}

/// Result of [`Discloud::create_snapshot`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreatedSnapshot {
    /// Version identifier of the new snapshot.
    pub version: String,
    /// Human-readable size (e.g. `"2.5 MB"`).
    pub size: String,
    /// Download URL.
    pub url: String,
    /// Every stored version of the application.
    pub all_versions: Vec<CreatedSnapshotVersion>,
}

/// A temporary download link for a snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct SnapshotDownload {
    /// Download URL.
    pub url: String,
    /// Expiration time (RFC 3339).
    pub expires_at: String,
    /// Version identifier.
    pub version: String,
    /// Size in bytes.
    pub size: Option<u64>,
}

#[derive(Deserialize)]
struct AppRef {
    #[serde(default)]
    id: String,
}

fn app_ref<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<AppRef>::deserialize(d)?.map(|a| a.id))
}

#[derive(Deserialize)]
struct VersionsResponse {
    #[serde(default)]
    versions: Vec<SnapshotVersion>,
}

#[derive(Deserialize)]
struct CreateResponse {
    snapshot: CreatedSnapshot,
}

#[derive(Deserialize)]
struct DownloadResponse {
    download: SnapshotDownload,
}

#[derive(Deserialize)]
struct DeleteResponse {
    #[serde(default)]
    deleted: String,
}

/// Checks the API rule for snapshot app IDs: `^[a-zA-Z0-9-]{1,40}$`.
fn snapshot_app(id: &str) -> Result<&str, Error> {
    let valid =
        (1..=40).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    if valid {
        Ok(id)
    } else {
        Err(Error::invalid(format!(
            "invalid application ID {id:?}: use 1 to 40 letters, digits or hyphens"
        )))
    }
}

/// Checks the version format `YYYYMMDD-HHMMSS`.
fn version(v: &str) -> Result<&str, Error> {
    let b = v.as_bytes();
    let valid = b.len() == 15
        && b[8] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 8 || c.is_ascii_digit());
    if valid {
        Ok(v)
    } else {
        Err(Error::invalid(format!(
            "invalid snapshot version {v:?}: expected YYYYMMDD-HHMMSS"
        )))
    }
}

impl Discloud {
    /// Lists snapshots, optionally filtered by application. `GET /snapshot`
    pub async fn get_snapshots(&self, query: &SnapshotQuery) -> Result<SnapshotList, Error> {
        let mut params = Vec::new();
        if let Some(page) = query.page {
            if page == 0 {
                return Err(Error::invalid("`page` starts at 1"));
            }
            params.push(("page", page.to_string()));
        }
        if let Some(limit) = query.limit {
            if !(1..=200).contains(&limit) {
                return Err(Error::invalid("`limit` must be between 1 and 200"));
            }
            params.push(("limit", limit.to_string()));
        }
        if let Some(summary) = query.summary {
            params.push(("summary", summary.to_string()));
        }
        if let Some(app) = &query.app_id {
            params.push(("appID", snapshot_app(app)?.to_owned()));
        }
        self.execute(self.request(Method::GET, &["snapshot"], &params)?)
            .await
    }

    /// Lists the snapshot versions of an application. `GET /snapshot/{appID}`
    pub async fn get_app_snapshots(&self, app: &str) -> Result<Vec<SnapshotVersion>, Error> {
        let r: VersionsResponse = self.get(&["snapshot", snapshot_app(app)?]).await?;
        Ok(r.versions)
    }

    /// Creates a snapshot of an application's files (Ruby plan or higher).
    /// `POST /snapshot/{appID}`
    pub async fn create_snapshot(&self, app: &str) -> Result<CreatedSnapshot, Error> {
        let r: CreateResponse = self
            .send(Method::POST, &["snapshot", snapshot_app(app)?])
            .await?;
        Ok(r.snapshot)
    }

    /// Returns a temporary download link for a snapshot version.
    /// `GET /snapshot/{appID}/versions/{version}`
    pub async fn get_snapshot_download(
        &self,
        app: &str,
        snapshot_version: &str,
    ) -> Result<SnapshotDownload, Error> {
        let r: DownloadResponse = self
            .get(&[
                "snapshot",
                snapshot_app(app)?,
                "versions",
                version(snapshot_version)?,
            ])
            .await?;
        Ok(r.download)
    }

    /// Deletes a snapshot version. Returns the deleted storage key.
    /// `DELETE /snapshot/{appID}/versions/{version}`
    pub async fn delete_snapshot(
        &self,
        app: &str,
        snapshot_version: &str,
    ) -> Result<String, Error> {
        let r: DeleteResponse = self
            .send(
                Method::DELETE,
                &[
                    "snapshot",
                    snapshot_app(app)?,
                    "versions",
                    version(snapshot_version)?,
                ],
            )
            .await?;
        Ok(r.deleted)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_format() {
        assert!(super::version("20240101-120000").is_ok());
        for bad in ["2024-01-01", "20240101120000", "2024010a-120000", ""] {
            assert!(super::version(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn app_id_format() {
        assert!(super::snapshot_app("1720000000000").is_ok());
        assert!(super::snapshot_app("../x").is_err());
    }
}
