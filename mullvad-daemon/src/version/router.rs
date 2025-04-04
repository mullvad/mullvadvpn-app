use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use futures::channel::{mpsc, oneshot};
use futures::future::{Fuse, FusedFuture};
use futures::stream::StreamExt;
use futures::FutureExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::{AppVersionInfo, SuggestedUpgrade};
use mullvad_update::version::VersionInfo;
use talpid_core::mpsc::Sender;

use crate::management_interface::AppUpgradeBroadcast;
use crate::DaemonEventSender;

use super::{
    check::{self, VersionCache, VersionUpdater},
    Error,
};

#[cfg(update)]
use super::downloader;
#[cfg(update)]
use std::mem;

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

    pub async fn get_latest_version(&self) -> Result<AppVersionInfo> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::GetLatestVersion(result_tx))
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)?
    }

    #[cfg(update)]
    pub async fn update_application(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::UpdateApplication { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    #[cfg(update)]
    pub async fn cancel_update(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::CancelUpdate { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
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
    version_event_sender: DaemonEventSender<AppVersionInfo>,
    /// Version updater
    version_check: check::VersionUpdaterHandle,
    /// Channel used to receive updates from `version_check`
    new_version_rx: mpsc::UnboundedReceiver<VersionCache>,
    /// Future that resolves when `get_latest_version` resolves
    version_request: Fuse<Pin<Box<dyn Future<Output = Result<VersionCache>> + Send>>>,
    /// Channels that receive responses to `get_latest_version`
    version_request_channels: Vec<oneshot::Sender<Result<AppVersionInfo>>>,

    /// Broadcast channel for app upgrade events
    app_upgrade_broadcast: AppUpgradeBroadcast,
}

enum Message {
    /// Enable or disable beta program
    SetBetaProgram {
        state: bool,
        result_tx: oneshot::Sender<()>,
    },
    /// Check for updates
    GetLatestVersion(oneshot::Sender<Result<AppVersionInfo>>),
    /// Update the application
    #[cfg(update)]
    UpdateApplication { result_tx: oneshot::Sender<()> },
    /// Cancel the ongoing update
    #[cfg(update)]
    CancelUpdate { result_tx: oneshot::Sender<()> },
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
        /// The version being upgraded to, derived from `version_info` and beta program state
        upgrading_to_version: mullvad_update::version::Version,
        /// Tokio task for the downloader handle
        downloader_handle:
            tokio::task::JoinHandle<std::result::Result<std::path::PathBuf, downloader::Error>>,
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
        version_event_sender: DaemonEventSender<AppVersionInfo>,
        beta_program: bool,
        app_upgrade_broadcast: AppUpgradeBroadcast,
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
                app_upgrade_broadcast,
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
                res = wait_for_update(&mut self.state) => {
                    // If the download was successful, we send the new version
                    if let Some(app_update_info) =  res {
                        let _ = self.version_event_sender.send(app_update_info);
                    }
                },
                Some(message) = self.rx.next() => self.handle_message(message).await,
                else => break,
            }
        }
        log::info!("Version router closed");
    }

    /// Handle [Message] sent by user
    #[cfg_attr(not(update), allow(clippy::unused_async))]
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
            #[cfg(update)]
            Message::UpdateApplication { result_tx } => {
                self.update_application();
                let _ = result_tx.send(());
            }
            #[cfg(update)]
            Message::CancelUpdate { result_tx } => {
                self.cancel_upgrade().await;
                let _ = result_tx.send(());
            }
        }
    }

    fn set_beta_program(&mut self, new_state: bool) {
        let prev_state = self.beta_program;
        if new_state == prev_state {
            return;
        }
        self.on_version_or_beta_change(None, new_state);
    }

    fn get_latest_version(
        &mut self,
        result_tx: oneshot::Sender<std::result::Result<AppVersionInfo, Error>>,
    ) {
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

    #[cfg(update)]
    fn update_application(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // Checking state: start upgrade, if upgrade is available
            RoutingState::HasVersion { version_info } => {
                let Some(upgrading_to_version) =
                    recommended_version_upgrade(&version_info.latest_version, self.beta_program)
                else {
                    // If there's no suggested upgrade, do nothing
                    log::trace!("Received update request without suggested upgrade");
                    self.state = RoutingState::HasVersion { version_info };
                    return;
                };

                let downloader_handle = tokio::spawn(downloader::Downloader::start(
                    upgrading_to_version.clone(),
                    self.app_upgrade_broadcast.clone(),
                ));

                log::debug!("Starting upgrade");
                self.state = RoutingState::Downloading {
                    version_info,
                    upgrading_to_version,
                    downloader_handle,
                };
            }
            // Already downloading/downloaded or there is no version: do nothing
            state => {
                self.state = state;
            }
        }
    }

    #[cfg(update)]
    async fn cancel_upgrade(&mut self) {
        match mem::replace(&mut self.state, RoutingState::NoVersion) {
            // If we're upgrading, emit an event if a version was received during the upgrade
            // Otherwise, just reset upgrade info to last known state
            RoutingState::Downloading {
                version_info,
                upgrading_to_version: _,
                downloader_handle,
            } => {
                // Abort download
                downloader_handle.abort();
                let _ = downloader_handle.await;

                // Reset app version info to last known state
                self.state = RoutingState::HasVersion { version_info };
            }
            // No-op unless we're downloading something right now
            // In the `Downloaded` state, we also do nothing
            state => self.state = state,
        };
    }

    /// Handle new version info
    ///
    /// If the router is in the process of upgrading, it will not propagate versions, but only
    /// remember it for when it transitions back into the "idle" (version check) state.
    fn on_new_version(&mut self, new_version: VersionCache) {
        self.on_version_or_beta_change(Some(new_version), self.beta_program);
    }

    fn on_version_or_beta_change(
        &mut self,
        new_version: Option<VersionCache>,
        new_beta_state: bool,
    ) {
        let prev_beta_state = self.beta_program;
        self.beta_program = new_beta_state;

        let new_app_version_info: Option<AppVersionInfo> =
            match mem::replace(&mut self.state, RoutingState::NoVersion) {
                // Set app version info
                RoutingState::NoVersion => {
                    if let Some(new_version) = new_version {
                        // Initial version is propagated
                        let app_version_info =
                            to_app_version_info(&new_version, self.beta_program, None);
                        self.state = RoutingState::HasVersion {
                            version_info: new_version,
                        };
                        Some(app_version_info)
                    } else {
                        None
                    }
                }
                // Update app version info
                RoutingState::HasVersion {
                    version_info: prev_version,
                    ..
                } => {
                    let new_version = new_version.as_ref().unwrap_or(&prev_version);
                    // Update version info
                    self.state = RoutingState::HasVersion {
                        version_info: new_version.clone(),
                    };
                    get_updated_app_version_info(
                        new_version,
                        &prev_version,
                        new_beta_state,
                        prev_beta_state,
                    )
                }
                // If we're upgrading, abort if the recommended version changes
                RoutingState::Downloading {
                    version_info: prev_version,
                    downloader_handle,
                    ..
                } => {
                    let new_version = new_version.as_ref().unwrap_or(&prev_version);

                    get_updated_app_version_info(
                        new_version,
                        &prev_version,
                        new_beta_state,
                        prev_beta_state,
                    )
                    .inspect(|new_app_version| {
                        downloader_handle.abort();

                        // Update version info
                        self.state = RoutingState::HasVersion {
                            version_info: new_version.clone(),
                        };
                        log::warn!(
                            "Received new version while upgrading: {new_app_version:?}, aborting"
                        );
                        let _ = self
                            .app_upgrade_broadcast
                            .send(mullvad_types::version::AppUpgradeEvent::Aborted);
                    })
                }
                RoutingState::Downloaded {
                    version_info: prev_version,
                    ..
                } => {
                    let new_version = new_version.as_ref().unwrap_or(&prev_version);
                    get_updated_app_version_info(
                        new_version,
                        &prev_version,
                        new_beta_state,
                        prev_beta_state,
                    )
                    .inspect(|new_app_version| {
                        // Update version info
                        self.state = RoutingState::HasVersion {
                            version_info: new_version.clone(),
                        };
                        log::warn!(
                            "Received new version while upgrading: {new_app_version:?}, aborting"
                        );
                        let _ = self
                            .app_upgrade_broadcast
                            .send(mullvad_types::version::AppUpgradeEvent::Aborted);
                    })
                }
            };

        if let Some(version_info) = new_app_version_info {
            let _ = self.version_event_sender.send(version_info.clone());

            // Cancel update notifications
            self.version_request = Fuse::terminated();
            // Notify all requesters
            for tx in self.version_request_channels.drain(..) {
                let _ = tx.send(Ok(version_info.clone()));
            }
        }
    }
}

