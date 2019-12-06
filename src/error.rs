use std::fmt;

use snafu::Snafu;

/// A error during Auditing.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    /// No Authenication, or authenication token doesn't have the required
    /// permissions.
    #[snafu(display("No authentication key for GitHub provided."))]
    NoAuthKey,

    /// Error occurred while making HTTP request to GitHub.
    #[snafu(display("Error from HTTP client: {}", source))]
    Http {
        backtrace: snafu::Backtrace,
        source: reqwest::Error,
    },

    /// An error with decoding headers into ¬ types.
    HyperX {
        source: hyperx::Error,
        backtrace: snafu::Backtrace,
    },

    #[snafu(display("{}", kind))]
    Audit { kind: AuditError },

    #[snafu(display("Unexpected key missing from GitHub data."))]
    MissingGitHubData,
}

impl Error {
    pub fn is_audit(&self) -> bool {
        match self {
            Self::Audit { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum AuditError {
    Disabled2Fa,
    AdminsHaveCommits,
    NoAuditsRan,
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let warn = match self {
            Self::Disabled2Fa => {
                "2 Factor Authentication is not required for \
                 members of the organisation."
            }
            Self::AdminsHaveCommits => {
                "Admins have commit activity. This is usually an indication \
                 that admin members are using their accounts for purposes other \
                 than administration."
            }

            Self::NoAuditsRan => "No audits were performed.",
        };

        let recommendation = match self {
            Self::Disabled2Fa => "Enable 2 Factor as a requirement for members.",
            Self::AdminsHaveCommits => {
                "Create seperate accounts for administration access to \
                 the organisation."
            }
            Self::NoAuditsRan => "Adjust your configuration to enable some of audit procedures.",
        };

        writeln!(
            f,
            "❗️ Warning:\n{warn}\n\n💡 Recommendation:\n{recommendation}",
            warn = warn,
            recommendation = recommendation
        )
    }
}
