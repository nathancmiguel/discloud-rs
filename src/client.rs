//! The [`Discloud`] client and the shared request pipeline.

use std::{fmt, sync::Arc, time::Duration};

use reqwest::{
    Client, Method, RequestBuilder, StatusCode, Url,
    header::{HeaderName, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};
use tracing::{debug, trace};

use crate::error::{Error, WireStatus};

/// Base URL of the public Discloud API (version 2).
pub const DEFAULT_BASE_URL: &str = "https://api.discloud.app/v2/";

const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const API_TOKEN_HEADER: HeaderName = HeaderName::from_static("api-token");
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Uploads and commits can take a long time before the API answers.
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(100);

/// Asynchronous client for the Discloud API.
///
/// The client is cheap to clone: clones share the same connection pool and
/// configuration, so create one and reuse it.
///
/// ```no_run
/// # async fn run() -> Result<(), discloud_rs::Error> {
/// let client = discloud_rs::Discloud::new("API_TOKEN");
/// let user = client.get_user_info().await?;
/// println!("{} MB of RAM used", user.used_ram);
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Discloud {
    inner: Arc<Inner>,
}

struct Inner {
    http: Client,
    token: String,
    base_url: Url,
}

impl fmt::Debug for Discloud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Discloud")
            .field("base_url", &self.inner.base_url.as_str())
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl Discloud {
    /// Creates a client with the default configuration.
    ///
    /// # Panics
    ///
    /// Panics if the TLS backend cannot be initialized, like
    /// [`reqwest::Client::new`]. Use [`Discloud::builder`] to handle that case.
    pub fn new(token: impl Into<String>) -> Self {
        Self::builder(token)
            .build()
            .expect("failed to initialize the HTTP client")
    }

    /// Returns a builder to customize the client.
    pub fn builder(token: impl Into<String>) -> DiscloudBuilder {
        DiscloudBuilder {
            token: token.into(),
            base_url: None,
            user_agent: None,
            timeout: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            read_timeout: DEFAULT_READ_TIMEOUT,
            http: None,
        }
    }

    /// Returns the base URL requests are sent to.
    pub fn base_url(&self) -> &str {
        self.inner.base_url.as_str()
    }

    fn url(&self, segments: &[&str], query: &[(&str, String)]) -> Result<Url, Error> {
        let mut url = self.inner.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|()| Error::invalid("the base URL cannot have path segments"))?;
            path.pop_if_empty();
            for segment in segments {
                // `Url` percent-encodes `/`, `?` and `#`, but silently drops
                // `.` and `..`, so reject those (and empty values) explicitly.
                if segment.is_empty() || *segment == "." || *segment == ".." {
                    return Err(Error::invalid(format!(
                        "invalid path parameter {segment:?}"
                    )));
                }
                path.push(segment);
            }
        }
        if !query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(query.iter().map(|(k, v)| (*k, v.as_str())));
        }
        Ok(url)
    }

    /// Starts a request to `segments` (joined and percent-encoded) with the
    /// authentication header set.
    pub(crate) fn request(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, String)],
    ) -> Result<RequestBuilder, Error> {
        let url = self.url(segments, query)?;
        let mut token = HeaderValue::from_str(&self.inner.token).map_err(|_| {
            Error::invalid("the API token contains characters not allowed in HTTP headers")
        })?;
        token.set_sensitive(true);
        Ok(self
            .inner
            .http
            .request(method, url)
            .header(API_TOKEN_HEADER, token))
    }

    /// Sends a request and decodes the body, returning the HTTP status too.
    pub(crate) async fn execute_with_status<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<(StatusCode, T), Error> {
        let request = request.build()?;
        let method = request.method().clone();
        let path = request.url().path().to_owned();
        trace!(%method, path, "sending Discloud API request");

        let response = self.inner.http.execute(request).await?;
        let status = response.status();
        let body = response.bytes().await?;
        // Bodies are never logged: they can contain secrets such as
        // environment variables or database credentials.
        debug!(%method, path, status = status.as_u16(), bytes = body.len(), "Discloud API response");

        let wire: WireStatus = serde_json::from_slice(&body).unwrap_or_default();
        if !status.is_success() {
            return Err(Error::from_status(status, wire));
        }
        if wire.status.as_deref() == Some("error") {
            return Err(Error::Api(wire.into_api_error(status)));
        }
        let value = serde_json::from_slice(&body).map_err(Error::Decode)?;
        Ok((status, value))
    }

    pub(crate) async fn execute<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, Error> {
        self.execute_with_status(request).await.map(|(_, v)| v)
    }

    /// Sends a request without a body.
    pub(crate) async fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        segments: &[&str],
    ) -> Result<T, Error> {
        self.execute(self.request(method, segments, &[])?).await
    }

    /// Sends a request with a JSON body.
    pub(crate) async fn send_json<T, B>(
        &self,
        method: Method,
        segments: &[&str],
        body: &B,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.execute(self.request(method, segments, &[])?.json(body))
            .await
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T, Error> {
        self.send(Method::GET, segments).await
    }
}

