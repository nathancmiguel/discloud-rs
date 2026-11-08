//! The authenticated user (`/user`, `/locale`, `/vscode`).

use std::{fmt, str::FromStr};

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Discloud, Error, apps::AppType, wire::MessageResponse};

/// Time left in the current plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct PlanTimeLeft {
    /// Days left.
    pub days: i64,
    /// Hours left.
    pub hours: i64,
    /// Minutes left.
    pub minutes: i64,
    /// Seconds left.
    pub seconds: i64,
}

/// Account information of the token owner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct User {
    /// User ID.
    #[serde(rename = "userID")]
    pub user_id: String,
    /// Username, if set.
    pub username: Option<String>,
    /// Avatar URL, if set.
    pub avatar: Option<String>,
    /// Total RAM of the plan, in MB.
    #[serde(rename = "totalRamMb")]
    pub total_ram: u32,
    /// RAM in use, in MB.
    #[serde(rename = "ramUsedMb")]
    pub used_ram: u32,
    /// Owned subdomains.
    pub subdomains: Vec<String>,
    /// Owned custom domains.
    #[serde(rename = "customdomains")]
    pub custom_domains: Vec<String>,
    /// IDs of owned applications.
    pub apps: Vec<String>,
    /// Plan name.
    pub plan: String,
    /// Locale code (e.g. `"en-US"`).
    pub locale: String,
    /// Time left in the current plan.
    #[serde(rename = "lastDataLeft")]
    pub last_data_left: Option<PlanTimeLeft>,
    /// End date of the plan, or `"Subscription"` for subscription plans.
    #[serde(rename = "planDataEnd")]
    pub plan_date_end: String,
}

/// Summary of an application returned by [`Discloud::get_vscode_user`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct VscodeAppStatus {
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
    /// Exit code of the last run.
    pub exit_code: Option<i32>,
    /// Programming language.
    pub lang: String,
    /// Entry point file.
    pub main_file: String,
    /// Avatar URL.
    #[serde(rename = "avatarURL")]
    pub avatar_url: Option<String>,
    /// Whether the application restarts automatically.
    pub auto_restart: bool,
    /// Installed APT packages.
    pub apts: Vec<String>,
    /// Cluster hosting the application.
    pub cluster_name: Option<String>,
    /// Version from `discloud.config`.
    pub version: Option<String>,
}

/// Extended account information used by the VS Code extension.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct VscodeUser {
    /// User ID.
    #[serde(rename = "userID")]
    pub user_id: String,
    /// Username, if set.
    pub username: Option<String>,
    /// Avatar URL, if set.
    pub avatar: Option<String>,
    /// Plan name.
    pub plan: String,
    /// Locale code.
    pub locale: String,
    /// Total RAM of the plan, in MB.
    #[serde(rename = "totalRamMb")]
    pub total_ram: u32,
    /// RAM in use, in MB.
    #[serde(rename = "ramUsedMb")]
    pub used_ram: u32,
    /// IDs of owned applications.
    pub apps: Vec<String>,
    /// IDs of moderated applications.
    #[serde(rename = "appsTeam")]
    pub apps_team: Vec<String>,
    /// Owned subdomains.
    pub subdomains: Vec<String>,
    /// Owned custom domains.
    #[serde(rename = "customdomains")]
    pub custom_domains: Vec<String>,
    /// Status of each owned application.
    #[serde(rename = "appsStatus")]
    pub apps_status: Vec<VscodeAppStatus>,
}

/// Language of the API messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Locale {
    /// Brazilian Portuguese (`pt-BR`).
    #[serde(rename = "pt-BR")]
    PtBR,
    /// American English (`en-US`).
    #[serde(rename = "en-US")]
    EnUS,
}

impl Locale {
    /// The locale code used by the API.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PtBR => "pt-BR",
            Self::EnUS => "en-US",
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Locale {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pt-BR" => Ok(Self::PtBR),
            "en-US" => Ok(Self::EnUS),
            _ => Err(Error::invalid(format!(
                "unsupported locale {s:?}; expected \"pt-BR\" or \"en-US\""
            ))),
        }
    }
}

#[derive(Deserialize)]
struct UserResponse<T> {
    user: T,
}

impl Discloud {
    /// Returns the account of the token owner. `GET /user`
    pub async fn get_user_info(&self) -> Result<User, Error> {
        let r: UserResponse<_> = self.get(&["user"]).await?;
        Ok(r.user)
    }

    /// Returns extended account information. `GET /vscode`
    pub async fn get_vscode_user(&self) -> Result<VscodeUser, Error> {
        let r: UserResponse<_> = self.get(&["vscode"]).await?;
        Ok(r.user)
    }

    /// Changes the language of the API messages. `PUT /locale/{locale}`
    pub async fn set_locale(&self, locale: Locale) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::PUT, &["locale", locale.as_str()])
            .await
            .map(drop)
    }
}
