//! See [Opt].

use anyhow::{Context, bail};
use clap::Parser;
use tokio::fs;

use crate::io_util::wait_for_confirm;

use client::api::HttpVersionInfoProvider;

mod client;
mod data_dir;
mod format;
mod io_util;
mod platform;

/// Filename for latest.json metadata
const LATEST_FILENAME: &str = "latest.json";

/// A tool that generates Mullvad version metadata for Android.
#[derive(Parser)]
pub enum Opt {
    /// Download version metadata from releases.mullvad.net or API endpoint
    Pull {
        /// Replace files without asking for confirmation
        #[arg(long, short = 'y')]
        assume_yes: bool,
        /// Also update the latest.json file
        #[arg(long, default_value_t = false)]
        latest_file: bool,
    },

    /// List releases in `work/`
    ListReleases,

    /// Add release to `work/`
    AddRelease {
        /// Version to add
        version: mullvad_version::Version,
    },

    /// Remove release from `work/`
    RemoveRelease {
        /// Version to remove
        version: mullvad_version::Version,
    },

    /// Return the latest releases.
    /// The output is in JSON format.
    SetLatestStableVersion {
        /// Version to set at latest
        version: mullvad_version::Version,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::Pull {
            assume_yes,
            latest_file,
        } => {
            platform::pull(assume_yes).await?;

            // Download latest.json metadata if available
            if latest_file {
                match HttpVersionInfoProvider::get_latest_versions_file().await {
                    Ok(json_str) => {
                        let work_path = platform::work_path_latest();

                        if !assume_yes && work_path.exists() {
                            let msg = format!(
                                "This will replace the existing file at {}. Continue?",
                                work_path.display()
                            );
                            if !wait_for_confirm(&msg).await {
                                bail!("Aborted");
                            }
                        }

                        fs::write(&work_path, json_str)
                            .await
                            .context("Failed to write")?;

                        println!("Updated {}", work_path.display());
                    }
                    Err(err) => {
                        eprintln!("{err:?}");
                    }
                }
            }

            Ok(())
        }
        Opt::ListReleases => platform::list_releases().await,
        Opt::AddRelease { version } => platform::add_release(&version).await,
        Opt::RemoveRelease { version } => platform::remove_release(&version).await,
        Opt::SetLatestStableVersion { version } => platform::set_latest_stable(&version).await,
    }
}
