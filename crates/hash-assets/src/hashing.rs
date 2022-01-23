//! Hashing of file contents with different algorithms.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
};

use actions_common::http::{Asset, AssetReader};
use anyhow::Result;
use blake2::Blake2b512;
use rayon::prelude::*;
use sha2::{Digest, Sha256, Sha512};
use tracing::info;

/// Hash the given list of assets with multiple hashing algorithms.
///
/// A list of hash files is returned, one for each algorithm containing the hashes of all given
/// files.
pub fn hash(files: Vec<(&Asset, AssetReader)>) -> Result<Vec<(String, Vec<u8>)>> {
    build_files(&build_hashes(files)?)
}

struct Hashes {
    values: HashMap<Hash, Vec<u8>>,
}

const DEFAULT_HASHES: &[Hash] = &[Hash::Blake2, Hash::Sha256, Hash::Sha512];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Hash {
    Blake2,
    Sha256,
    Sha512,
}

impl Hash {
    fn extension(self) -> &'static str {
        match self {
            Self::Blake2 => "b2",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    fn hasher(self) -> Hasher {
        match self {
            Self::Blake2 => Hasher::Blake2(Blake2b512::new()),
            Self::Sha256 => Hasher::Sha256(Sha256::new()),
            Self::Sha512 => Hasher::Sha512(Sha512::new()),
        }
    }
}

enum Hasher {
    Blake2(Blake2b512),
    Sha256(Sha256),
    Sha512(Sha512),
}

impl Hasher {
    fn update(&mut self, value: &[u8]) {
        match self {
            Self::Blake2(h) => h.update(value),
            Self::Sha256(h) => h.update(value),
            Self::Sha512(h) => h.update(value),
        }
    }

    fn finalize(self) -> Vec<u8> {
        match self {
            Self::Blake2(h) => h.finalize().to_vec(),
            Self::Sha256(h) => h.finalize().to_vec(),
            Self::Sha512(h) => h.finalize().to_vec(),
        }
    }
}

impl Hashes {
    fn digest(mut input: impl BufRead, hashes: &[Hash]) -> Result<Self> {
        let mut hashes = hashes
            .iter()
            .map(|hash| (*hash, hash.hasher()))
            .collect::<HashMap<_, _>>();

        loop {
            let buffer = input.fill_buf()?;
            if buffer.is_empty() {
                break Ok(Self {
                    values: hashes
                        .into_iter()
                        .map(|(hash, hasher)| (hash, hasher.finalize()))
                        .collect(),
                });
            }

            hashes.values_mut().for_each(|hasher| hasher.update(buffer));

            let length = buffer.len();
            input.consume(length);
        }
    }
}

fn build_hashes(files: Vec<(&Asset, AssetReader)>) -> Result<Vec<(&Asset, Hashes)>> {
    files
        .into_par_iter()
        .map(|(asset, reader)| {
            let input = BufReader::new(reader);
            Hashes::digest(input, DEFAULT_HASHES).map(|hashes| (asset, hashes))
        })
        .inspect(|item| {
            if let Ok((asset, _)) = item {
                info!(name = %asset.name, "hashed asset");
            }
        })
        .collect()
}

fn build_files(file_hashes: &[(&Asset, Hashes)]) -> Result<Vec<(String, Vec<u8>)>> {
    DEFAULT_HASHES
        .iter()
        .map(|&hash| {
            write_hashes(
                file_hashes,
                format!("checksums.{}", hash.extension()),
                |h| h.values.get(&hash),
            )
        })
        .collect()
}

fn write_hashes(
    files: &[(&Asset, Hashes)],
    name: String,
    f: impl Fn(&Hashes) -> Option<&Vec<u8>>,
) -> Result<(String, Vec<u8>)> {
    let mut checksums = Vec::new();

    for (asset, hashes) in files {
        if let Some(hash) = f(hashes) {
            writeln!(&mut checksums, "{} *{}", hex::encode(hash), asset.name,)?;
        }
    }

    info!(%name, "built hashsum file");

    Ok((name, checksums))
}
