//! GH Auditor: Audit and enforce access permission policy for an organisation.
#![warn(missing_docs)]

mod builder;
mod config;
mod error;

pub use builder::AuditorBuilder;

use std::borrow::Cow;

use hyperx::header::TypedHeaders;
use snafu::{OptionExt, ResultExt};

/// Alias `Result` for convenience.
pub type Result<T> = std::result::Result<T, error::Error>;

/// The auditor of a GitHub organisation.
#[derive(Debug)]
pub struct Auditor<'a> {
    /// The authentication token for GitHub.
    auth_key: String,
    /// The HTTP client.
    client: Cow<'a, reqwest::Client>,
    /// The current configuration.
    config: config::Config,
    /// The GitHub organisation.
    organisation: serde_json::Value,

    has_run_audit: bool,
}

impl<'a> Auditor<'a> {
    /// Perform the audit.
    /// # Errors
    /// If one of the audits has failed.
    pub fn audit(&mut self) -> std::result::Result<(), Vec<error::Error>> {
        self.has_run_audit = false;
        let mut errors = Vec::new();

        macro_rules! try_and_collect_errors {
            ($ex: expr) => {
                if let Err(error) = $ex {
                    errors.push(error);
                }
            };
        }

        try_and_collect_errors!(self.audit_2fa());
        try_and_collect_errors!(self.audit_admin_commit_activity());

        if !self.has_run_audit {
            errors.push(error::Error::Audit {
                kind: error::AuditError::NoAuditsRan,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Audit that 2fa is enforced for the organisation.
    fn audit_2fa(&mut self) -> Result<()> {
        if self.config.enforces_2fa {
            self.mark_audit("2 Factor Authenication");
            let enabled = self
                .organisation
                .get("two_factor_requirement_enabled")
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false);

            if enabled {
                log::info!("✅ 2 Factor Authenication required for members");
            } else {
                return Err(error::Error::Audit {
                    kind: error::AuditError::Disabled2Fa,
                });
            }
        }

        Ok(())
    }

    fn audit_admin_commit_activity(&mut self) -> Result<()> {
        if self.config.admins_have_no_commit_activity {
            self.mark_audit("Admin Commit Activity");

            let members_url = self
                .organisation
                .get("members_url")
                .and_then(serde_json::Value::as_str)
                .context(error::MissingGitHubData)?
                .replace("{/member}", "");

            let members = self
                .client
                .get(&members_url)
                .bearer_auth(&self.auth_key)
                .send()
                .context(error::Http)?
                .json::<serde_json::Value>()
                .context(error::Http)?;

            log::info!("{:#?}", members);
        }

        Ok(())
    }

    /// Whether the `Auditor` has run at least one auditing procedure.
    pub fn has_run(&self) -> bool {
        self.has_run_audit
    }

    /// Gets a list of all members of an organisation from GitHub.
    fn member_list(&self, url: &str) -> Result<Vec<serde_json::Value>> {
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.auth_key)
            .send()
            .context(error::Http)?;

        let mut next = response
            .headers()
            .decode::<hyperx::header::Link>()
            .context(error::HyperX)?
            .values();

        Ok(vec![])
    }

    /// Convenience method to mark that at least one audit was performed on
    /// the repo.
    fn mark_audit(&mut self, msg: &str) {
        log::info!("⏳ Auditing {}", msg);
        self.has_run_audit = true;
    }
}