/// Builder for a customized [`Discloud`] client.
#[derive(Debug)]
#[must_use]
pub struct DiscloudBuilder {
    token: String,
    base_url: Option<String>,
    user_agent: Option<String>,
    timeout: Option<Duration>,
    connect_timeout: Duration,
    read_timeout: Duration,
    http: Option<Client>,
}

impl DiscloudBuilder {
    /// Overrides the API base URL (defaults to [`DEFAULT_BASE_URL`]).
    /// Useful for proxies and tests.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Overrides the `User-Agent` header (defaults to `discloud-rs/<version>`).
    /// Ignored when [`http_client`](Self::http_client) is set.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sets a total timeout per request (none by default).
    /// Ignored when [`http_client`](Self::http_client) is set.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the connection timeout (10 seconds by default).
    /// Ignored when [`http_client`](Self::http_client) is set.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets the timeout between two reads of a response (100 seconds by
    /// default, because uploads and commits answer slowly).
    /// Ignored when [`http_client`](Self::http_client) is set.
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Uses an existing [`reqwest::Client`], sharing its connection pool.
    /// The timeout and user-agent settings of this builder are then ignored.
    pub fn http_client(mut self, client: Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] for an invalid base URL or token and
    /// [`Error::Http`] if the HTTP client cannot be created.
    pub fn build(self) -> Result<Discloud, Error> {
        if HeaderValue::from_str(&self.token).is_err() {
            return Err(Error::invalid(
                "the API token contains characters not allowed in HTTP headers",
            ));
        }

        let base = self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
        let base_url = Url::parse(base)
            .map_err(|e| Error::invalid(format!("invalid base URL {base:?}: {e}")))?;
        if base_url.cannot_be_a_base() {
            return Err(Error::invalid(format!("invalid base URL {base:?}")));
        }

        let http = match self.http {
            Some(client) => client,
            None => {
                let mut builder = Client::builder()
                    .user_agent(self.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT))
                    .connect_timeout(self.connect_timeout)
                    .read_timeout(self.read_timeout);
                if let Some(timeout) = self.timeout {
                    builder = builder.timeout(timeout);
                }
                builder.build()?
            }
        };

        Ok(Discloud {
            inner: Arc::new(Inner {
                http,
                token: self.token,
                base_url,
            }),
        })
    }
}

/// Whether a route operates on the caller's own applications (`/app/...`) or
/// on applications the caller moderates (`/team/...`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    App,
    Team,
}

impl Scope {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Team => "team",
        }
    }
}

/// Rejects the special `all` identifier where a single application is
/// expected, pointing at the method that handles every application.
pub(crate) fn single_id<'a>(id: &'a str, all_variant: &str) -> Result<&'a str, Error> {
    if id.eq_ignore_ascii_case("all") {
        return Err(Error::invalid(format!(
            "\"all\" is not a valid application ID here; use `{all_variant}` instead"
        )));
    }
    app_id(id)
}

/// Validates an application ID for routes that have no bulk variant.
pub(crate) fn app_id(id: &str) -> Result<&str, Error> {
    if id.eq_ignore_ascii_case("all") {
        return Err(Error::invalid(
            "this operation does not accept \"all\"; pass a single application ID",
        ));
    }
    if id.trim().is_empty() {
        return Err(Error::invalid("the application ID must not be empty"));
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(base: &str) -> Discloud {
        Discloud::builder("token").base_url(base).build().unwrap()
    }

    #[test]
    fn urls_are_joined_and_encoded() {
        let c = client(DEFAULT_BASE_URL);
        let url = c.url(&["app", "a/b?c#d"], &[]).unwrap();
        assert_eq!(
            url.as_str(),
            "https://api.discloud.app/v2/app/a%2Fb%3Fc%23d"
        );

        let c = client("http://127.0.0.1:1234");
        let url = c.url(&["snapshot"], &[("page", "2".into())]).unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:1234/snapshot?page=2");
    }

    #[test]
    fn dot_segments_are_rejected() {
        let c = client(DEFAULT_BASE_URL);
        for bad in ["", ".", ".."] {
            assert!(matches!(
                c.url(&["app", bad], &[]),
                Err(Error::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn all_is_rejected_for_single_routes() {
        assert!(single_id("all", "x").is_err());
        assert!(single_id(" ", "x").is_err());
        assert_eq!(single_id("123", "x").unwrap(), "123");
    }

    #[test]
    fn invalid_token_is_rejected() {
        assert!(Discloud::builder("bad\ntoken").build().is_err());
    }

    #[test]
    fn debug_redacts_token() {
        let c = Discloud::new("secret-token");
        assert!(!format!("{c:?}").contains("secret-token"));
    }
}
