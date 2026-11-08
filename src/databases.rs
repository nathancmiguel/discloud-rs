//! Managed databases (`/database`).
//!
//! Creation and restoration are asynchronous: they return the new database ID
//! right away; poll [`Discloud::get_database`] to follow the provisioning.

use std::collections::BTreeMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    Discloud, Error,
    apps::MIN_RAM_MB,
    env::EnvVars,
    wire::{DataResponse, MessageResponse, string_or_number},
};

/// Environment variables a preset expects.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabasePresetVars {
    /// Required variable names.
    pub required: Vec<String>,
    /// Optional variable names.
    pub optional: Vec<String>,
    /// Variables filled from the preset suggestions
    /// (variable name → suggestion field name).
    pub from_suggestions: BTreeMap<String, String>,
}

/// A database engine offered by the platform.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabasePreset {
    /// Engine (`postgres`, `mysql`, `mariadb`, `mongodb`, `redis`, `valkey`, `clickhouse`).
    pub r#type: String,
    /// Available versions.
    pub versions: Vec<String>,
    /// Available extensions.
    pub extensions: Vec<String>,
    /// Default port.
    pub default_port: Option<u16>,
    /// Suggested display name.
    pub suggested_display_name: Option<String>,
    /// Suggested database name.
    pub suggested_db_name: Option<String>,
    /// Suggested user name.
    pub suggested_user: Option<String>,
    /// Suggested password.
    pub suggested_password: Option<String>,
    /// Expected environment variables.
    pub vars: DatabasePresetVars,
}

/// Random names suggested for a new database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabaseNames {
    /// Suggested display name.
    pub suggested_display_name: String,
    /// Suggested database name.
    pub suggested_db_name: String,
    /// Suggested user name.
    pub suggested_user: String,
}

/// Engine information of a database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct DatabaseEngine {
    /// Engine name.
    pub r#type: String,
    /// Engine version.
    pub version: String,
    /// Port.
    pub port: Option<u16>,
}

/// A database as listed by [`Discloud::get_databases`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabaseSummary {
    /// Database ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Owner's user ID.
    #[serde(rename = "userID")]
    pub user_id: String,
    /// Display name.
    pub display_name: String,
    /// Project the database belongs to.
    #[serde(rename = "projectID")]
    pub project_id: Option<String>,
    /// Engine information.
    pub db: DatabaseEngine,
    /// Hosting cluster.
    pub cluster_name: Option<String>,
    /// Allocated RAM in MB.
    pub total_ram: u32,
    /// Hostname.
    pub hostname: String,
    /// Creation time in milliseconds since the Unix epoch.
    pub added_at_timestamp: Option<u64>,
}

/// Owner of a database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct DatabaseOwner {
    /// User ID.
    pub id: String,
    /// Username.
    pub username: Option<String>,
    /// Avatar URL.
    pub avatar: Option<String>,
}

/// Full information about a database.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct Database {
    /// Database ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Project the database belongs to.
    #[serde(rename = "projectID")]
    pub project_id: Option<String>,
    /// Hosting cluster.
    pub cluster_name: Option<String>,
    /// Hostname.
    pub hostname: String,
    /// Display name.
    pub display_name: String,
    /// Database type (e.g. `postgresql`).
    pub db_type: String,
    /// Engine (e.g. `postgres`).
    pub db_engine: String,
    /// Engine version.
    pub db_version: String,
    /// Enabled extensions.
    pub extensions: Vec<String>,
    /// Allocated RAM in MB.
    pub total_ram: u32,
    /// RAM in use, in MB.
    pub total_used_ram: Option<f64>,
    /// Owner.
    pub owner: Option<DatabaseOwner>,
    /// Your permissions, when you are not the owner (raw JSON).
    pub perms: Option<serde_json::Value>,
    /// Environment variables (credentials).
    pub vars: EnvVars,
    /// Whether the database is running.
    pub online: bool,
    /// Whether the container exists.
    pub exists: bool,
    /// Time since the last restart.
    #[serde(rename = "last_restart")]
    pub last_restart: Option<String>,
    /// Human-readable size of the data volume.
    pub data_volume_size_human: Option<String>,
}

