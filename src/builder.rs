use std::borrow::Cow;

use crate::{config::Config, error, Auditor, Result};

use snafu::*;

const GITHUB_AUTH_ENV_KEY: &str = "GITHUB_AUTH_KEY";

/// A builder struct for the `Auditor`. Allows you to configure GitHub
/// organisation, underlying http client, audit configuration, and
/// authentication token.
pub struct AuditorBuilder<'a> {
    auth_key: Option<String>,
    client: Option<Cow<'a, reqwest::Client>>,
    config: Config,
    org: String,
}

impl<'a> AuditorBuilder<'a> {
    /// Creates a default `AuditorBuilder`.
    /// # Example
    /// ```
    /// use gh_auditor::{Auditor, AuditorBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let auditor: Auditor = AuditorBuilder::new("rust-lang").finish()?;
    /// # }
    /// ```
    pub fn new<I: Into<String>>(org: I) -> Self {
        Self {
            org: org.into(),
            client: None,
            config: Config::default(),
            auth_key: None,
        }
    }

    /// Set the authentication key used for GitHub requests. `Auditor` requires
    /// at least `admin:read` access on the organisation.  (Default:
    /// `$GITHUB_AUTH_KEY` environment variable)
    pub fn auth_key<I: Into<String>>(mut self, auth_key: I) -> Self {
        self.auth_key = Some(auth_key.into());
        self
    }

    /// Sets the http client, if desired. (Default: `reqwest::Client::new()`)
    pub fn client<I: Into<Cow<'a, reqwest::Client>>>(mut self, client: I) -> Self {
        self.client = Some(client.into());
        self
    }

    /// Sets the initial configuration. See `Config` for information on
    /// audit configuration. (Default: `Config::default()`)
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Consume `AuditorBuilder` and construct `Auditor` with configuration or
    /// default values, and query for organisation information.
    /// # Errors
    /// If no authentication key was not provided or invalid.
    pub fn finish(self) -> Result<Auditor<'a>> {
        let auth_key = self
            .auth_key
            .or_else(|| std::env::var(GITHUB_AUTH_ENV_KEY).ok())
            .context(error::NoAuthKey)?;

        let client = self
            .client
            .unwrap_or_else(|| Cow::Owned(reqwest::Client::new()));

        let organisation = client
            .get(&format!("https://api.github.com/orgs/{}", self.org))
            .bearer_auth(&auth_key)
            .send()
            .context(error::Http)?
            .json()
            .context(error::Http)?;

        Ok(Auditor {
            auth_key,
            client,
            config: self.config,
            organisation,
            has_run_audit: false,
        })
    }
}
