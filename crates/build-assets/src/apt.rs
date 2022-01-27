use std::{
    io::{BufRead, Cursor},
    process::Command, collections::HashSet,
};

use anyhow::{ensure, Result};

pub fn list_packages() -> Result<HashSet<String>> {
    let output = Command::new("sudo")
        .args(["apt-cache", "pkgnames"])
        .output()?;
    ensure!(
        output.status.success(),
        "failed listing APT packages: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    Cursor::new(output.stdout)
        .lines()
        .map(|line| line.map_err(Into::into))
        .collect()
}

pub fn update() -> Result<()> {
    let output = Command::new("sudo").args(["apt-get", "update"]).output()?;
    ensure!(
        output.status.success(),
        "failed updating APT cache: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

pub fn install_package(pkg: &str) -> Result<()> {
    let output = Command::new("sudo")
        .args(["apt-get", "install", "--yes", pkg])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()?;
    ensure!(
        output.status.success(),
        "failed installing APT package `{}`: {:?}",
        pkg,
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