/// Engine of a new database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DatabaseSpec {
    /// Engine name (see [`DatabasePreset::type`]).
    #[serde(rename = "type")]
    pub kind: String,
    /// Engine version.
    pub version: String,
    /// Extensions to enable.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,
    /// Port (defaults to the engine's port).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Fields of a new database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatabase {
    /// Display name.
    pub display_name: String,
    /// Engine.
    pub db: DatabaseSpec,
    /// RAM in MB (at least 100).
    pub total_ram: u32,
    /// Environment variables (credentials).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: EnvVars,
    /// Project to create the database in.
    #[serde(rename = "projectID", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Restart automatically after a crash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_restart: Option<bool>,
}

/// Fields to change on a database. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatabase {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// New hostname (at most 25 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// New RAM in MB (100 to 16384); triggers a rebuild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ram: Option<u32>,
}

/// Result of [`Discloud::update_database`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdatedDatabase {
    /// Database ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Display name.
    pub display_name: Option<String>,
    /// Allocated RAM in MB.
    pub total_ram: Option<u32>,
    /// Hostname.
    pub hostname: Option<String>,
    /// Whether the change triggered a rebuild of the container.
    #[serde(skip)]
    pub rebuilding: bool,
}

/// Fields to restore a backup into a new database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDatabase {
    /// Backup to restore.
    pub backup_id: String,
    /// RAM in MB (at least 100).
    pub total_ram: u32,
    /// Display name of the new database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Project to create the database in.
    #[serde(rename = "projectID", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Environment variables (mutually exclusive with `vars_encrypted`).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: EnvVars,
    /// Encrypted variables (mutually exclusive with `vars`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars_encrypted: Option<String>,
    /// Restart automatically after a crash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_restart: Option<bool>,
    /// Engine version of the new database.
    #[serde(
        rename = "db",
        skip_serializing_if = "Option::is_none",
        serialize_with = "version_obj"
    )]
    pub db_version: Option<String>,
}

fn version_obj<S: serde::Serializer>(v: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    struct Version<'a> {
        version: &'a str,
    }
    match v {
        Some(version) => Version { version }.serialize(s),
        None => s.serialize_none(),
    }
}

/// A backup created by [`Discloud::backup_database`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct NewDatabaseBackup {
    /// Backup ID.
    #[serde(deserialize_with = "string_or_number")]
    pub backup_id: String,
    /// Download URL.
    pub download_url: String,
}

/// A stored database backup.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabaseBackup {
    /// Backup ID.
    #[serde(deserialize_with = "string_or_number")]
    pub backup_id: String,
    /// Creation time (RFC 3339).
    pub created_at: String,
    /// Size in bytes.
    pub size: Option<u64>,
}

/// A page of database backups.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabaseBackupPage {
    /// Backups of this page.
    pub backups: Vec<DatabaseBackup>,
    /// Cursor of the next page, if there is one.
    pub next_cursor: Option<String>,
}

/// Detail level of [`Discloud::get_database_backups`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DatabaseBackupDetail {
    /// IDs only.
    Minimal,
    /// IDs, dates and sizes.
    Full,
}

/// Paging options of [`Discloud::get_database_backups`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatabaseBackupQuery {
    /// Page size between 1 and 1000 (default 50).
    pub limit: Option<u32>,
    /// Cursor returned by the previous page.
    pub cursor: Option<String>,
    /// Detail level.
    pub detail: Option<DatabaseBackupDetail>,
}

/// A temporary download link for a database backup.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatabaseDownload {
    /// Download URL.
    pub download_url: String,
    /// Expiration time (RFC 3339).
    pub expires_at: String,
}

/// Logs of a database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct DatabaseLogs {
    /// Complete logs.
    pub big: Option<String>,
    /// Condensed logs.
    pub small: Option<String>,
    /// URL to view the logs online, if available.
    pub url: Option<String>,
}

#[derive(Deserialize)]
struct PresetsData {
    #[serde(default)]
    databases: Vec<DatabasePreset>,
}

