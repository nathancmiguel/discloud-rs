//! An async Rust client for the [Discloud](https://discloud.com) hosting API.
//!
//! Create one [`Discloud`] client with your API token and call its methods.
//! Each method maps to one API route, which is noted in its documentation.
//!
//! ```no_run
//! use discloud_rs::{AppStartError, Discloud};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Discloud::new(std::env::var("DISCLOUD_TOKEN")?);
//!
//! for app in client.get_all_apps().await? {
//!     println!("{} ({}): online = {}", app.name, app.id, app.online);
//! }
//!
//! match client.start_app("APP_ID").await {
//!     Ok(status) => println!("started, online = {}", status.online),
//!     Err(AppStartError::AlreadyOnline) => println!("already online"),
//!     Err(e) => return Err(e.into()),
//! }
//! # Ok(()) }
//! ```
//!
//! # Errors
//!
//! Every method returns [`Error`] (or a small wrapper such as
//! [`AppStartError`] for expected outcomes). API failures carry an
//! [`ApiError`] with the HTTP status, the API message and its error code.
//!
//! # Features
//!
//! - `rustls` *(default)*: TLS through rustls.
//! - `native-tls`: TLS through the platform's native stack.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod apps;
mod client;
mod custom_domains;
mod databases;
mod env;
mod error;
mod explorer;
mod moderators;
mod projects;
mod snapshots;
mod subdomains;
mod team_apps;
mod teams;
mod upload;
mod user;
mod wire;

pub use apps::{
    App, AppBackup, AppCommit, AppDeleteAll, AppLogs, AppLogsTerminal, AppProfileUpdate,
    AppRestartAll, AppStartAll, AppStartStatus, AppStatus, AppStopAll, AppType, AptPackage,
    ConsoleOutput, ExecOutput, LastCommit, NetIO, SyncGit,
};
pub use client::{DEFAULT_BASE_URL, Discloud, DiscloudBuilder};
pub use custom_domains::{CustomDomain, CustomDomainApp, CustomDomainMigration, DnsInstruction};
pub use databases::{
    CreateDatabase, Database, DatabaseBackup, DatabaseBackupDetail, DatabaseBackupPage,
    DatabaseBackupQuery, DatabaseDownload, DatabaseEngine, DatabaseLogs, DatabaseNames,
    DatabaseOwner, DatabasePreset, DatabasePresetVars, DatabaseSpec, DatabaseSummary,
    NewDatabaseBackup, RestoreDatabase, UpdateDatabase, UpdatedDatabase,
};
pub use env::EnvVars;
pub use error::{ApiError, AppRamError, AppStartError, AppStopError, Error};
pub use explorer::{DirectoryListing, FileContent, FileItem, FileKind};
#[allow(deprecated)]
pub use moderators::{APITeamMember, TeamManager};
pub use moderators::{TeamMember, TeamPerms};
pub use projects::{
    AddAppToProject, CreateProject, Project, ProjectApp, ProjectAppLink, ProjectDetails,
    ProjectMigration, ProjectSummary, UpdateProject,
};
pub use snapshots::{
    CreatedSnapshot, CreatedSnapshotVersion, SnapshotDownload, SnapshotList, SnapshotQuery,
    SnapshotVersion,
};
pub use subdomains::Subdomain;
pub use team_apps::{TeamApp, TeamAppOwner};
pub use teams::{
    CreateTeam, RolePermissions, Team, TeamDetails, TeamMemberInfo, TeamProject, TeamsApp,
    UpdateTeam,
};
pub use upload::UploadApp;
pub use user::{Locale, PlanTimeLeft, User, VscodeAppStatus, VscodeUser};

/// Compiles the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
