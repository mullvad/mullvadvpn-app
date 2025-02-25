//! Generate metadata for installer artifacts

use anyhow::Context;
use std::path::Path;
use tokio::{
    fs,
    io::{AsyncSeekExt, BufReader},
};

use mullvad_update::{format, hash};

/// Generate `format::Installer` for a given `artifact`.
///
/// The presence of the files relative to `base_urls` is not verified.
/// See [crate::config::Config::base_urls] for the assumptions made.
pub async fn generate_installer_details(
    architecture: format::Architecture,
    base_urls: &[String],
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

    println!("Generating checksum for {}", artifact.display());

    let checksum = hash::checksum(file)
        .await
        .context("Failed to compute checksum")?;

    // Construct URLs from base URLs
    let filename = artifact
        .file_name()
        .and_then(|f| f.to_str())
        .context("Unexpected filename")?;
    let urls = base_urls
        .iter()
        .map(|base_url| {
            let base_url = base_url.split('/').next().unwrap_or(base_url);
            format!("{base_url}/{}", filename)
        })
        .collect();

    Ok(format::Installer {
        architecture,
        urls,
        size: file_size.try_into().context("Invalid file size")?,
        sha256: hex::encode(checksum),
    })
}
