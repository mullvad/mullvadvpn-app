//! Generate metadata for installer artifacts

use anyhow::Context;
use std::path::Path;
use tokio::{fs, io::BufReader};

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
    let file = fs::File::open(artifact)
        .await
        .with_context(|| format!("Failed to open file at {}", artifact.display()))?;
    let metadata = file
        .metadata()
        .await
        .context("Failed to retrieve file metadata")?;
    let file_size = metadata.len();
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
    let urls = derive_urls(base_urls, filename);

    Ok(format::Installer {
        architecture,
        urls,
        size: file_size.try_into().context("Invalid file size")?,
        sha256: hex::encode(checksum),
    })
}

fn derive_urls(base_urls: &[String], filename: &str) -> Vec<String> {
    base_urls
        .iter()
        .map(|base_url| {
            let url = base_url.strip_suffix("/").unwrap_or(&base_url);
            format!("{url}/{}", filename)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test derivation of URLs from base URLs
    #[tokio::test]
    pub async fn test_urls() {
        let base_urls = vec![
            "https://fake1.fake/".to_string(),
            "https://fake2.fake".to_string(),
        ];

        assert_eq!(
            &derive_urls(&base_urls, "test.exe"),
            &["https://fake1.fake/test.exe", "https://fake2.fake/test.exe",]
        );
    }
}
