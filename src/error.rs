//! Error types returned by the client.

use std::fmt;

use reqwest::StatusCode;
use serde::Deserialize;

/// Details of an error reported by the Discloud API.
///
/// The API reports failures either with a non-2xx HTTP status or, for several
/// routes, with HTTP 200 and `"status": "error"` in the body. Both cases are
/// normalized into this structure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApiError {
    /// HTTP status code of the response (may be `200` for in-body errors).
    pub http_status: u16,
    /// Machine-readable error code, when the API provides one
    /// (for example `SNAPSHOT:APP_BUSY` or `CLUSTER_MISMATCH`).
    pub code: Option<String>,
    /// Numeric `statusCode` field from the body, when present.
    pub status_code: Option<i64>,
    /// Human-readable message from the API, or a generic description of the
    /// HTTP status when the body has no message.
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (HTTP {}", self.message, self.http_status)?;
        if let Some(code) = &self.code {
            write!(f, ", code {code}")?;
        }
        f.write_str(")")
    }
}

/// The error type of every fallible client operation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The request could not be sent or the response could not be read
    /// (connection failure, timeout, TLS error, ...).
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
    /// HTTP 401: the API token is missing, invalid or blocked.
    #[error("unauthorized: {0}")]
    Unauthorized(ApiError),
    /// HTTP 403: the token is valid but lacks permission or the plan does not
    /// include the feature.
    #[error("forbidden: {0}")]
    Forbidden(ApiError),
    /// HTTP 404: the requested resource does not exist.
    #[error("not found: {0}")]
    NotFound(ApiError),
    /// HTTP 409: the request conflicts with the current state of the resource.
    #[error("conflict: {0}")]
    Conflict(ApiError),
    /// HTTP 400: the API rejected the request parameters.
    #[error("bad request: {0}")]
    BadRequest(ApiError),
    /// HTTP 429: too many requests, or another operation on the same resource
    /// is still running.
    #[error("rate limited: {0}")]
    RateLimited(ApiError),
    /// HTTP 5xx: the API failed to process the request.
    #[error("server error: {0}")]
    Server(ApiError),
    /// Any other API-reported failure, including HTTP 200 responses whose body
    /// has `"status": "error"`.
    #[error("API error: {0}")]
    Api(ApiError),
    /// The response body did not match the expected schema.
    #[error("failed to decode the API response: {0}")]
    Decode(#[source] serde_json::Error),
    /// A local file could not be read.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The arguments were rejected before sending the request.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl Error {
    /// Returns the API error details, if this error was reported by the API.
    pub fn api_error(&self) -> Option<&ApiError> {
        match self {
            Self::Unauthorized(e)
            | Self::Forbidden(e)
            | Self::NotFound(e)
            | Self::Conflict(e)
            | Self::BadRequest(e)
            | Self::RateLimited(e)
            | Self::Server(e)
            | Self::Api(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the HTTP status associated with the error, if any.
    pub fn http_status(&self) -> Option<u16> {
        match self {
            Self::Http(e) => e.status().map(|s| s.as_u16()),
            _ => self.api_error().map(|e| e.http_status),
        }
    }

    pub(crate) fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Builds the error for a non-success HTTP status.
    pub(crate) fn from_status(status: StatusCode, body: WireStatus) -> Self {
        let err = body.into_api_error(status);
        match status.as_u16() {
            400 => Self::BadRequest(err),
            401 => Self::Unauthorized(err),
            403 => Self::Forbidden(err),
            404 => Self::NotFound(err),
            409 => Self::Conflict(err),
            429 => Self::RateLimited(err),
            500..=599 => Self::Server(err),
            _ => Self::Api(err),
        }
    }

    /// Returns `true` for an API error delivered with a 2xx status and
    /// `"status": "error"` in the body.
    pub(crate) fn is_in_body_error(&self) -> bool {
        matches!(self, Self::Api(e) if (200..300).contains(&e.http_status))
    }
}

/// The status fields shared by every Discloud response body.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct WireStatus {
    #[serde(default)]
    pub(crate) status: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default, rename = "statusCode")]
    status_code: Option<i64>,
    #[serde(default)]
    code: Option<serde_json::Value>,
}

impl WireStatus {
    pub(crate) fn into_api_error(self, status: StatusCode) -> ApiError {
        let code = match self.code {
            Some(serde_json::Value::String(s)) => Some(s),
            Some(serde_json::Value::Number(n)) => Some(n.to_string()),
            _ => None,
        };
        let message = self
            .message
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| {
                status
                    .canonical_reason()
                    .unwrap_or("request failed")
                    .to_owned()
            });
        ApiError {
            http_status: status.as_u16(),
            code,
            status_code: self.status_code,
            message,
        }
    }
}

/// Error returned by [`Discloud::start_app`](crate::Discloud::start_app) and
/// [`Discloud::start_team_app`](crate::Discloud::start_team_app).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppStartError {
    /// The application is already online.
    #[error("the application is already online")]
    AlreadyOnline,
    /// Any other failure.
    #[error(transparent)]
    Other(#[from] Error),
}

/// Error returned by [`Discloud::stop_app`](crate::Discloud::stop_app) and
/// [`Discloud::stop_team_app`](crate::Discloud::stop_team_app).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppStopError {
    /// The application is already offline.
    #[error("the application is already offline")]
    AlreadyOffline,
    /// Any other failure.
    #[error(transparent)]
    Other(#[from] Error),
}

/// Error returned by [`Discloud::set_app_ram`](crate::Discloud::set_app_ram) and
/// [`Discloud::set_team_app_ram`](crate::Discloud::set_team_app_ram).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppRamError {
    /// The requested amount is not allowed (not enough free RAM on the plan,
    /// or below the minimum of 100 MB for bots / 512 MB for sites).
    #[error("RAM quantity not allowed: {0}")]
    ForbiddenQuantity(String),
    /// Any other failure.
    #[error(transparent)]
    Other(#[from] Error),
}
