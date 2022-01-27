#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use actions_common::{env, http};
use anyhow::Result;
use build_assets::{
    apt, archive, cargo::CargoBuilder, rustup, toolchain::Toolchain, triple::TripleExt,
};
use serde::Deserialize;
use serde_with::{rust as de, CommaSeparator};
use target_lexicon::Triple;
use tracing::info;

#[derive(Deserialize)]
struct Opt {
    #[serde(default, with = "de::display_fromstr")]
    toolchain: Toolchain,
    #[serde(default = "Triple::host", with = "de::display_fromstr")]
    target: Triple,
    #[serde(default, with = "de::StringWithSeparator::<CommaSeparator>")]
    features: Vec<String>,
    bin: String,
}

fn main() -> Result<()> {
    actions_common::tracing::init(env!("CARGO_CRATE_NAME"));

    let opt = env::input::<Opt>()?;
    let github = env::github()?;

    if let Some(pkg) = opt.target.apt_toolchain() {
        let installed_pkgs = apt::list_packages()?;
        info!("checked for installed APT packages");

        if !installed_pkgs.contains(pkg) {
            apt::update()?;
            info!("updated APT cache");

            apt::install_package(pkg)?;
            info!(%pkg, "installed APT package");
        }
    }

    rustup::install_toolchain(&opt.toolchain, &opt.target, &[])?;
    info!(toolchain = %opt.toolchain, target = %opt.target, "installed Rust toolchain");

    // let installed_targets = rustup::list_targets()?;
    // info!("checked for installed Rust targets");

    // if !installed_targets.contains(&opt.target) {
    //     rustup::install_target(&opt.target)?;
    //     info!(target = %opt.target, "installed Rust target");
    // }

    let binary = CargoBuilder::new(&opt.bin, &opt.target)
        .with_compiler(opt.target.cc_compiler())
        .with_features(&opt.features)
        .run()?;
    info!("compiled binary");

    let (name, data) = if opt.target.is_windows() {
        archive::zip(&binary, opt.bin.as_str(), &opt.target)
    } else {
        archive::tar_gz(&binary, opt.bin.as_str(), &opt.target)
    }?;

    info!("packaged binary as archive file");

    let release = http::get_release(&github.token, &github.repository, &github.ref_name)?;
    http::upload_asset(&github.token, &github.repository, release.id, &name, &data)?;

    info!("attached archive to release");

    Ok(())
}
