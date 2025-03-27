// TODO:
/*
If a new upgrade version becomes available during an app upgrade it should not affect the suggested upgrade version in the SuggestedUpgrade message if the upgrade is still in progress. That is, if the current state is one of
AppUpgradeDownloadStarting
AppUpgradeDownloadProgress
AppUpgradeVerifyingInstaller
*/

// TODO: How should interruptions be handled?
// Probably magically resume when going from Idle/DownloadFailed -> Downloading

// TODO: handle abort
// TODO: handle beginning/resuming download

use std::path::PathBuf;
use std::time::Duration;
use futures::channel::mpsc;
use mullvad_update::app::AppDownloaderParameters;

pub struct AppUpdater {
    state: UpdateState,
    event_tx: mpsc::Sender<UpdateStateEvent>,
}

pub enum UpdaterCommand {
    /// Download or resume download
    DownloadAndVerify {
        
    },
    /// Abort the current operation
    Abort,
}

/// App updater state
pub enum UpdateState {
    Idle,
    /// An installer is being downloaded
    Downloading {
        /// A fraction in `[0,1]` that describes how much of the installer has been downloaded
        complete_frac: f32,
        /// Estimated time left
        time_left: Duration,
    },
    /// Failed to download installer
    DownloadFailed,
    /// The downloaded installer is being verified
    Verifying,
    /// VerificationFailed
    VerificationFailed,
    /// There is a downloaded and verified installer available
    Verified {
        verified_installer_path: PathBuf,
    },
}

pub type UpdateStateEvent = UpdateState;

impl AppUpdater {
    pub fn spawn(
        event_tx: mpsc::Sender<UpdateStateEvent>,
    ) -> mpsc::Sender<UpdaterCommand> {
        let (event_tx, event_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            Self {
                state: UpdateState::Idle,
                event_tx,
            }
            .run(event_rx)
            .await
        });
        updater
    }

    fn new(event_tx: mpsc::Sender<UpdateStateEvent>) -> Self {
        Self {
            state: UpdateState::Idle,
            event_tx,
        }
    }

    fn begin_download(&mut self, version: mullvad_update::version::Version) {
        let params = AppDownloaderParameters {
            app_version: version.version,
            // TODO: select url
            app_url: version.urls[0],
            app_size: version.size,
            app_progress: ProgressUpdater::default(),
            app_sha256: version.sha256,
            // TODO: mkdir
            cache_dir: cache_dir().join("mullvad-update"),
        };
        let downloader = HttpAppDownloader::from(params);

        // TODO: begin download + verify
    }

    async fn run(mut self, mut event_rx: mpsc::Receiver<UpdateStateEvent>) {
        loop {
            match self.state {
                UpdateState::Idle => {
                    // Check for updates
                    // If update available, download
                    // If download fails, go to DownloadFailed
                    // If download succeeds, go to Verifying
                }
                UpdateState::Downloading { .. } => {
                    // Check for download progress
                    // If download fails, go to DownloadFailed
                    // If download succeeds, go to Verifying
                }
                UpdateState::DownloadFailed => {
                    // Retry download
                }
                UpdateState::Verifying => {
                    // Check for verification progress
                    // If verification fails, go to VerificationFailed
                    // If verification succeeds, go to Verified
                }
                UpdateState::VerificationFailed => {
                    // Retry verification
                }
                UpdateState::Verified { .. } => {
                    // Notify user that an update is available
                    // If user accepts, start upgrade
                }
            }
            if let Some(event) = event_rx.next().await {
                self.state = event;
            }
        }
    }
}

#[derive(Default)]
struct ProgressUpdater {
    //began_download: Duration,
    complete_frac: f32,
}

impl mullvad_update::client::fetch::ProgressUpdater for ProgressUpdater {
    fn set_url(&mut self, url: &str) {}

    fn set_progress(&mut self, fraction_complete: f32) {
        if (self.complete_frac - fraction_complete).abs() < 0.01 {
            return;
        }

        self.complete_frac = fraction_complete;

        // TODO: estimate time left based on how much was downloaded (maybe in last n seconds)
        // TODO: emit Downloading event
    }

    fn clear_progress(&mut self) {
        self.complete_frac = 0.;
        // TODO: emit Downloading event
    }
}
