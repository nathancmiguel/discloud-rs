//! `*.discloud.app` subdomains (`/subdomain`). Requires the Platinum plan or higher.

use reqwest::Method;
use serde::Deserialize;

use crate::{Discloud, Error, wire::MessageResponse};

/// A registered subdomain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct Subdomain {
    /// Subdomain name (without `.discloud.app`).
    pub id: String,
    /// Owner's user ID.
    #[serde(rename = "userID")]
    pub user_id: String,
    /// `1` = available, `2` = in use by an application.
    pub status: u8,
    /// Creation time in milliseconds since the Unix epoch.
    pub date: u64,
}

impl Subdomain {
    /// Whether an application is using the subdomain.
    pub fn in_use(&self) -> bool {
        self.status == 2
    }
}

#[derive(Deserialize)]
struct ListResponse {
    #[serde(default)]
    subdomains: Vec<Subdomain>,
}

#[derive(Deserialize)]
struct OneResponse {
    subdomain: Subdomain,
}

/// Checks the API's subdomain rule: `^[a-z0-9-]{2,20}$`.
fn validate(name: &str) -> Result<&str, Error> {
    let valid = (2..=20).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if valid {
        Ok(name)
    } else {
        Err(Error::invalid(format!(
            "invalid subdomain {name:?}: use 2 to 20 lowercase letters, digits or hyphens"
        )))
    }
}

impl Discloud {
    /// Lists your subdomains. `GET /subdomain/all`
    pub async fn get_subdomains(&self) -> Result<Vec<Subdomain>, Error> {
        let r: ListResponse = self.get(&["subdomain", "all"]).await?;
        Ok(r.subdomains)
    }

    /// Returns one subdomain. `GET /subdomain/{subdomainName}`
    pub async fn get_subdomain(&self, name: &str) -> Result<Subdomain, Error> {
        let r: OneResponse = self.get(&["subdomain", validate(name)?]).await?;
        Ok(r.subdomain)
    }

    /// Registers a subdomain. `POST /subdomain/{subdomainName}`
    pub async fn create_subdomain(&self, name: &str) -> Result<Subdomain, Error> {
        let r: OneResponse = self
            .send(Method::POST, &["subdomain", validate(name)?])
            .await?;
        Ok(r.subdomain)
    }

    /// Deletes a subdomain that no application uses. `DELETE /subdomain/{subdomainName}`
    pub async fn delete_subdomain(&self, name: &str) -> Result<(), Error> {
        self.send::<MessageResponse>(Method::DELETE, &["subdomain", validate(name)?])
            .await
            .map(drop)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn subdomain_rule() {
        assert!(super::validate("my-bot-01").is_ok());
        for bad in ["a", "UPPER", "with_underscore", "this-name-is-way-too-long"] {
            assert!(super::validate(bad).is_err(), "{bad}");
        }
    }
}
