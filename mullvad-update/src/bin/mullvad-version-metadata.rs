//! See [Opt].

use anyhow::{anyhow, Context};
use clap::Parser;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tokio::{fs, io};

use mullvad_update::format::{self, key};

#[allow(dead_code)]
const DEFAULT_EXPIRY_MONTHS: u32 = 6;

// TODO: fail whenever 'files' doesn't contain at least one file

/// A tool that generates signed Mullvad version metadata.
#[derive(Parser)]
pub enum Opt {
    /// Generate an ed25519 secret key
    GenerateKey,

    /// Download version metadata from releases.mullvad.net or API endpoint
    /// meta download
    DownloadMetadataFiles,

    /// Create empty metadata files
    /// meta create-metadata-file metadata/{windows,macos,linux}-metadata.json
    CreateMetadataFile {
        /// Files to write template to
        files: Vec<PathBuf>,
    },

    /// List releases in the given metadata files
    /// meta list metadata/windows-metadata.json
    /// meta list metadata/*
    ListReleases {
        /// Files to list releases for
        files: Vec<PathBuf>,
    },

    /// Add release to the given files
    /// meta add-release 2025.4 [--rollout <rate>] metadata/windows-metadata.json
    /// meta add-release 2025.4 [--rollout <rate>] metadata/*
    AddRelease {
        /// Version to add
        version: mullvad_version::Version,
        /// Rollout percentage (default is 1)
        /// TODO: must be 0..=1

        #[arg(long, short = 'w')]
        rollout: Option<f32>,
        /// Files to change `version` for
        files: Vec<PathBuf>,
        // TODO: installers
    },

    /// Remove release from the given files
    /// meta remove-release 2025.3 metadata/windows-metadata.json
    /// meta remove-release 2025.3 metadata/*
    RemoveRelease {
        /// Version to remove
        version: mullvad_version::Version,
        /// Files to remove `version` from
        files: Vec<PathBuf>,
    },

    /// Modify release in the given files
    /// meta modify-release 2025.4 [--rollout <rate>] metadata/windows-metadata.json
    /// meta modify-release 2025.4 [--rollout <rate>] metadata/*
    ModifyRelease {
        /// Version to modify
        version: mullvad_version::Version,
        /// Rollout percentage. The default is 1
        /// TODO: must be 0..=1
        #[arg(long, short = 'w')]
        rollout: Option<f32>,
        /// Files to remove `version` from
        files: Vec<PathBuf>,
    },

    /// Sign a JSON payload using an ed25519 key and output the signed metadata
    /// meta sign metadata/*
    Sign {
        /// Secret ed25519 key used for signing, as hexadecimal string
        secret: key::SecretKey,
        /// Files to sign
        files: Vec<PathBuf>,
    },

    /// Verify that payloads are signed by a given ed25519 pubkey
    /// meta verify metadata/*
    Verify {
        /// Files to verify
        files: Vec<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::GenerateKey => {
            println!("{}", key::SecretKey::generate());
            Ok(())
        }
        Opt::CreateMetadataFile { files } => {
            let mut response = format::Response::default();
            let json = serde_json::to_string_pretty(&response)
                .expect("Failed to serialize empty response");
            for file in files {
                fs::write(file, &json).await?;
            }
            Ok(())
        }
        Opt::Sign { secret, files } => {
            for file in files {
                println!("Signing file {}...", file.display());
                sign(&file, secret.clone())
                    .await
                    .context("Failed to sign file")?;
            }
            Ok(())
        }
        Opt::DownloadMetadataFiles => {
            /*const VERSION_METADATA_URLS: &[&str] = [

            ];*/
            // TODO
            //reqwest::get("https://releases.mullvad.net/")

            // TODO: verify
            Ok(())
        }
        Opt::ListReleases { files } => {
            for file in files {
                println!("Releases for file {}", file.display());

                let bytes = fs::read(file).await.context("Failed to read file")?;
                let mut response: format::Response =
                    serde_json::from_slice(&bytes).context("Failed to deserialize file")?;

                response.releases.sort_by(|a, b| {
                    mullvad_version::Version::version_ordering(&b.version, &a.version)
                });

                // Version: 2025.3 (arm, x86) (50%)
                // <Changelog>
                for release in response.releases {
                    print_release_info(&release);
                }
            }
            Ok(())
        }
        Opt::AddRelease {
            version,
            rollout,
            files,
        } => {
            // add-release <version-number> [--rollout <rate>] <changelog-path> <artifact-dir> <url-base> <metadata-files>
            /*for file in files {
                let bytes = fs::read(file).await.context("Failed to read file")?;
                let response: format::Response = serde_json::from_slice(&bytes).context("Failed to deserialize file")?;
            }*/
            Ok(())
        }
        Opt::RemoveRelease { version, files } => {
            for file in files {
                let bytes = fs::read(&file).await.context("Failed to read file")?;
                let mut response: format::Response =
                    serde_json::from_slice(&bytes).context("Failed to deserialize file")?;

                let Some(found_release_ind) = response
                    .releases
                    .iter()
                    .position(|release| release.version == version)
                else {
                    continue;
                };

                let removed_release = response.releases.swap_remove(found_release_ind);

                println!("Removed release in {}", file.display());
                print_release_info(&removed_release);

                let json = serde_json::to_string_pretty(&response)
                    .context("Failed to serialize updated metadata")?;
                fs::write(file, &json).await?;
            }
            Ok(())
        }
        Opt::ModifyRelease {
            version,
            rollout,
            files,
        } => {
            for file in files {
                let bytes = fs::read(&file).await.context("Failed to read file")?;
                let mut response: format::Response =
                    serde_json::from_slice(&bytes).context("Failed to deserialize file")?;

                let Some(release) = response
                    .releases
                    .iter_mut()
                    .find(|release| release.version == version)
                else {
                    continue;
                };

                if let Some(new_rollout) = rollout {
                    release.rollout = new_rollout;
                }

                println!("Updated release in {}", file.display());
                print_release_info(&release);

                let json = serde_json::to_string_pretty(&response)
                    .context("Failed to serialize updated metadata")?;
                fs::write(file, &json).await?;
            }
            Ok(())
        }
        Opt::Verify { files } => todo!(),
    }
}

fn print_release_info(release: &format::Release) {
    let mut architectures: Vec<_> = release
        .installers
        .iter()
        .map(|installer| installer.architecture.to_string())
        .collect();
    architectures.dedup();
    let architectures = architectures.join(", ");

    println!(
        "- {} ({}) ({}%)",
        release.version,
        architectures,
        (release.rollout * 100.) as u32
    );
}

async fn sign(file: &Path, secret: key::SecretKey) -> anyhow::Result<()> {
    // Read unsigned JSON data
    let data: Vec<u8> = fs::read(file).await?;

    // Deserialize version data
    let response: format::Response =
        serde_json::from_slice(&data).context("Failed to deserialize version metadata")?;

    // Sign it
    let signed_response = format::SignedResponse::sign(secret, response)?;

    // Print it
    println!(
        "{}",
        serde_json::to_string_pretty(&signed_response)
            .context("Failed to serialize signed version")?
    );

    Ok(())
}
