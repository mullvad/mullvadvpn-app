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
struct VersionRouter<S = DaemonEventSender<AppVersionInfo>> {
    daemon_rx: mpsc::UnboundedReceiver<Message>,
    state: State,
    beta_program: bool,
    version_event_sender: S,
    /// Version updater
    version_check: check::VersionUpdaterHandle,
    /// Channel used to receive updates from `version_check`
    new_version_rx: mpsc::UnboundedReceiver<VersionCache>,
    /// Future that resolves when `get_latest_version` resolves
    version_request: Fuse<Pin<Box<dyn Future<Output = Result<VersionCache>> + Send>>>,
    /// Channels that receive responses to `get_latest_version`
    version_request_channels: Vec<oneshot::Sender<Result<AppVersionInfo>>>,

    /// Broadcast channel for app upgrade events
    #[cfg(update)]
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
enum State {
    /// There is no version available yet
    NoVersion,
    /// Running version checker, no upgrade in progress
    HasVersion { version_cache: VersionCache },
    /// Download is in progress, so we don't forward version checks
    #[cfg(update)]
    Downloading {
        /// Version info received from `HasVersion`
        version_cache: VersionCache,
        /// The version being upgraded to, derived from `version_info` and beta program state
        upgrading_to_version: mullvad_update::version::Version,
        /// Tokio task for the downloader handle
        downloader_handle: downloader::DownloaderHandle,
    },
    /// Download is complete. We have a verified binary
    #[cfg(update)]
    Downloaded {
        /// Version info received from `HasVersion`
        version_cache: VersionCache,
        /// Path to verified installer
        verified_installer_path: PathBuf,
    },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::NoVersion => write!(f, "NoVersion"),
            State::HasVersion { .. } => write!(f, "HasVersion"),
            #[cfg(update)]
            State::Downloading {
                upgrading_to_version,
                ..
            } => write!(f, "Downloading '{}'", upgrading_to_version.version),
            #[cfg(update)]
            State::Downloaded {
                verified_installer_path,
                ..
            } => write!(f, "Downloaded '{}'", verified_installer_path.display()),
        }
    }
}

impl State {
    fn get_version_cache(&self) -> Option<&VersionCache> {
        match self {
            State::NoVersion => None,
            State::HasVersion { version_cache, .. } => Some(version_cache),
            #[cfg(update)]
            State::Downloading { version_cache, .. } | State::Downloaded { version_cache, .. } => {
                Some(version_cache)
            }
        }
    }

    fn get_verified_installer_path(&self) -> Option<&PathBuf> {
        match self {
            #[cfg(update)]
            State::Downloaded {
                verified_installer_path,
                ..
            } => Some(verified_installer_path),
            _ => None,
        }
    }
}

#[cfg_attr(not(update), allow(unused_variables))]
pub(crate) fn spawn_version_router(
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
            VersionUpdater::spawn(api_handle, availability_handle, cache_dir, new_version_tx).await;

        VersionRouter {
            daemon_rx: rx,
            state: State::NoVersion,
            beta_program,
            version_check,
            version_event_sender,
            new_version_rx,
            version_request: Fuse::terminated(),
            version_request_channels: vec![],
            #[cfg(update)]
            app_upgrade_broadcast,
        }
        .run()
        .await;
    });
    VersionRouterHandle { tx }
}

