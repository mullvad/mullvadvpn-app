// TODO:
/*
If a new upgrade version becomes available during an app upgrade it should not affect the suggested upgrade version in the SuggestedUpgrade message if the upgrade is still in progress. That is, if the current state is one of
AppUpgradeDownloadStarting
AppUpgradeDownloadProgress
AppUpgradeVerifyingInstaller
*/

use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use mullvad_update::app::{AppDownloaderParameters, HttpAppDownloader};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

use super::Error;
type Result<T> = std::result::Result<T, Error>;

pub struct AppUpdater {
    task: tokio::task::JoinHandle<()>,
}

pub type AbortHandle = oneshot::Sender<()>;

/// App updater event
pub enum UpdateEvent {
    /// Download progress update
    Downloading {
        /// A fraction in `[0,1]` that describes how much of the installer has been downloaded
        complete_frac: f32,
        /// Estimated time left
        time_left: Duration,
    },
    /// Download failed due to some error
    DownloadFailed,
    /// Download completed, so verifying now
    Verifying,
    /// The verification failed due to some error
    VerificationFailed,
    /// There is a downloaded and verified installer available
    Verified { verified_installer_path: PathBuf },
}

impl AppUpdater {
    /// Begin or resume download of `version`
    pub async fn spawn(
        version: mullvad_update::version::Version,
        event_tx: mpsc::Sender<UpdateEvent>,
    ) -> Result<AppUpdater> {
        // TODO: select url
        let url = version.urls[0].clone();

        let download_dir = mullvad_paths::cache_dir()?.join("mullvad-update");
        fs::create_dir_all(&download_dir)
            .await
            .map_err(Error::CreateDownloadDir)?;

        let params = AppDownloaderParameters {
            app_version: version.version,
            app_url: url.clone(),
            app_size: version.size,
            app_progress: ProgressUpdater::new(url, event_tx.clone()),
            app_sha256: version.sha256,
            cache_dir: download_dir,
        };
        let downloader = HttpAppDownloader::from(params);

        let task = tokio::spawn(async move {
            if let Err(error) = downloader.download_executable().await {
                event_tx.send(UpdateEvent::DownloadFailed);
                return;
            }

            event_tx.send(UpdateEvent::Verifying);

            if let Err(error) = downloader.verify().await {
                event_tx.send(UpdateEvent::VerificationFailed);
                return;
            }

            event_tx.send(UpdateEvent::Verified {
                verified_installer_path: downloader.bin_path(),
            });
        });

        Ok(AppUpdater { task })
    }
}

struct ProgressUpdater {
    //began_download: Duration,
    url: String,
    event_tx: mpsc::Sender<UpdateEvent>,
    complete_frac: f32,
}

impl ProgressUpdater {
    fn new(url: String, event_tx: mpsc::Sender<UpdateEvent>) -> Self {
        Self {
            url,
            event_tx,
            complete_frac: 0.,
        }
    }
}

impl mullvad_update::fetch::ProgressUpdater for ProgressUpdater {
    fn set_url(&mut self, _url: &str) {
        // ignored since we already know the URL
    }

    fn set_progress(&mut self, fraction_complete: f32) {
        if (self.complete_frac - fraction_complete).abs() < 0.01 {
            return;
        }

        self.complete_frac = fraction_complete;

        self.event_tx.send(UpdateEvent::Downloading {
            complete_frac: fraction_complete,
            // TODO: estimate time left based on how much was downloaded (maybe in last n seconds)
            time_left: Duration::ZERO,
        });
    }

    fn clear_progress(&mut self) {
        self.complete_frac = 0.;

        self.event_tx.send(UpdateEvent::Downloading {
            complete_frac: 0.,
            // TODO: Check if this is reasonable
            time_left: Duration::ZERO,
        });
    }
}
