//! See [Opt].

use anyhow::{Context, bail};
use clap::Parser;
use std::path::PathBuf;
use tokio::fs;

use mullvad_update::api::HttpVersionInfoProvider;

use crate::io_util::wait_for_confirm;

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
    QueryLatest,
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
                match HttpVersionInfoProvider::get_latest_android_versions_file().await {
                    Ok(json_str) => {
                        let path_buf = get_data_dir().join(LATEST_FILENAME);
                        let path = path_buf.as_path();

                        if !assume_yes && path.exists() {
                            let msg = format!(
                                "This will replace the existing file at {}. Continue?",
                                path.display()
                            );
                            if !wait_for_confirm(&msg).await {
                                bail!("Aborted");
                            }
                        }

                        fs::write(path, json_str).await.context("Failed to write")?;

                        println!("Updated {}", path.display());
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
        Opt::QueryLatest => {
            #[derive(Default, serde::Serialize)]
            struct SummaryQueryResult {
                android: Option<QueryResultOs>,
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

            let out = platform::query_latest().await?;
            summary_result.android = Some(QueryResultOs {
                stable: out.stable.into(),
                beta: out.beta.map(Into::into),
            });

            let json = serde_json::to_string_pretty(&summary_result)
                .context("Failed to serialize versions")?;
            println!("{json}");

            Ok(())
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    std::env::home_dir()
        .expect("No home dir found")
        .join(".local/share/mullvad-release-android")
}