impl<S: Sender<AppVersionInfo> + Send + 'static> VersionRouter<S> {
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
                Some(message) = self.daemon_rx.next() => self.handle_message(message),
                else => break,
            }
        }
        log::info!("Version router closed");
    }

    /// Handle [Message] sent by user
    fn handle_message(&mut self, message: Message) {
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
                self.cancel_upgrade();
                let _ = result_tx.send(());
            }
        }
    }

    /// Handle new version info
    ///
    /// If the router is in the process of upgrading, it will not propagate versions, but only
    /// remember it for when it transitions back into the "idle" (version check) state.
    fn on_new_version(&mut self, version_cache: VersionCache) {
        match &mut self.state {
            State::NoVersion => {
                // Receive first version
                let app_version_info = to_app_version_info(&version_cache, self.beta_program, None);
                let _ = self.version_event_sender.send(app_version_info.clone());
                self.state = State::HasVersion { version_cache };
            }
            // Already have version info, just update it
            State::HasVersion {
                version_cache: prev_cache,
            } => {
                if let Some(version_info) = updated_app_version_info_on_new_version_cache(
                    prev_cache,
                    &version_cache,
                    self.beta_program,
                ) {
                    // New version available
                    let _ = self.version_event_sender.send(version_info.clone());
                }
                self.state = State::HasVersion { version_cache };
            }
            #[cfg(update)]
            State::Downloaded {
                version_cache: ref mut prev_cache,
                ..
            }
            | State::Downloading {
                version_cache: ref mut prev_cache,
                ..
            } => {
                // If version changed, cancel download
                if let Some(version_info) = updated_app_version_info_on_new_version_cache(
                    prev_cache,
                    &version_cache,
                    self.beta_program,
                ) {
                    log::warn!("Received new version while upgrading: {version_info:?}, aborting");

                    let _ = self.version_event_sender.send(version_info.clone());
                    self.state = State::HasVersion { version_cache };
                } else {
                    *prev_cache = version_cache;
                }
            }
        }

        // Notify version requesters
        if let Some(cache) = self.state.get_version_cache() {
            self.notify_version_requesters(to_app_version_info(
                cache,
                self.beta_program,
                self.state.get_verified_installer_path().cloned(),
            ));
        }
    }

    fn notify_version_requesters(&mut self, new_app_version_info: AppVersionInfo) {
        // Cancel update notifications
        self.version_request = Fuse::terminated();
        // Notify all requesters
        for tx in self.version_request_channels.drain(..) {
            let _ = tx.send(Ok(new_app_version_info.clone()));
        }
    }

    fn set_beta_program(&mut self, new_state: bool) {
        if new_state == self.beta_program {
            return;
        }
        let previous_state = self.beta_program;
        self.beta_program = new_state;
        let Some(new_app_version_info) = self.state.get_version_cache().and_then(|version_cache| {
            updated_app_version_info_on_new_beta(version_cache, previous_state, new_state)
        }) else {
            return;
        };

        // Always cancel download if the suggested upgrade changes

        let version_cache = match mem::replace(&mut self.state, State::NoVersion) {
            #[cfg(update)]
            State::Downloaded { version_cache, .. } | State::Downloading { version_cache, .. } => {
                log::warn!("Switching beta after while updating resulted in new suggested upgrade: {:?}, aborting", new_app_version_info.suggested_upgrade);
                version_cache
            }
            State::HasVersion { version_cache } => version_cache,
            State::NoVersion => {
                unreachable!("Can't get recommended upgrade on beta change without version")
            }
        };

        self.state = State::HasVersion { version_cache };
        let _ = self.version_event_sender.send(new_app_version_info.clone());

        self.notify_version_requesters(new_app_version_info);
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
        use crate::version::downloader::spawn_downloader;

        match mem::replace(&mut self.state, State::NoVersion) {
            // If we're already downloading or have a version, do nothing
            State::Downloaded { version_cache, .. } | State::HasVersion { version_cache } => {
                let Some(upgrading_to_version) =
                    recommended_version_upgrade(&version_cache.latest_version, self.beta_program)
                else {
                    // If there's no suggested upgrade, do nothing
                    log::debug!("Received update request without suggested upgrade");
                    self.state = State::HasVersion { version_cache };
                    return;
                };
                log::info!(
                    "Starting upgrade to version {}",
                    upgrading_to_version.version
                );

                let downloader_handle = spawn_downloader(
                    upgrading_to_version.clone(),
                    self.app_upgrade_broadcast.clone(),
                );

                self.state = State::Downloading {
                    version_cache,
                    upgrading_to_version,
                    downloader_handle,
                };
            }
            // Already downloading/downloaded or there is no version: do nothing
            state => {
                log::debug!("Ignoring update request while in state {:?}", state);
                self.state = state;
            }
        }
    }

    #[cfg(update)]
    fn cancel_upgrade(&mut self) {
        match mem::replace(&mut self.state, State::NoVersion) {
            // If we're upgrading, emit an event if a version was received during the upgrade
            // Otherwise, just reset upgrade info to last known state
            State::Downloaded { version_cache, .. } | State::Downloading { version_cache, .. } => {
                self.state = State::HasVersion { version_cache };
            }
            // No-op unless we're downloading something right now
            // In the `Downloaded` state, we also do nothing
            state => self.state = state,
        };
        debug_assert!(!matches!(
            self.state,
            State::Downloading { .. } | State::NoVersion
        ));
    }
}

