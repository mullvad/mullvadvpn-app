//! See [Opt].

use clap::Parser;

mod client;
mod data_dir;
mod format;
mod io_util;
mod platform;

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
                platform::pull_latest(assume_yes).await?;
            }

            Ok(())
        }
        Opt::ListReleases => platform::list_releases().await,
        Opt::AddRelease { version } => platform::add_release(&version).await,
        Opt::RemoveRelease { version } => platform::remove_release(&version).await,
        Opt::SetLatestStableVersion { version } => platform::set_latest_stable(&version).await,
    }
}
