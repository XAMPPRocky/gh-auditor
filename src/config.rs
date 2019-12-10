/// A struct configuring which audits the `Auditor` should run.
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Config {
    // Toggles
    /// Warns if the organisation requires 2 factor authenication for all
    /// of it's members. (Default: `true`)
    pub enforces_2fa: bool,
    /// Warns if an organisation has admin accounts that have commit activity.
    /// (Default: `true`)
    pub admins_have_no_commit_activity: bool,
    /// Warns if an organisation has repositories that have unprotected master
    /// branches. (Default: `false`)
    pub all_repos_master_is_protected: bool,

    // Whitelists
    /// Matches a list of installed applications in a organisation against a
    /// whitelist, if provided. Uses the URL slug of the app e.g. `foobar`.
    /// Warns if there are installations other than ones specified **or** There
    /// is a missing installation from an organisation. (Default: `None`)
    pub installed_app_whitelist: Option<Vec<String>>,
    /// Matches a list of admins in a organisation against a whitelist, if
    /// provided. Uses the username of person's GitHub account (e.g. `bors`).
    /// Warns if there are admins other than ones specified **or** There
    /// are missing admins from an organisation. (Default: `None`)
    pub admin_whitelist: Option<Vec<String>>,
    /// Matches a list of users in a organisation against a whitelist, if
    /// provided. Uses the username of person's GitHub account (e.g. `bors`).
    /// Warns if there are users other than ones specified **or** There
    /// are missing users from an organisation. (Default: `None`)
    pub member_whitelist: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enforces_2fa: true,
            admins_have_no_commit_activity: true,
            all_repos_master_is_protected: true,
            installed_app_whitelist: None,
            admin_whitelist: None,
            member_whitelist: None,
        }
    }
}
