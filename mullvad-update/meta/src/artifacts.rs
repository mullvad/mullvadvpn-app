//! Generate metadata for installer artifacts
use anyhow::Context;
use std::path::Path;
use tokio::{
    fs,
    io::{AsyncSeekExt, BufReader},
};

use mullvad_update::{format, verify::Sha256Verifier};

/// Generate `format::Installer`
pub async fn generate_installer_details(
    architecture: format::Architecture,
    artifact: &Path,
) -> anyhow::Result<format::Installer> {
    let mut file = fs::File::open(artifact)
        .await
        .context(format!("Failed to open file at {}", artifact.display()))?;
    file.seek(std::io::SeekFrom::End(0))
        .await
        .context("Failed to seek to end")?;
    let file_size = file
        .stream_position()
        .await
        .context("Failed to get file size")?;
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .context("Failed to reset file pos")?;
    let file = BufReader::new(file);

    let checksum = Sha256Verifier::generate_hash(file)
        .await
        .context("Failed to compute checksum")?;

    Ok(format::Installer {
        architecture,
        // TODO: fetch cdns from config
        urls: vec![],
        size: file_size.try_into().context("Invalid file size")?,
        sha256: hex::encode(checksum),
    })
}
