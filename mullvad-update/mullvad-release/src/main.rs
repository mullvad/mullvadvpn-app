//! See [Opt].
//!
//! The tool can be installed using `cargo install --locked --path .`, after which it can be invoked
//! with `mullvad-release ...`.

use anyhow::{Context, bail};
use clap::Parser;
use std::{path::Path, str::FromStr};
use tokio::fs;

use config::Config;
use io_util::create_dir_and_write;
use platform::Platform;

use mullvad_update::{
    api::HttpVersionInfoProvider,
    format::{self, SignedResponse, key},
    version::{FULLY_ROLLED_OUT, Rollout},
};

use crate::io_util::wait_for_confirm;

mod artifacts;
mod config;
mod github;
mod io_util;
mod platform;

/// Metadata expiry to use when not specified (months from now)
const DEFAULT_EXPIRY_MONTHS: usize = 6;

/// Rollout to use when not specified
const DEFAULT_ROLLOUT: Rollout = FULLY_ROLLED_OUT;

/// Filename for latest.json metadata
const LATEST_FILENAME: &str = "latest.json";

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

        /// Replace signed files without asking for confirmation
        #[arg(long, short = 'y')]
        assume_yes: bool,

        /// Also update the latest.json file
        #[arg(long, default_value_t = false)]
        latest_file: bool,
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
        /// Rollout fraction to set (0 = not rolled out, 1 = fully rolled out).
        #[arg(long, default_value_t = DEFAULT_ROLLOUT)]
        rollout: Rollout,
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
        /// If set, modify the rollout fraction.
        #[arg(long)]
        rollout: Option<Rollout>,
    },

    /// Sign using an ed25519 key and output the signed metadata to `signed/`
    /// A secret ed25519 key will be read from stdin
    Sign {
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
        /// When the metadata expires, in months from now
        #[arg(long, default_value_t = DEFAULT_EXPIRY_MONTHS)]
        expiry: usize,
        /// Replace signed files without asking for confirmation
        #[arg(long, short = 'y')]
        assume_yes: bool,
    },

    /// Verify that payloads are signed by a given ed25519 pubkey
    Verify {
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
    },

    /// Return the latest releases in `signed/` based on the given parameters.
    /// The output is in JSON format.
    QueryLatest {
        /// Platforms to query for. All if none are specified
        platforms: Vec<Platform>,
        /// Rollout threshold to use (0 = not rolled out, 1 = fully rolled out).
        ///
        /// By default, any non-zero rollout is accepted.
        /// Setting the value to zero will also show supported versions that have
        /// been released but are currently not being rolled out.
        // TODO: this prints 0%, but should print 1.1920929e-7
        #[arg(long, default_value_t = mullvad_update::version::SUPPORTED_VERSION)]
        rollout: Rollout,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let config = Config::load_or_create().await?;

    match opt {
        Opt::GenerateKey => {
            let secret = key::SecretKey::generate();
            println!("Secret key: {secret}");
            println!("Public key: {}", secret.pubkey());
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
            latest_file,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                platform.pull(assume_yes).await?;
            }

            // Download latest.json metadata if available
            if latest_file {
                match HttpVersionInfoProvider::get_latest_versions_file()
                    .await
                    .and_then(|json| {
                        serde_json::to_string_pretty(&json).context("Failed to format JSON")
                    }) {
                    Ok(json) => {
                        let path = Path::new(LATEST_FILENAME);

                        if !assume_yes && path.exists() {
                            let msg = format!(
                                "This will replace the existing file at {}. Continue?",
                                path.display()
                            );
                            if !wait_for_confirm(&msg).await {
                                bail!("Aborted");
                            }
                        }

                        fs::write(path, json).await.context("Failed to write")?;

                        println!("Updated {}", path.display());
                    }
                    Err(err) => {
                        eprintln!("{err:?}");
                    }
                }
            }

            Ok(())
        }
        Opt::Sign {
            platforms,
            expiry,
            assume_yes,
        } => {
            let key_str = io_util::wait_for_input("Enter ed25519 secret: ")
                .await
                .context("Failed to read secret from stdin")?;
            let secret = key::SecretKey::from_str(&key_str).context("Invalid secret")?;

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
            let mut any_failed = false;
            for platform in all_platforms_if_empty(platforms) {
                if let Err(err) = platform.modify_release(&version, rollout).await {
                    any_failed = true;
                    eprintln!("Error for {platform}: {err}");
                }
            }
            if any_failed {
                bail!("Some platforms failed to be modified");
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
        Opt::QueryLatest { platforms, rollout } => {
            #[derive(Default, serde::Serialize)]
            struct SummaryQueryResult {
                linux: Option<QueryResultOs>,
                windows: Option<QueryResultOs>,
                macos: Option<QueryResultOs>,
            }
            #[derive(serde::Serialize)]
            struct QueryResultOs {
                stable: QueryResultVersion,
                beta: Option<QueryResultVersion>,
            }
            #[derive(serde::Serialize)]
            struct QueryResultVersion {
                version: mullvad_version::Version,
            }
            impl From<mullvad_version::Version> for QueryResultVersion {
                fn from(version: mullvad_version::Version) -> Self {
                    QueryResultVersion { version }
                }
            }

            let mut summary_result = SummaryQueryResult::default();

            for platform in all_platforms_if_empty(platforms) {
                let out = platform.query_latest(rollout).await?;

                match platform {
                    Platform::Linux => {
                        summary_result.linux = Some(QueryResultOs {
                            stable: out.stable.into(),
                            beta: out.beta.map(Into::into),
                        });
                    }
                    Platform::Windows => {
                        summary_result.windows = Some(QueryResultOs {
                            stable: out.stable.into(),
                            beta: out.beta.map(Into::into),
                        });
                    }
                    Platform::Macos => {
                        summary_result.macos = Some(QueryResultOs {
                            stable: out.stable.into(),
                            beta: out.beta.map(Into::into),
                        });
                    }
                }
            }

            let json = serde_json::to_string_pretty(&summary_result)
                .context("Failed to serialize versions")?;
            println!("{json}");

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