#[derive(Deserialize)]
struct IdResponse {
    id: String,
}

#[derive(Deserialize)]
struct DatabaseResponse {
    database: Database,
}

#[derive(Deserialize)]
struct UpdateResponse {
    #[serde(default)]
    rebuilding: bool,
    #[serde(default)]
    data: UpdatedDatabase,
}

#[derive(Deserialize)]
struct LogsApp {
    #[serde(default)]
    terminal: DatabaseLogs,
}

#[derive(Deserialize)]
struct LogsResponse {
    app: LogsApp,
}

#[derive(Serialize)]
struct BackupBody {
    gzip: bool,
}

fn db(id: &str) -> Result<&str, Error> {
    if id.trim().is_empty() {
        Err(Error::invalid("the database ID must not be empty"))
    } else {
        Ok(id)
    }
}

fn check_ram(ram: u32) -> Result<(), Error> {
    if ram < MIN_RAM_MB {
        Err(Error::invalid(format!(
            "databases need at least {MIN_RAM_MB} MB of RAM"
        )))
    } else {
        Ok(())
    }
}

impl Discloud {
    /// Lists the available database engines. `GET /database/presets`
    pub async fn get_database_presets(&self) -> Result<Vec<DatabasePreset>, Error> {
        let r: DataResponse<PresetsData> = self.get(&["database", "presets"]).await?;
        Ok(r.data.databases)
    }

    /// Suggests random names for a new database, optionally for an engine.
    /// `GET /database/presets/random-names`
    pub async fn get_random_database_names(
        &self,
        engine: Option<&str>,
    ) -> Result<DatabaseNames, Error> {
        let query: Vec<_> = engine.map(|e| ("type", e.to_owned())).into_iter().collect();
        let r: DataResponse<_> = self
            .execute(self.request(
                Method::GET,
                &["database", "presets", "random-names"],
                &query,
            )?)
            .await?;
        Ok(r.data)
    }

    /// Lists your databases. `GET /database`
    pub async fn get_databases(&self) -> Result<Vec<DatabaseSummary>, Error> {
        let r: DataResponse<_> = self.get(&["database"]).await?;
        Ok(r.data)
    }

    /// Starts provisioning a database and returns its ID. `POST /database`
    pub async fn create_database(&self, new: &CreateDatabase) -> Result<String, Error> {
        if new.display_name.trim().is_empty() {
            return Err(Error::invalid("the display name must not be empty"));
        }
        if new.db.kind.is_empty() || new.db.version.is_empty() {
            return Err(Error::invalid("the database type and version are required"));
        }
        check_ram(new.total_ram)?;
        let r: IdResponse = self.send_json(Method::POST, &["database"], new).await?;
        Ok(r.id)
    }

    /// Returns a database. `GET /database/{dbID}`
    pub async fn get_database(&self, id: &str) -> Result<Database, Error> {
        let r: DatabaseResponse = self.get(&["database", db(id)?]).await?;
        Ok(r.database)
    }

    /// Updates a database. `PATCH /database/{dbID}`
    pub async fn update_database(
        &self,
        id: &str,
        update: &UpdateDatabase,
    ) -> Result<UpdatedDatabase, Error> {
        if let Some(ram) = update.total_ram {
            if !(MIN_RAM_MB..=16384).contains(&ram) {
                return Err(Error::invalid(
                    "`total_ram` must be between 100 and 16384 MB",
                ));
            }
        }
        if update.hostname.as_ref().is_some_and(|h| h.len() > 25) {
            return Err(Error::invalid(
                "the hostname must have at most 25 characters",
            ));
        }
        let r: UpdateResponse = self
            .send_json(Method::PATCH, &["database", db(id)?], update)
            .await?;
        let mut updated = r.data;
        updated.rebuilding = r.rebuilding;
        Ok(updated)
    }

