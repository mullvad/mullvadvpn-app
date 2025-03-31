use std::future::Future;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;

use futures::channel::{mpsc, oneshot};
use futures::future::{Fuse, FusedFuture};
use futures::stream::StreamExt;
use futures::FutureExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::{AppUpgradeEvent, AppVersionInfo, SuggestedUpgrade};
use mullvad_update::version::VersionInfo;
use talpid_core::mpsc::Sender;

use crate::DaemonEventSender;

use super::{
    check::{self, VersionCache, VersionUpdater},
    downloader, Error,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct VersionRouterHandle {
    tx: mpsc::UnboundedSender<Message>,
}

impl VersionRouterHandle {
    pub async fn set_show_beta_releases(&self, state: bool) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::SetBetaProgram { state, result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    pub async fn get_latest_version(&self) -> Result<mullvad_types::version::AppVersionInfo> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::GetLatestVersion(result_tx))
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)?
    }

    pub async fn update_application(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::UpdateApplication { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    pub async fn cancel_update(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::CancelUpdate { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    pub fn new_upgrade_event_listener(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<mullvad_types::version::AppUpgradeEvent>> {
        let (event_tx, event_rx) = mpsc::unbounded();
        self.tx
            .send(Message::NewUpgradeEventListener { event_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        Ok(event_rx)
    }
}

/// Router of version updates and update requests.
///
/// New available app version events are forwarded from the [`VersionUpdater`].
/// If an update is in progress, these events are paused until the update is completed or canceled.
/// This is done to prevent frontends from confusing which version is currently being installed,
/// in case new version info is received while the update is in progress.
pub struct VersionRouter {
    rx: mpsc::UnboundedReceiver<Message>,
    state: RoutingState,
    beta_program: bool,
    version_event_sender: DaemonEventSender<mullvad_types::version::AppVersionInfo>,
    /// Version updater
    version_check: check::VersionUpdaterHandle,
    /// Channel used to receive updates from `version_check`
    new_version_rx: mpsc::UnboundedReceiver<VersionCache>,
    /// Future that resolves when `get_latest_version` resolves
    version_request: Fuse<Pin<Box<dyn Future<Output = Result<VersionCache>> + Send>>>,
    /// Channels that receive responses to `get_latest_version`
    version_request_channels: Vec<oneshot::Sender<Result<mullvad_types::version::AppVersionInfo>>>,
    /// Channel used to send upgrade events from [downloader::Downloader]
    update_event_tx: mpsc::UnboundedSender<downloader::UpdateEvent>,
    /// Channel used to receive upgrade events from [downloader::Downloader]
    update_event_rx: mpsc::UnboundedReceiver<downloader::UpdateEvent>,
    /// Clients that will also receive events
    upgrade_listeners: Vec<mpsc::UnboundedSender<AppUpgradeEvent>>,
}

enum Message {
    /// Enable or disable beta program
    SetBetaProgram {
        state: bool,
        result_tx: oneshot::Sender<()>,
    },
    /// Check for updates
    GetLatestVersion(oneshot::Sender<Result<mullvad_types::version::AppVersionInfo>>),
    /// Update the application
    UpdateApplication { result_tx: oneshot::Sender<()> },
    /// Cancel the ongoing update
    CancelUpdate { result_tx: oneshot::Sender<()> },
    /// Listen for events
    NewUpgradeEventListener {
        /// Channel for receiving update events
        event_tx: mpsc::UnboundedSender<AppUpgradeEvent>,
    },
}

#[derive(Debug)]
enum RoutingState {
    /// There is no version available yet
    NoVersion,
    /// Running version checker, no upgrade in progress
    HasVersion { version_info: VersionCache },
    /// Download is in progress, so we don't forward version checks
    Downloading {
        /// Version info received from `HasVersion`
        version_info: VersionCache,
        /// The version being upgraded to (derived from `suggested_upgrade`).
        /// Should be one of the versions in `version_info`.
        upgrading_to_version: mullvad_update::version::Version,
        /// Version check update received while paused
        /// When transitioning out of `Upgrading`, this will cause `version_info` to be updated
        new_version: Option<VersionCache>,
        /// Tokio task for the downloader handle
        downloader_handle: tokio::task::JoinHandle<()>,
    },
    /// Download is complete. We have a verified binary
    Downloaded {
        /// Version info received from `HasVersion`
        version_info: VersionCache,
        /// Path to verified installer
        verified_installer_path: PathBuf,
    },
}

impl VersionRouter {
    pub(crate) fn spawn(
        api_handle: MullvadRestHandle,
        availability_handle: ApiAvailability,
        cache_dir: PathBuf,
        version_event_sender: DaemonEventSender<mullvad_types::version::AppVersionInfo>,
        beta_program: bool,
    ) -> VersionRouterHandle {
        let (tx, rx) = mpsc::unbounded();

        tokio::spawn(async move {
            let (new_version_tx, new_version_rx) = mpsc::unbounded();
            let version_check =
                VersionUpdater::spawn(api_handle, availability_handle, cache_dir, new_version_tx)
                    .await;

            let (update_event_tx, update_event_rx) = mpsc::unbounded();

            // TODO: tokio::join! here?
            Self {
                rx,
                state: RoutingState::NoVersion,
                beta_program,
                version_check,
                version_event_sender,
                new_version_rx,
                version_request: Fuse::terminated(),
                version_request_channels: vec![],
                update_event_tx,
                update_event_rx,
                upgrade_listeners: vec![],
            }
            .run()
            .await;
        });
        VersionRouterHandle { tx }
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                // Respond to version info requests
                update_result = &mut self.version_request => {
                    if self.version_request.is_terminated() {
                        log::trace!("Version info future is terminated");
                        continue;
                    }
                    match update_result {
                        Ok(new_version) => {
                            self.on_new_version(new_version.clone());
                        }
                        Err(error) => {
                            log::error!("Failed to retrieve version: {error}");
                            for tx in self.version_request_channels.drain(..) {
                                // TODO: More appropriate error? But Error isn't Clone
                                let _ = tx.send(Err(Error::UpdateAborted));
                            }
                        }
                    }
                }
                // Received version event from `check`
                Some(new_version) = self.new_version_rx.next() => {
                    self.on_new_version(new_version);
                }
                // Received upgrade event from `downloader`
                Some(update_event) = self.update_event_rx.next() => {
                    self.handle_update_event(update_event);
                }
                Some(message) = self.rx.next() => self.handle_message(message).await,
                else => break,
            }
        }
        log::info!("Version router closed");
    }

    /// Handle [Message] sent by user
    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::SetBetaProgram { state, result_tx } => {
                self.set_beta_program(state);
                // We're happy as soon as the internal state has changed; no need to wait for
                // version update
                let _ = result_tx.send(());
            }
            Message::GetLatestVersion(result_tx) => {
                self.get_latest_version(result_tx);
            }
            Message::UpdateApplication { result_tx } => {
                self.update_application().await;
                let _ = result_tx.send(());
            }
            Message::CancelUpdate { result_tx } => {
                self.cancel_upgrade().await;
                let _ = result_tx.send(());
            }
            Message::NewUpgradeEventListener {
                event_tx: result_tx,
            } => {
                self.upgrade_listeners.push(result_tx);
            }
        }
    }

    fn set_beta_program(&mut self, new_state: bool) {
        let prev_state = self.beta_program;
        if new_state == prev_state {
            return;
        }

        match &self.state {
            // Emit version event if suggested upgrade changes
            RoutingState::HasVersion { version_info }
            | RoutingState::Downloaded { version_info, .. } => {
                let prev_app_version_info = to_app_version_info(version_info, prev_state);
                let new_app_version_info = to_app_version_info(version_info, new_state);

                if new_app_version_info != prev_app_version_info {
                    let _ = self.version_event_sender.send(new_app_version_info);

                    // Note: If we're in the `Downloaded` state, this resets the state to `HasVersion`
                    self.state = RoutingState::HasVersion {
                        version_info: version_info.clone(),
                    };

                    self.notify_version_requesters();
                }
            }
            // If there's no version or upgrading, do nothing
            RoutingState::NoVersion | RoutingState::Downloading { .. } => (),
        }
    }

    fn get_latest_version(
        &mut self,
        result_tx: oneshot::Sender<std::result::Result<AppVersionInfo, Error>>,
    ) {
        match &self.state {
            // When not upgrading, potentially fetch new version info, and append `result_tx` to
            // list of channels to notify.
            // We don't wait on `get_version_info` so that we don't block user commands.
            RoutingState::NoVersion
            | RoutingState::HasVersion { .. }
            | RoutingState::Downloaded { .. } => {
                // Start a version request unless already in progress
                if self.version_request.is_terminated() {
                    let check = self.version_check.clone();
                    let check_fut: Pin<Box<dyn Future<Output = Result<VersionCache>> + Send>> =
                        Box::pin(async move { check.get_version_info().await });
                    self.version_request = check_fut.fuse();
                }
                // Append to response channels
                self.version_request_channels.push(result_tx);
            }
            // During upgrades, just pass on the last known version
            RoutingState::Downloading {
                version_info,
                upgrading_to_version,
                new_version: _,
                downloader_handle: _,
            } => {
                let suggested_upgrade = suggested_upgrade_for_version(upgrading_to_version);
                let info = AppVersionInfo {
                    current_version_supported: version_info.current_version_supported,
                    suggested_upgrade: Some(suggested_upgrade),
                };
                let _ = result_tx.send(Ok(info));
            }
        }
    }

    async fn update_application(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // Checking state: start upgrade, if upgrade is available
            RoutingState::HasVersion { version_info } => {
                let Some(suggested_upgrade) =
                    suggested_upgrade(&version_info.latest_version, self.beta_program)
                else {
                    // If there's no suggested upgrade, do nothing
                    log::trace!("Received update request without suggested upgrade");
                    self.state = RoutingState::HasVersion { version_info };
                    return;
                };

                let downloader_handle = tokio::spawn(
                    downloader::Downloader::start(
                        suggested_upgrade.clone(),
                        self.update_event_tx.clone(),
                    )
                    .await
                    .expect("TODO: handle err"),
                );

                log::debug!("Starting upgrade");
                self.state = RoutingState::Downloading {
                    version_info,
                    upgrading_to_version: suggested_upgrade,
                    new_version: None,
                    downloader_handle,
                };

                // Notify callers of `get_latest_version`: cancel the version check and
                // advertise the last known version as latest
                self.notify_version_requesters();
            }
            // Already downloading/downloaded or there is no version: do nothing
            state => {
                self.state = state;
            }
        }
    }

    async fn cancel_upgrade(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // If we're upgrading, emit an event if a version was received during the upgrade
            // Otherwise, just reset upgrade info to last known state
            RoutingState::Downloading {
                version_info,
                upgrading_to_version: _,
                new_version,
                downloader_handle,
            } => {
                // Abort download
                downloader_handle.abort();
                let _ = downloader_handle.await;

                // Reset app version info to last known state
                self.state = RoutingState::HasVersion { version_info };

                // If we also received an upgrade, emit new version event
                if let Some(version) = new_version {
                    self.on_new_version(version);
                }
            }
            // No-op unless we're downloading something right now
            // In the `Downloaded` state, we also do nothing
            state => self.state = state,
        };
    }

    /// Handle new version info
    ///
    /// If the router is in the process of upgrading, it will send not propagate versions, but only
    /// remember it for when it transitions back into the "idle" (version check) state.
    fn on_new_version(&mut self, version: VersionCache) {
        match &mut self.state {
            // Set app version info
            RoutingState::NoVersion => {
                self.state = RoutingState::HasVersion {
                    version_info: version.clone(),
                };

                // Initial version is propagated
                let app_version_info = to_app_version_info(&version, self.beta_program);
                let _ = self.version_event_sender.send(app_version_info);
            }
            // Update app version info
            RoutingState::HasVersion { .. } | RoutingState::Downloaded { .. } => {
                // If the version changed, notify channel
                let prev_version = to_app_version_info(&version, self.beta_program);
                let new_version = to_app_version_info(&version, self.beta_program);
                if new_version != prev_version {
                    let _ = self.version_event_sender.send(new_version.clone());

                    // Note: If we're in the `Downloaded` state, this resets the state to `HasVersion`
                    self.state = RoutingState::HasVersion {
                        version_info: version,
                    };
                }
            }
            // If we're upgrading, remember the new version, but don't send any notification
            RoutingState::Downloading {
                ref mut new_version,
                ..
            } => {
                *new_version = Some(version);
            }
        }

        // Notfify callers of `get_latest_version`
        self.notify_version_requesters();
    }

    fn handle_update_event(&mut self, event: downloader::UpdateEvent) {
        debug_assert!(
            matches!(self.state, RoutingState::Downloading { .. }),
            "unexpected routing state: {:?}",
            self.state
        );

        use downloader::UpdateEvent;

        match event {
            UpdateEvent::Downloading {
                server,
                complete_frac: f32,
                time_left,
            } => {
                // TODO: emit version event to clients
            }
            UpdateEvent::DownloadFailed => {
                // TODO: transition to HasVersion state
                // TODO: emit version event to clients
            }
            UpdateEvent::Verifying => {
                // TODO: emit version event to clients
            }
            UpdateEvent::VerificationFailed => {
                // TODO: transition to HasVersion state
                // TODO: emit version event to clients
            }
            UpdateEvent::Verified {
                verified_installer_path,
            } => {
                // TODO: transition to Downloaded state
                // TODO: emit version event to clients
            }
        }
    }

    /// Notify clients requesting a version
    fn notify_version_requesters(&mut self) {
        // Cancel update notifications
        self.version_request = Fuse::terminated();

        let version_info = match &self.state {
            RoutingState::NoVersion => {
                log::error!("Dropping version request channels since there's no version");
                self.version_request_channels.clear();
                return;
            }
            // Update app version info
            RoutingState::HasVersion { version_info }
            | RoutingState::Downloaded { version_info, .. } => {
                to_app_version_info(version_info, self.beta_program)
            }
            // If we're upgrading, emit the version we're currently upgrading to
            RoutingState::Downloading {
                version_info,
                upgrading_to_version,
                ..
            } => {
                let suggested_upgrade = suggested_upgrade_for_version(upgrading_to_version);
                AppVersionInfo {
                    current_version_supported: version_info.current_version_supported,
                    suggested_upgrade: Some(suggested_upgrade),
                }
            }
        };

        // Notify all requesters
        for tx in self.version_request_channels.drain(..) {
            let _ = tx.send(Ok(version_info.clone()));
        }
    }
}

