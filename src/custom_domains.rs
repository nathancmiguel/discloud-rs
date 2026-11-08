//! Custom domains pointed at your applications (`/customdomain`).

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Discloud, Error, client::app_id, wire::MessageResponse};

/// Application a custom domain points to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct CustomDomainApp {
    /// Application ID.
    pub id: String,
    /// Application name.
    pub name: String,
    /// Whether the application is online.
    pub online: Option<bool>,
    /// Application type (`0` = bot, `1` = site).
    pub r#type: Option<i32>,
}

/// A DNS record to create for a custom domain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct DnsInstruction {
    /// Record type (e.g. `CNAME`).
    pub r#type: String,
    /// Record name.
    pub name: String,
    /// Record content.
    pub content: String,
}

/// A custom domain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct CustomDomain {
    /// Domain name.
    pub id: String,
    /// DNS verification status (e.g. `active`, `pending_check`).
    #[serde(rename = "dns_status")]
    pub dns_status: Option<String>,
    /// Creation time in milliseconds since the Unix epoch.
    pub date: Option<u64>,
    /// Whether the domain uses the legacy configuration.
    pub is_legacy: Option<bool>,
    /// Whether the DNS configuration is valid (verification only).
    pub valid: Option<bool>,
    /// Verification message.
    pub message: Option<String>,
    /// Target application.
    pub app: Option<CustomDomainApp>,
    /// DNS records to create.
    pub instructions: Vec<DnsInstruction>,
}

/// Result of [`Discloud::edit_custom_domain`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct CustomDomainMigration {
    /// Whether the domain was moved to the new application.
    pub migrated: bool,
    /// Domain name.
    pub domain: String,
    /// API message.
    pub message: String,
}

#[derive(Deserialize)]
struct ListResponse {
    #[serde(default)]
    domains: Vec<CustomDomain>,
}

#[derive(Deserialize)]
struct OneResponse {
    domain: CustomDomain,
}

#[derive(Serialize)]
struct CreateBody<'a> {
    #[serde(rename = "domainName")]
    domain_name: &'a str,
    #[serde(rename = "appID")]
    app_id: &'a str,
}

#[derive(Serialize)]
struct EditBody<'a> {
    #[serde(rename = "newAppID")]
    new_app_id: &'a str,
}

fn domain(name: &str) -> Result<&str, Error> {
    if name.trim().is_empty() || name.contains(char::is_whitespace) {
        return Err(Error::invalid(format!("invalid domain name {name:?}")));
    }
    // These names collide with fixed routes and cannot be addressed.
    if matches!(name, "all" | "create") {
        return Err(Error::invalid(format!("{name:?} is a reserved route name")));
    }
    Ok(name)
}

impl Discloud {
    /// Lists your custom domains. `GET /customdomain/all`
    pub async fn get_custom_domains(&self) -> Result<Vec<CustomDomain>, Error> {
        let r: ListResponse = self.get(&["customdomain", "all"]).await?;
        Ok(r.domains)
    }

    /// Returns one custom domain. `GET /customdomain/{domainName}`
    pub async fn get_custom_domain(&self, name: &str) -> Result<CustomDomain, Error> {
        let r: OneResponse = self.get(&["customdomain", domain(name)?]).await?;
        Ok(r.domain)
    }

    /// Links a custom domain to a website application. The returned domain
    /// lists the DNS records to create. `POST /customdomain/create`
    pub async fn create_custom_domain(&self, name: &str, app: &str) -> Result<CustomDomain, Error> {
        let body = CreateBody {
            domain_name: domain(name)?,
            app_id: app_id(app)?,
        };
        let r: OneResponse = self
            .send_json(Method::POST, &["customdomain", "create"], &body)
            .await?;
        Ok(r.domain)
    }

    /// Moves a custom domain to another application.
    /// `PUT /customdomain/{domainName}/edit`
    pub async fn edit_custom_domain(
        &self,
        name: &str,
        new_app: &str,
    ) -> Result<CustomDomainMigration, Error> {
        let body = EditBody {
            new_app_id: app_id(new_app)?,
        };
        self.send_json(Method::PUT, &["customdomain", domain(name)?, "edit"], &body)
            .await
    }

    /// Removes a custom domain. `DELETE /customdomain/{domainName}/remove`
    pub async fn remove_custom_domain(&self, name: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["customdomain", domain(name)?, "remove"])
            .await
            .map(drop)
    }

    /// Checks the DNS configuration of a custom domain.
    /// `GET /customdomain/{domainName}/verify`
    pub async fn verify_custom_domain(&self, name: &str) -> Result<CustomDomain, Error> {
        let r: OneResponse = self.get(&["customdomain", domain(name)?, "verify"]).await?;
        Ok(r.domain)
    }
}
