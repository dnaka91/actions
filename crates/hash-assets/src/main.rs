#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use actions_common::{
    env, glob,
    http::{self, Release},
};
use anyhow::{Context, Result};
use hash_assets::hashing;
use rayon::prelude::*;
use serde::Deserialize;
use serde_with::{rust as de, CommaSeparator};
use tracing::info;

#[derive(Deserialize)]
struct Opt {
    #[serde(
        default = "default_globs",
        with = "de::StringWithSeparator::<CommaSeparator>"
    )]
    globs: Vec<String>,
}

fn default_globs() -> Vec<String> {
    vec!["*.tar.gz".to_owned(), "*.zip".to_owned()]
}

fn main() -> Result<()> {
    actions_common::tracing::init(env!("CARGO_CRATE_NAME"));

    let opt = env::input::<Opt>()?;
    let github = env::github()?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()?;

    let release = http::get_release(&github.token, &github.repository, &github.ref_name)
        .context("failed getting release info")?;

    let globset = glob::build_globset(&opt.globs)?;
    let assets = release
        .assets
        .par_iter()
        .filter_map(|asset| {
            globset
                .is_match(&asset.name)
                .then(|| http::download_asset(&github.token, asset).map(|r| (asset, r)))
        })
        .collect::<Result<Vec<_>>>()
        .context("failed downloading assets")?;

    let hashes = hashing::hash(assets).context("failed hashing assets")?;

    upload_files(&github.token, &github.repository, &release, &hashes)?;

    Ok(())
}

fn upload_files(
    token: &str,
    repo: &str,
    release: &Release,
    files: &[(String, Vec<u8>)],
) -> Result<()> {
    files.into_par_iter().try_for_each(|(name, file)| {
        if let Some(asset) = release.assets.iter().find(|asset| &asset.name == name) {
            http::delete_asset(token, repo, asset.id)?;
            info!(name = %asset.name, "deleted existing asset");
        }

        http::upload_asset(token, repo, release.id, name, file)?;
        info!(%name, "uploaded new asset");

        anyhow::Ok(())
    })
}