fn get_updated_app_version_info(
    prev_version: &VersionCache,
    new_version: &VersionCache,
    new_beta_state: bool,
    prev_beta_state: bool,
) -> Option<AppVersionInfo> {
    let prev_app_version = to_app_version_info(prev_version, prev_beta_state, None);
    let new_app_version = to_app_version_info(new_version, new_beta_state, None);

    // Update version info
    if new_app_version != prev_app_version {
        Some(new_app_version)
    } else {
        None
    }
}

/// Wait for the update to finish. In case no update is in progress (or the platform does not
/// support in-app upgrades), then the future will never resolve as to not escape the select statement.
#[allow(clippy::unused_async, unused_variables)]
async fn wait_for_update(state: &mut RoutingState) -> Option<AppVersionInfo> {
    #[cfg(update)]
    match mem::replace(state, RoutingState::NoVersion) {
        RoutingState::Downloading {
            version_info,
            downloader_handle,
            upgrading_to_version,
            ..
        } => match downloader_handle.await {
            Ok(Ok(verified_installer_path)) => {
                let app_update_info = AppVersionInfo {
                    current_version_supported: version_info.current_version_supported,
                    suggested_upgrade: Some({
                        SuggestedUpgrade {
                            version: upgrading_to_version.version.clone(),
                            changelog: upgrading_to_version.changelog.clone(),
                            verified_installer_path: Some(verified_installer_path.clone()),
                        }
                    }),
                };
                *state = RoutingState::Downloaded {
                    version_info,
                    verified_installer_path,
                };
                // TODO: send version info + version checks + upgrade event
                Some(app_update_info)
            }
            Ok(Err(_err)) => {
                *state = RoutingState::HasVersion { version_info };
                None
            }
            Err(join_err) => {
                log::error!("Downloader task ended unexpectedly: {join_err}");
                *state = RoutingState::HasVersion { version_info };
                None
            }
        },
        other_state => {
            // Revert to original state
            *state = other_state;
            let () = std::future::pending().await;
            unreachable!()
        }
    }
    #[cfg(not(update))]
    {
        let () = std::future::pending().await;
        unreachable!()
    }
}

/// Extract [`AppVersionInfo`], containing upgrade version and `current_version_supported`
/// from [VersionCache] and beta program state.
fn to_app_version_info(
    cache: &VersionCache,
    beta_program: bool,
    verified_installer_path: Option<PathBuf>,
) -> AppVersionInfo {
    let current_version_supported = cache.current_version_supported;
    let suggested_upgrade = recommended_version_upgrade(&cache.latest_version, beta_program)
        .as_ref()
        .map(|version| suggested_upgrade_for_version(version, verified_installer_path));
    AppVersionInfo {
        current_version_supported,
        suggested_upgrade,
    }
}

/// Extract upgrade version from [VersionCache] based on `beta_program`
fn recommended_version_upgrade(
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
    verified_installer_path: Option<PathBuf>,
) -> SuggestedUpgrade {
    SuggestedUpgrade {
        version: version_details.version.clone(),
        changelog: version_details.changelog.clone(),
        verified_installer_path,
    }
}
