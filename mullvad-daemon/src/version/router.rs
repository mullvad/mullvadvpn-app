use std::future::Future;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;

use futures::channel::{mpsc, oneshot};
use futures::future::{Fuse, FusedFuture};
use futures::stream::StreamExt;
use futures::FutureExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::{AppVersionInfo, SuggestedUpgrade};
use talpid_core::mpsc::Sender;

use crate::DaemonEventSender;

use super::{
    check::{self, VersionCache, VersionUpdater},
    Error,
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
        let (result_tx, result_rx) = mpsc::unbounded();
        self.tx
            .send(Message::NewUpgradeEventListener { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        Ok(result_rx)
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
    version_request: Fuse<Pin<Box<dyn Future<Output = Result<VersionCache>> + Send>>>,
    version_request_channels: Vec<oneshot::Sender<Result<mullvad_types::version::AppVersionInfo>>>,
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
        result_tx: mpsc::UnboundedSender<mullvad_types::version::AppUpgradeEvent>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum RoutingState {
    /// There is no version available yet
    NoVersion,
    /// Running version checker, no upgrade in progress
    HasVersion { version_info: AppVersionInfo },
    /// Upgrade is in progress, so we don't forward version checks
    Upgrading {
        version_info: AppVersionInfo,
        /// Version check update received while paused
        new_version: Option<VersionCache>,
        //update_progress: mullvad_types::version::UpdateProgress,
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
                self.beta_program = state;
                // We're happy as soon as the internal state has changed; no need to wait for
                // version update
                let _ = result_tx.send(());
            }
            Message::GetLatestVersion(result_tx) => {
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
            Message::UpdateApplication { result_tx } => {
                self.update_application();
                let _ = result_tx.send(());
            }
            Message::CancelUpdate { result_tx } => {
                self.cancel_upgrade();
                let _ = result_tx.send(());
            }
            Message::NewUpgradeEventListener { result_tx } => {
                todo!();
            }
        }
    }

    fn update_application(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // Checking state: start upgrade, if upgrade is available
            RoutingState::HasVersion { version_info } => {
                // TODO: actually start update
                // TODO: check suggested upgrade
                log::debug!("Starting upgrade");
                self.state = RoutingState::Upgrading {
                    version_info,
                    new_version: None,
                };
            }
            // Already upgrading or no version: do nothing
            state => {
                self.state = state;
            }
        }
    }

    fn cancel_upgrade(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // No-op unless we're upgrading
            state @ RoutingState::NoVersion | state @ RoutingState::HasVersion { .. } => {
                self.state = state;
            }
            // If we're upgrading, emit an event if a version was received during the upgrade
            RoutingState::Upgrading {
                version_info,
                new_version,
            } => {
                self.state = RoutingState::HasVersion { version_info };

                // If we also received an upgrade, emit new version event
                if let Some(version) = new_version {
                    self.on_new_version(version);
                }
            }
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
                let app_version_info = to_app_version_info(version, self.beta_program);
                // Initial version is propagated
                let _ = self.version_event_sender.send(app_version_info.clone());
                self.state = RoutingState::HasVersion {
                    version_info: app_version_info,
                };
            }
            // Update app version info
            RoutingState::HasVersion {
                ref mut version_info,
            } => {
                let new_version = to_app_version_info(version, self.beta_program);
                // If the version changed, notify channel
                if &new_version != version_info {
                    let _ = self.version_event_sender.send(new_version.clone());
                    *version_info = new_version;
                }
            }
            // If we're upgrading, remember the new version, but don't send any notification
            RoutingState::Upgrading {
                ref mut new_version,
                ..
            } => {
                *new_version = Some(version);
            }
        }

        self.notify_version_requesters();
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
            RoutingState::HasVersion { version_info } => version_info,
            // If we're upgrading, remember the new version, but don't update app version info
            RoutingState::Upgrading { version_info, .. } => version_info,
        };

        // Notify all requesters
        for tx in self.version_request_channels.drain(..) {
            let _ = tx.send(Ok(version_info.clone()));
        }
    }
}

fn to_app_version_info(cache: VersionCache, beta_program: bool) -> AppVersionInfo {
    let version = if beta_program {
        cache
            .latest_version
            .beta
            .unwrap_or(cache.latest_version.stable)
    } else {
        cache.latest_version.stable
    };

    // TODO: if the current version is up to date, set suggested_upgrade to None

    mullvad_types::version::AppVersionInfo {
        current_version_supported: cache.current_version_supported,
        suggested_upgrade: Some(SuggestedUpgrade {
            version: version.version,
            changelog: Some(version.changelog),
            // TODO: This should return the downloaded & verified path, if it exists
            verified_installer_path: None,
        }),
    }
}