fn updated_app_version_info_on_new_version_cache(
    version_cache: &VersionCache,
    new_version_cache: &VersionCache,
    beta_program: bool,
) -> Option<AppVersionInfo> {
    let prev_app_version = to_app_version_info(version_cache, beta_program, None);
    let new_app_version = to_app_version_info(new_version_cache, beta_program, None);

    // Update version info
    if new_app_version != prev_app_version {
        Some(new_app_version)
    } else {
        None
    }
}

fn updated_app_version_info_on_new_beta(
    version_cache: &VersionCache,
    previous_beta_state: bool,
    new_beta_state: bool,
) -> Option<AppVersionInfo> {
    let prev_app_version = to_app_version_info(version_cache, previous_beta_state, None);
    let new_app_version = to_app_version_info(version_cache, new_beta_state, None);

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
async fn wait_for_update(state: &mut State) -> Option<AppVersionInfo> {
    #[cfg(update)]
    match state {
        State::Downloading {
            version_cache,
            ref mut downloader_handle,
            upgrading_to_version,
            ..
        } => match downloader_handle.wait().await {
            Ok(verified_installer_path) => {
                let app_update_info = AppVersionInfo {
                    current_version_supported: version_cache.current_version_supported,
                    suggested_upgrade: Some({
                        SuggestedUpgrade {
                            version: upgrading_to_version.version.clone(),
                            changelog: upgrading_to_version.changelog.clone(),
                            verified_installer_path: Some(verified_installer_path.clone()),
                        }
                    }),
                };
                *state = State::Downloaded {
                    version_cache: version_cache.clone(),
                    verified_installer_path,
                };
                Some(app_update_info)
            }
            Err(err) => {
                log::trace!("Downloader task ended: {err}");
                *state = State::HasVersion {
                    version_cache: version_cache.clone(),
                };
                None
            }
        },
        _ => {
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
    let suggested_upgrade =
        recommended_version_upgrade(&cache.latest_version, beta_program).map(|version| {
            SuggestedUpgrade {
                version: version.version,
                changelog: version.changelog,
                verified_installer_path,
            }
        });
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

#[cfg(test)]
mod test {
    use futures::channel::mpsc::unbounded;

    use super::*;

    struct VersionRouterChannels {
        version_event_sender: futures::channel::mpsc::UnboundedSender<AppVersionInfo>,
        daemon_tx: futures::channel::mpsc::UnboundedSender<Message>,
    }

    fn make_version_router() -> (
        VersionRouter<futures::channel::mpsc::UnboundedSender<AppVersionInfo>>,
        VersionRouterChannels,
    ) {
        let (version_event_sender, version_event_receiver) = unbounded();
        let (daemon_tx, daemon_rx) = unbounded();
        let (app_upgrade_broadcast, _) = tokio::sync::broadcast::channel(1);
        (
            VersionRouter {
                daemon_rx,
                state: State::NoVersion,
                beta_program: false,
                version_event_sender,
                version_check: todo!(),
                new_version_rx: todo!(),
                version_request: Fuse::terminated(),
                version_request_channels: vec![],
                app_upgrade_broadcast,
            },
            VersionRouterChannels {
                version_event_sender,
                daemon_tx,
            },
        )
    }

    #[test]
    fn test_upgrade_with_no_version() {
        let (mut version_router, channels) = make_version_router();
        let upgrade_events = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        assert!(matches!(version_router.state, State::NoVersion));
        assert!(version_router.version_request.is_terminated());
        assert!(version_router.version_request_channels.is_empty());
    }
}
