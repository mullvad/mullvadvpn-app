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
use mullvad_update::app::{AppDownloader, AppDownloaderParameters, HttpAppDownloader};
use rand::seq::SliceRandom;
use std::time::Duration;
use std::{future::Future, path::PathBuf};
use tokio::fs;

use super::Error;
type Result<T> = std::result::Result<T, Error>;

pub struct Downloader(());

pub type AbortHandle = oneshot::Sender<()>;

/// App updater event
pub enum UpdateEvent {
    /// Download progress update
    Downloading {
        /// Server that the app is being downloaded from
        server: String,
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

impl Downloader {
    /// Begin or resume download of `version`
    pub async fn start(
        version: mullvad_update::version::Version,
        event_tx: mpsc::UnboundedSender<UpdateEvent>,
    ) -> Result<impl Future<Output = ()>> {
        let url = select_cdn_url(&version.urls)
            .ok_or(Error::NoUrlFound)?
            .to_owned();

        let download_dir = mullvad_paths::cache_dir()?.join("mullvad-update");
        fs::create_dir_all(&download_dir)
            .await
            .map_err(Error::CreateDownloadDir)?;

        let params = AppDownloaderParameters {
            app_version: version.version,
            app_url: url.clone(),
            app_size: version.size,
            app_progress: ProgressUpdater::new(server_from_url(&url), event_tx.clone()),
            app_sha256: version.sha256,
            cache_dir: download_dir,
        };
        let mut downloader = HttpAppDownloader::from(params);

        Ok(async move {
            if let Err(_error) = downloader.download_executable().await {
                let _ = event_tx.unbounded_send(UpdateEvent::DownloadFailed);
                return;
            }

            let _ = event_tx.unbounded_send(UpdateEvent::Verifying);

            if let Err(_error) = downloader.verify().await {
                let _ = event_tx.unbounded_send(UpdateEvent::VerificationFailed);
                return;
            }

            let _ = event_tx.unbounded_send(UpdateEvent::Verified {
                verified_installer_path: downloader.bin_path(),
            });
        })
    }
}

struct ProgressUpdater {
    //began_download: Duration,
    server: String,
    event_tx: mpsc::UnboundedSender<UpdateEvent>,
    complete_frac: f32,
}

impl ProgressUpdater {
    fn new(server: String, event_tx: mpsc::UnboundedSender<UpdateEvent>) -> Self {
        Self {
            server,
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

        let _ = self.event_tx.send(UpdateEvent::Downloading {
            server: self.server.clone(),
            complete_frac: fraction_complete,
            // TODO: estimate time left based on how much was downloaded (maybe in last n seconds)
            time_left: Duration::ZERO,
        });
    }

    fn clear_progress(&mut self) {
        self.complete_frac = 0.;

        let _ = self.event_tx.send(UpdateEvent::Downloading {
            server: self.server.clone(),
            complete_frac: 0.,
            // TODO: Check if this is reasonable
            time_left: Duration::ZERO,
        });
    }
}

/// Select a mirror to download from
/// Currently, the selection is random
fn select_cdn_url(urls: &[String]) -> Option<&str> {
    urls.choose(&mut rand::thread_rng()).map(String::as_str)
}

/// Extract domain name from a URL
fn server_from_url(url: &str) -> String {
    let url = url.strip_prefix("https://").unwrap_or(url);
    let (server, _) = url.split_once('/').unwrap_or((url, ""));
    server.to_owned()
}
