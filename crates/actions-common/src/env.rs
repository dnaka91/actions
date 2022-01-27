//! Parsing of common command line arguments.

use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};

/// Arguments that are always mandatory for all actions or provided through env vars in a runner
/// environment.
#[derive(Deserialize)]
pub struct GithubArgs {
    /// The access token for GitHub, mandatory for any interactions with the API.
    pub token: String,
    /// User and repo name like `dnaka91/actions`.
    pub repository: String,
    /// Branch or tag name where an action was triggered.
    pub ref_name: String,
}

/// Load all commonly present environment variables.
pub fn github() -> Result<GithubArgs> {
    parse("GITHUB_")
}

/// Load input variables.
pub fn input<T: DeserializeOwned>() -> Result<T> {
    parse("INPUT_")
}

fn parse<T: DeserializeOwned>(prefix: &str) -> Result<T> {
    envy::prefixed(prefix)
        .from_iter(std::env::vars_os().filter_map(|(k, v)| {
            k.into_string()
                .ok()
                .zip(v.into_string().ok())
                .filter(|(_, v)| !v.is_empty())
        }))
        .map_err(Into::into)
}
