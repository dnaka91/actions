use std::{fs::File, io::Cursor};

use anyhow::Result;
use camino::Utf8Path;
use flate2::{write::GzEncoder, Compression};
use tar::{Builder as TarBuilder, Header as TarHeader};
use target_lexicon::Triple;
use zip::{write::FileOptions as ZipFileOptions, CompressionMethod, ZipWriter};

pub fn tar_gz(file: &Utf8Path, name: &str, target: &Triple) -> Result<(String, Vec<u8>)> {
    let archive_name = format!("{name}-{target}.tar.gz");

    let builder = GzEncoder::new(Vec::new(), Compression::best());
    let mut builder = TarBuilder::new(builder);

    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(file.metadata()?.len());
    header.set_path(name)?;
    header.set_cksum();

    builder.append(&header, File::open(file)?)?;

    let data = builder.into_inner()?.finish()?;

    Ok((archive_name, data))
}

pub fn zip(file: &Utf8Path, name: &str, target: &Triple) -> Result<(String, Vec<u8>)> {
    let archive_name = format!("{name}-{target}.zip");

    let writer = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(writer);

    let options = ZipFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    writer.start_file(name, options)?;

    let mut file = File::open(file)?;
    std::io::copy(&mut file, &mut writer)?;

    let data = writer.finish()?.into_inner();

    Ok((archive_name, data))
}
