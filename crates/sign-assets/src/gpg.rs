//! Calling GPG to manage keys and sign files.

use std::{
    io::{self, BufRead, Cursor, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use actions_common::http::{Asset, AssetReader};
use anyhow::{bail, ensure, Result};
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use tracing::info;

/// Standard arguments for GPG that are passed to **every** invocation.
const DEFAULT_ARGS: &[&str] = &[
    "--batch",
    "--with-colons",
    "--yes",
    "--pinentry-mode",
    "loopback",
];

/// Identifier for a previously imported key. Can be created by importing a key with [`import_key`].
pub struct KeyId(String);

/// Import a new key into GPG. The key must be accessible on the file system and a passphrase must
/// be given if the key is protected, or the import will fail.
pub fn import_key(key: &str, passphrase: Option<&str>) -> Result<KeyId> {
    let gpg = find_gpg()?;
    let key = {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(key.as_bytes())?;
        file
    };

    let mut cmd = Command::new(&gpg);

    cmd.arg("--import")
        .args(DEFAULT_ARGS)
        .args(["--import-options", "import-show"]);

    if let Some(passphrase) = passphrase {
        cmd.args(["--passphrase", passphrase]);
    }

    let output = cmd.arg(key.path()).output()?;

    ensure!(
        output.status.success(),
        "failed importing key: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = Cursor::new(output.stdout).lines();
    for line in lines {
        if let Some(line) = line?.strip_prefix("fpr:") {
            let id = line.trim_matches(':').to_owned();
            info!(%id, "imported GPG key");

            return Ok(KeyId(id));
        }
    }

    bail!("failed finding key ID")
}

/// Delete both private and public part of the given key from GPG.
pub fn delete_key(key_id: &KeyId) -> Result<()> {
    let gpg = find_gpg()?;

    let output = Command::new(&gpg)
        .arg("--delete-secret-keys")
        .args(DEFAULT_ARGS)
        .arg(&key_id.0)
        .output()?;

    ensure!(
        output.status.success(),
        "failed deleting secret key: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    info!(id = %key_id.0, "deleted secret GPG key");

    let output = Command::new(&gpg)
        .arg("--delete-keys")
        .args(DEFAULT_ARGS)
        .arg(&key_id.0)
        .output()?;

    ensure!(
        output.status.success(),
        "failed deleting public key: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    info!(id = %key_id.0, "deleted public GPG key");

    Ok(())
}

/// Sign the list of given file contents with GPG.
///
/// The files are a tuple of file name and content. The file name is mostly used for error reporting
/// and logging, but to generate a name for the signature as well.
#[allow(clippy::missing_panics_doc)]
pub fn sign(
    key_id: &KeyId,
    passphrase: Option<&str>,
    files: Vec<(&Asset, AssetReader)>,
) -> Result<Vec<(String, Vec<u8>)>> {
    let gpg = find_gpg()?;

    files
        .into_par_iter()
        .map(|(asset, mut reader)| {
            let mut cmd = Command::new(&gpg);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            cmd.arg("--detach-sign")
                .args(DEFAULT_ARGS)
                .args(["--armor", "--output", "-"])
                .args(["--local-user", &key_id.0]);

            if let Some(passphrase) = passphrase {
                cmd.args(["--passphrase", passphrase]);
            }

            let mut child = cmd.arg("-").spawn()?;
            let stdin = child.stdin.as_mut().unwrap();
            io::copy(&mut reader, stdin)?;

            let output = child.wait_with_output()?;

            ensure!(
                output.status.success(),
                "failed creating signature for {:?}: {}",
                asset.name,
                String::from_utf8_lossy(&output.stderr)
            );

            info!(name = %asset.name, "signed file");

            Ok((format!("{}.asc", asset.name), output.stdout))
        })
        .collect()
}

/// Try finding the system-installed GPG executable.
fn find_gpg() -> Result<&'static Path> {
    static GPG: OnceCell<PathBuf> = OnceCell::new();

    GPG.get_or_try_init(|| which::which("gpg"))
        .map(PathBuf::as_path)
        .map_err(Into::into)
}
