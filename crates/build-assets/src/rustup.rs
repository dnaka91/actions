use std::{
    collections::HashSet,
    io::{BufRead, Cursor},
    process::Command,
};

use anyhow::{ensure, Result};
use target_lexicon::Triple;

use crate::toolchain::Toolchain;

pub fn list_toolchains() -> Result<HashSet<String>> {
    let output = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()?;
    ensure!(
        output.status.success(),
        "failed listing installed toolchains: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    #[allow(clippy::map_unwrap_or)]
    Cursor::new(&output.stdout)
        .lines()
        .map(|line| {
            let line = line?;
            Ok(line
                .split_whitespace()
                .next()
                .map(str::to_owned)
                .unwrap_or(line))
        })
        .collect()
}

pub fn install_toolchain(
    toolchain: &Toolchain,
    target: &Triple,
    components: &[&str],
) -> Result<()> {
    let mut cmd = Command::new("rustup");
    cmd.args(["toolchain", "install"]);
    cmd.arg(toolchain.to_string());
    cmd.args(["--profile", "minimal"]);
    cmd.arg("--target").arg(target.to_string());

    for component in components {
        cmd.args(["--component", component]);
    }

    let output = cmd.output()?;
    ensure!(
        output.status.success(),
        "failed installing toolchain: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

pub fn list_targets() -> Result<HashSet<Triple>> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;
    ensure!(
        output.status.success(),
        "failed listing installed targets: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    Cursor::new(&output.stdout)
        .lines()
        .map(|line| Ok(line?.parse()?))
        .collect()
}

pub fn install_target(target: &Triple) -> Result<()> {
    let output = Command::new("rustup")
        .args(["target", "add"])
        .arg(target.to_string())
        .output()?;
    ensure!(
        output.status.success(),
        "failed adding target: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
