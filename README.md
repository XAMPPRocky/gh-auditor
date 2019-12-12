# GitHub Auditor
`gh-auditor` is program that allows you to easily check if your organisation
meets your criteria.

## Checks

* Organisation has 2 Factor Authenication enabled
* Seperate accounts for adminstration
* All master branches are protected

#### In Progress
- [ ] Member whitelist
- [ ] Admin whitelist
- [ ] Additional branch protection checks (e.g. requiring verified commits)

## Usage
To run an audit on an organisation you need provide the organisation name and
a GitHub access token with `admin:read` rights.

```
gh-auditor rust-lang
```

By default `gh-auditor` will try to read from the `GITHUB_API_KEY` environment
variable. You can supply it from the command line.

```
gh-auditor -t "<token>" rust-lang
```

#### CLI
```
Erin P. <xampprocky@gmail.com>

USAGE:
    gh-auditor [OPTIONS] <organisation>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --token <token>    GitHub authentication token.

ARGS:
    <organisation>    GitHub Organisation to audit. Requires `admin:read` level permissions
```
