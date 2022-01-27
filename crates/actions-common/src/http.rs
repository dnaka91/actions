//! HTTP functions to interact with the GitHub API.

use std::io::{self, Read};

use anyhow::Result;
use attohttpc::{ResponseReader, body};
use serde::Deserialize;

/// Information about a specific release on GitHub.
#[derive(Deserialize)]
pub struct Release {
    /// Unique identifier.
    pub id: ReleaseId,
    /// Already attached assets.
    pub assets: Vec<Asset>,
}

/// Identifier for GitHub releases.
#[derive(Clone, Copy, Deserialize)]
#[serde(transparent)]
pub struct ReleaseId(u64);

/// Attached asset of a [`Release`].
#[derive(Deserialize)]
pub struct Asset {
    /// Unique identifier.
    pub id: AssetId,
    /// File name as shown in the GitHub UI.
    pub name: String,
    /// Download URL to directly download the asset.
    pub browser_download_url: String,
}

/// Identifier for GitHub assets which are part of a release.
#[derive(Clone, Copy, Deserialize)]
#[serde(transparent)]
pub struct AssetId(u64);

/// Content reader for a single [`Asset`].
pub struct AssetReader(ResponseReader);

impl Read for AssetReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

/// Get information about a release on GitHub, identified by its Git tag.
pub fn get_release(token: &str, repo: &str, tag: &str) -> Result<Release> {
    attohttpc::get(format!(
        "https://api.github.com/repos/{}/releases/tags/{}",
        repo, tag
    ))
    .bearer_auth(token)
    .header("Accept", "application/vnd.github.v3+json")
    .send()?
    .error_for_status()?
    .json()
    .map_err(Into::into)
}

/// Open a release asset for download.
pub fn download_asset(token: &str, asset: &Asset) -> Result<AssetReader> {
    let resp = attohttpc::get(&asset.browser_download_url)
        .bearer_auth(token)
        .follow_redirects(true)
        .send()?
        .error_for_status()?;

    let (_, _, reader) = resp.split();

    Ok(AssetReader(reader))
}

/// Upload an asset to an existing release.
pub fn upload_asset(
    token: &str,
    repo: &str,
    release: ReleaseId,
    name: &str,
    file: &[u8],
) -> Result<()> {
    attohttpc::post(format!(
        "https://uploads.github.com/repos/{}/releases/{}/assets?name={}",
        repo, release.0, name
    ))
    .bearer_auth(token)
    .header("Accept", "application/vnd.github.v3+json")
    .header("Content-Type", "text/plain")
    .body(body::Bytes(file))
    .send()?
    .error_for_status()?;

    Ok(())
}

/// Delete an already existing asset from a release.
pub fn delete_asset(token: &str, repo: &str, asset: AssetId) -> Result<()> {
    attohttpc::delete(format!(
        "https://api.github.com/repos/{}/releases/assets/{}",
        repo, asset.0
    ))
    .bearer_auth(token)
    .header("Accept", "application/vnd.github.v3+json")
    .send()?
    .error_for_status()?;

    Ok(())
}
