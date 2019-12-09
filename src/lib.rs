//! GH Auditor: Audit and enforce access permission policy for an organisation.
#![warn(missing_docs)]

mod builder;
mod config;
mod error;

pub use builder::AuditorBuilder;

use std::borrow::Cow;

use hyperx::header::{RelationType, TypedHeaders};
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
                    log::error!("{}", error);
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
            self.mark_audit("2 Factor Authentication");
            let enabled = self
                .organisation
                .get("two_factor_requirement_enabled")
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false);

            if enabled {
                log::info!("✅ 2 Factor Authentication required for members");
            } else {
                return Err(error::Error::Audit {
                    kind: error::AuditError::Disabled2Fa,
                });
            }
        }

        Ok(())
    }

    fn audit_admin_commit_activity(&mut self) -> Result<()> {
        if !self.config.admins_have_no_commit_activity {
            return Ok(());
        }
        self.mark_audit("Admin Commit Activity");

        let mut members_url = self
            .organisation
            .get("members_url")
            .and_then(serde_json::Value::as_str)
            .context(error::MissingGitHubData)?
            .replace("{/member}", "");

        members_url += "?role=admin";

        let members = self.get_all(members_url)?;
        let mut found_members = Vec::new();

        for member in members {
            let events_url = member
                .get("events_url")
                .and_then(serde_json::Value::as_str)
                .context(error::MissingGitHubData)?
                .replace("{/privacy}", "");

            let has_pushed = self.find(events_url, |e| {
                e.get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "PushEvent")
                    .unwrap_or(false)
            })?;

            if has_pushed.is_some() {
                found_members.push(member);
            }
        }

        if found_members.is_empty() {
            log::info!("✅ No recent push activity on admin accounts.");
            Ok(())
        } else {
            Err(error::Error::Audit {
                kind: error::AuditError::AdminsHaveCommits(found_members),
            })
        }
    }

    /// Whether the `Auditor` has run at least one auditing procedure.
    pub fn has_run(&self) -> bool {
        self.has_run_audit
    }

    fn find(
        &self,
        url: String,
        pred: impl FnMut(&&serde_json::Value) -> bool + Copy,
    ) -> Result<Option<serde_json::Value>> {
        let mut next = Some(url);

        while let Some(url) = next {
            let mut response = self
                .client
                .get(&url)
                .bearer_auth(&self.auth_key)
                .send()
                .context(error::Http)?;

            next = response
                .headers()
                .decode::<hyperx::header::Link>()
                .ok()
                .and_then(|v| {
                    v.values()
                        .iter()
                        .find(|link| {
                            link.rel()
                                .map(|rel| rel.contains(&RelationType::Next))
                                .unwrap_or(false)
                        })
                        .map(|l| l.link())
                        .map(str::to_owned)
                });

            let json = response.json::<serde_json::Value>().context(error::Http)?;

            let item = json.as_array().and_then(|v| v.iter().find(pred));

            if let Some(item) = item {
                return Ok(Some(item.clone()));
            }
        }

        Ok(None)
    }

    /// Gets a all entries from a URL using `Link: rel="next";` from GitHub.
    fn get_all(&self, url: String) -> Result<Vec<serde_json::Value>> {
        let mut entities = Vec::new();
        let mut next = Some(url);

        while let Some(url) = next {
            let mut response = self
                .client
                .get(&url)
                .bearer_auth(&self.auth_key)
                .send()
                .context(error::Http)?;

            next = response
                .headers()
                .decode::<hyperx::header::Link>()
                .ok()
                .and_then(|v| {
                    v.values()
                        .iter()
                        .find(|link| {
                            link.rel()
                                .map(|rel| rel.contains(&RelationType::Next))
                                .unwrap_or(false)
                        })
                        .map(|l| l.link())
                        .map(str::to_owned)
                });

            let json = response.json::<serde_json::Value>().context(error::Http)?;

            entities.extend_from_slice(&json.as_array().context(error::MissingGitHubData)?);
        }

        Ok(entities)
    }

    /// Convenience method to mark that at least one audit was performed on
    /// the repo.
    fn mark_audit(&mut self, msg: &str) {
        log::info!("⏳ Auditing {}", msg);
        self.has_run_audit = true;
    }
}
