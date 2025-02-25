//! See [Opt].
//!
//! The tool can be installed using `cargo install --locked --path .`, after which it can be invoked
//! with `meta ...`.

use anyhow::{bail, Context};
use clap::Parser;
use config::Config;
use io_util::create_dir_and_write;

use mullvad_update::format::{self, key, SignedResponse};

use platform::Platform;

mod artifacts;
mod config;
mod github;
mod io_util;
mod platform;

/// Metadata expiry to use when not specified (months from now)
#[allow(dead_code)]
const DEFAULT_EXPIRY_MONTHS: usize = 6;

/// Rollout to use when not specified
const DEFAULT_ROLLOUT: f32 = 1.;

/// Lowest version to accept using 'verify'
const MIN_VERIFY_METADATA_VERSION: usize = 0;

/// Verification public key
const VERIFYING_PUBKEY: &str = include_str!("../../test-pubkey");

/// A tool that generates signed Mullvad version metadata.
///
/// Unsigned work is stored in `work/`, and signed work is stored in `signed/`
#[derive(Parser)]
pub enum Opt {
    /// Generate an ed25519 secret key
    GenerateKey,

    /// Create empty metadata files in work directory
    CreateMetadataFile {
        /// Platforms to write template for
        platforms: Vec<Platform>,
    },

    /// Download version metadata from releases.mullvad.net or API endpoint and store it in
    /// `signed/`
    Pull {
        /// Platforms to write template for
        platforms: Vec<Platform>,

        /// Replace signed files without user input
        #[arg(long, short = 'y')]
        assume_yes: bool,
    },

    /// List releases in `work/`
    ListReleases {
        /// Platforms to list releases for. All if none are specified
        platforms: Vec<Platform>,
    },

    /// Add release to `work/`
    AddRelease {
        /// Version to add
        version: mullvad_version::Version,
        /// Platforms to add releases for. All if none are specified
        platforms: Vec<Platform>,
        /// Rollout percentage (default is 1)
        #[arg(long)]
        rollout: Option<f32>,
    },

    /// Remove release from `work/`
    RemoveRelease {
        /// Version to remove
        version: mullvad_version::Version,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
    },

    /// Modify release in `work/`
    ModifyRelease {
        /// Version to modify
        version: mullvad_version::Version,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
        /// Rollout percentage. The default is 1
        #[arg(long)]
        rollout: Option<f32>,
    },

    /// Sign using an ed25519 key and output the signed metadata to `signed/`
    Sign {
        /// Secret ed25519 key used for signing, as hexadecimal string
        secret: key::SecretKey,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
        /// When the metadata expires, in months from now
        #[arg(long, default_value_t = DEFAULT_EXPIRY_MONTHS)]
        expiry: usize,
        /// Replace signed files without user input
        #[arg(long, short = 'y')]
        assume_yes: bool,
    },

    /// Verify that payloads are signed by a given ed25519 pubkey
    Verify {
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let config = Config::load_or_create().await?;

    match opt {
        Opt::GenerateKey => {
            println!("{}", key::SecretKey::generate().to_string());
            Ok(())
        }
        Opt::CreateMetadataFile { platforms } => {
            let json = serde_json::to_string_pretty(&SignedResponse {
                signatures: vec![],
                signed: format::Response::default(),
            })
            .expect("Failed to serialize empty response");
            for platform in all_platforms_if_empty(platforms) {
                let work_path = platform.work_path();
                println!("Adding empty template to {}", work_path.display());
                create_dir_and_write(work_path, &json).await?;
            }
            Ok(())
        }
        Opt::Pull {
            platforms,
            assume_yes,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                platform.pull(assume_yes).await?;
            }
            Ok(())
        }
        Opt::Sign {
            secret,
            platforms,
            expiry,
            assume_yes,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                platform
                    .sign(secret.clone(), expiry, assume_yes)
                    .await
                    .context("Failed to sign file")?;
            }
            Ok(())
        }
        Opt::ListReleases { platforms } => {
            for platform in all_platforms_if_empty(platforms) {
                platform.list_releases().await?;
                println!();
            }
            Ok(())
        }
        Opt::AddRelease {
            version,
            platforms,
            rollout,
        } => {
            let changes = github::fetch_changes_text(&version).await?;
            println!("\nchanges.txt for tag {version}:\n\n-- begin\n{changes}\n--end\n\n");

            for platform in all_platforms_if_empty(platforms) {
                platform
                    .add_release(&version, &changes, &config.base_urls, rollout)
                    .await?;
            }
            Ok(())
        }
        Opt::RemoveRelease { version, platforms } => {
            for platform in all_platforms_if_empty(platforms) {
                platform.remove_release(&version).await?;
            }
            Ok(())
        }
        Opt::ModifyRelease {
            version,
            platforms,
            rollout,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                platform.modify_release(&version, rollout).await?;
            }
            Ok(())
        }
        Opt::Verify { platforms } => {
            let mut any_failed = false;
            for platform in all_platforms_if_empty(platforms) {
                if let Err(err) = platform.verify().await {
                    any_failed = true;
                    eprintln!("Error for {platform}: {err:?}");
                }
            }
            if any_failed {
                bail!("Some signatures failed to be verified");
            }
            Ok(())
        }
    }
}

fn all_platforms_if_empty(platforms: Vec<Platform>) -> Vec<Platform> {
    if platforms.is_empty() {
        return Platform::all().to_vec();
    }
    platforms
}