fn to_app_version_info(cache: &VersionCache, beta_program: bool) -> AppVersionInfo {
    let current_version_supported = cache.current_version_supported;
    let suggested_upgrade = suggested_upgrade(&cache.latest_version, beta_program)
        .as_ref()
        .map(suggested_upgrade_for_version);
    AppVersionInfo {
        current_version_supported,
        suggested_upgrade,
    }
}

/// Extract upgrade version from [VersionCache] based on `beta_program`
fn suggested_upgrade(
    version_info: &VersionInfo,
    beta_program: bool,
) -> Option<mullvad_update::version::Version> {
    let version_details = if beta_program {
        version_info.beta.as_ref().unwrap_or(&version_info.stable)
    } else {
        &version_info.stable
    };

    // Set suggested upgrade if the received version is newer than the current version
    let current_version = mullvad_version::VERSION.parse().unwrap();
    if version_details.version > current_version {
        Some(version_details.to_owned())
    } else {
        None
    }
}

/// Convert [mullvad_update::version::Version] to [SuggestedUpgrade]
fn suggested_upgrade_for_version(
    version_details: &mullvad_update::version::Version,
) -> SuggestedUpgrade {
    SuggestedUpgrade {
        version: version_details.version.clone(),
        changelog: Some(version_details.changelog.clone()),
        // TODO: This should return the downloaded & verified path, if it exists
        verified_installer_path: None,
    }
}