    /// Deletes a database. `DELETE /database/{dbID}`
    pub async fn delete_database(&self, id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["database", db(id)?])
            .await
            .map(drop)
    }

    /// Starts a database. `PUT /database/{dbID}/start`
    pub async fn start_database(&self, id: &str) -> Result<(), Error> {
        self.db_action(id, "start").await
    }

    /// Stops a database. `PUT /database/{dbID}/stop`
    pub async fn stop_database(&self, id: &str) -> Result<(), Error> {
        self.db_action(id, "stop").await
    }

    /// Restarts a database. `PUT /database/{dbID}/restart`
    pub async fn restart_database(&self, id: &str) -> Result<(), Error> {
        self.db_action(id, "restart").await
    }

    async fn db_action(&self, id: &str, action: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::PUT, &["database", db(id)?, action])
            .await
            .map(drop)
    }

    /// Creates a backup of a database. `POST /database/{dbID}/backup`
    pub async fn backup_database(&self, id: &str, gzip: bool) -> Result<NewDatabaseBackup, Error> {
        let r: DataResponse<_> = self
            .send_json(
                Method::POST,
                &["database", db(id)?, "backup"],
                &BackupBody { gzip },
            )
            .await?;
        Ok(r.data)
    }

    /// Lists the backups of a database. `GET /database/{dbID}/backups`
    pub async fn get_database_backups(
        &self,
        id: &str,
        query: &DatabaseBackupQuery,
    ) -> Result<DatabaseBackupPage, Error> {
        let mut params = Vec::new();
        if let Some(limit) = query.limit {
            if !(1..=1000).contains(&limit) {
                return Err(Error::invalid("`limit` must be between 1 and 1000"));
            }
            params.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = &query.cursor {
            params.push(("cursor", cursor.clone()));
        }
        if let Some(detail) = query.detail {
            let detail = match detail {
                DatabaseBackupDetail::Minimal => "minimal",
                DatabaseBackupDetail::Full => "full",
            };
            params.push(("detail", detail.to_owned()));
        }
        let r: DataResponse<_> = self
            .execute(self.request(Method::GET, &["database", db(id)?, "backups"], &params)?)
            .await?;
        Ok(r.data)
    }

    /// Returns a temporary download link for a backup, valid for
    /// `expires_in_days` (1 to 14) days. `GET /database/{dbID}/backups/{backupId}/download`
    pub async fn get_database_backup_download(
        &self,
        id: &str,
        backup_id: &str,
        expires_in_days: Option<u8>,
    ) -> Result<DatabaseDownload, Error> {
        let mut params = Vec::new();
        if let Some(days) = expires_in_days {
            if !(1..=14).contains(&days) {
                return Err(Error::invalid("`expires_in_days` must be between 1 and 14"));
            }
            params.push(("expiresInDays", days.to_string()));
        }
        let r: DataResponse<_> = self
            .execute(self.request(
                Method::GET,
                &["database", db(id)?, "backups", backup_id, "download"],
                &params,
            )?)
            .await?;
        Ok(r.data)
    }

    /// Restores a backup into a new database and returns its ID.
    /// `POST /database/{dbID}/restore`
    pub async fn restore_database(
        &self,
        id: &str,
        restore: &RestoreDatabase,
    ) -> Result<String, Error> {
        if restore.backup_id.trim().is_empty() {
            return Err(Error::invalid("the backup ID must not be empty"));
        }
        if !restore.vars.is_empty() && restore.vars_encrypted.is_some() {
            return Err(Error::invalid(
                "`vars` and `vars_encrypted` are mutually exclusive",
            ));
        }
        check_ram(restore.total_ram)?;
        let r: IdResponse = self
            .send_json(Method::POST, &["database", db(id)?, "restore"], restore)
            .await?;
        Ok(r.id)
    }

    /// Returns the logs of a database. `GET /database/{dbID}/logs`
    pub async fn get_database_logs(&self, id: &str) -> Result<DatabaseLogs, Error> {
        let r: LogsResponse = self.get(&["database", db(id)?, "logs"]).await?;
        Ok(r.app.terminal)
    }

    /// Clears the logs of a database. `DELETE /database/{dbID}/logs`
    pub async fn delete_database_logs(&self, id: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["database", db(id)?, "logs"])
            .await
            .map(drop)
    }
}
