use snafu::{OptionExt, Snafu};
use std::borrow::Cow;

pub type Result<T> = std::result::Result<T, Error>;

const GITHUB_AUTH_ENV_KEY: &str = "GITHUB_AUTH_KEY";

/// The auditor of a GitHub organisation.
pub struct Auditor<'a> {
    /// The GitHub organisation.
    org: String,
    /// The HTTP client.
    client: Cow<'a, surf::Client<surf::http_client::native::NativeClient>>,
    /// The current configuration.
    config: AuditConfig,
    /// The authentication token for GitHub.
    auth_key: String,
}

pub struct AuditorBuilder<'a> {
    org: String,
    client: Option<Cow<'a, surf::Client>>,
    config: AuditConfig,
    auth_key: Option<String>,
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
            config: AuditConfig::default(),
            auth_key: None,
        }
    }

    pub fn auth_key<I: Into<String>>(mut self, auth_key: I) -> Self {
        self.auth_key = auth_key.into();
        self
    }

    pub fn client<I: Into<Cow<'a, surf::Client>>>(mut self, client: I) -> Self {
        self.client = client.into();
        self
    }

    pub fn config(mut self, config: AuditConfig) -> Self {
        self.config = config;
        self
    }

    pub fn finish(mut self) -> Result<Auditor> {
        let auth_key = self
            .auth_key
            .or_else(|| std::env::var(GITHUB_AUTH_ENV_KEY))
            .context(Error::NoAuthKey)?;

        Auditor {
            org: self.org,
            client: self.client.unwrap_or_else(surf::Client::new),
            config: self.config,
            auth_key,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("No authentication key for GitHub provided."))]
    NoAuthKey,
}

/// A struct configuring which audits the `Auditor` should run.
pub struct AuditConfig {
    // Toggles
    /// Warns if the organisation requires 2 factor authenication for all
    /// of it's members. (Default: `true`)
    enforces_2fa: bool,
    /// Warns if an organisation has admin accounts that have commit activity.
    /// (Default: `true`)
    admins_have_no_commit_activity: bool,
    /// Warns if an organisation has repositories that have unprotected master
    /// branches. (Default: `false`)
    all_repos_master_is_protected: bool,

    // Whitelists
    /// Matches a list of installed applications in a organisation against a
    /// whitelist, if provided. Uses the URL slug of the app e.g. `foobar`.
    /// Warns if there are installations other than ones specified **or** There
    /// is a missing installation from an organisation. (Default: `None`)
    installed_app_whitelist: Option<Vec<String>>,
    /// Matches a list of admins in a organisation against a whitelist, if
    /// provided. Uses the username of person's GitHub account (e.g. `bors`).
    /// Warns if there are admins other than ones specified **or** There
    /// are missing admins from an organisation. (Default: `None`)
    admin_whitelist: Option<Vec<String>>,
    /// Matches a list of users in a organisation against a whitelist, if
    /// provided. Uses the username of person's GitHub account (e.g. `bors`).
    /// Warns if there are users other than ones specified **or** There
    /// are missing users from an organisation. (Default: `None`)
    member_whitelist: Option<Vec<String>>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enforces_2fa: true,
            admins_have_no_commit_activity: true,
            all_repos_master_is_protected: false,
            installed_app_whitelist: None,
            admin_whitelist: None,
            member_whitelist: None,
        }
    }
}
