//! Parsing of common command line arguments.

use clap::Args;

/// Arguments that are always mandatory for all actions or provided through env vars in a runner
/// environment.
#[derive(Args)]
pub struct GithubArgs {
    /// The access token for GitHub, mandatory for any interactions with the API.
    #[clap(long, env = "GITHUB_TOKEN")]
    pub token: String,
    /// User and repo name like `dnaka91/actions`.
    #[clap(long, env = "GITHUB_REPOSITORY")]
    pub repository: String,
    /// Branch or tag name where an action was triggered.
    #[clap(long, env = "GITHUB_REF_NAME")]
    pub ref_name: String,
}
