use std::ops::ControlFlow;
use std::path::PathBuf;

use futures::channel::{mpsc, oneshot};
use futures::stream::StreamExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::{AppVersionInfo, SuggestedUpgrade};
#[cfg(in_app_upgrade)]
use mullvad_update::app::{AppDownloader, AppDownloaderParameters, HttpAppDownloader};
use mullvad_update::version::{Rollout, VersionInfo};
use talpid_core::mpsc::Sender;
#[cfg(in_app_upgrade)]
use talpid_types::ErrorExt;

use crate::DaemonEventSender;
use crate::management_interface::AppUpgradeBroadcast;

#[cfg(in_app_upgrade)]
use super::downloader::ProgressUpdater;
use super::{
    Error,
    check::{VersionCache, VersionUpdater},
};

#[cfg(in_app_upgrade)]
use super::downloader;
use std::mem;

pub type Result<T> = std::result::Result<T, Error>;

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

    #[cfg(in_app_upgrade)]
    pub async fn update_application(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::UpdateApplication { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    #[cfg(in_app_upgrade)]
    pub async fn cancel_update(&self) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::CancelUpdate { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }

    #[cfg(in_app_upgrade)]
    pub async fn get_cache_dir(&self) -> Result<PathBuf> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::GetCacheDir { result_tx })
            .map_err(|_| Error::VersionRouterClosed)?;
        result_rx.await.map_err(|_| Error::VersionRouterClosed)
    }
}

// These wrapper traits and type aliases exist to help feature gate the module
#[cfg(in_app_upgrade)]
trait Downloader:
    AppDownloader + Send + 'static + From<AppDownloaderParameters<ProgressUpdater>>
{
}
#[cfg(not(in_app_upgrade))]
trait Downloader {}

#[cfg(in_app_upgrade)]
type DefaultDownloader = HttpAppDownloader<ProgressUpdater>;
#[cfg(not(in_app_upgrade))]
type DefaultDownloader = ();

impl Downloader for DefaultDownloader {}

/// Router of version updates and update requests.
///
/// New available app version events are forwarded from the [`VersionUpdater`].
/// If an update is in progress, these events are paused until the update is completed or canceled.
/// This is done to prevent frontends from confusing which version is currently being installed,
/// in case new version info is received while the update is in progress.
struct VersionRouter<S = DaemonEventSender<AppVersionInfo>, D = DefaultDownloader> {
    daemon_rx: mpsc::UnboundedReceiver<Message>,
    state: State,
    beta_program: bool,
    version_event_sender: S,
    /// Channel used to trigger a version check. The result will always be sent to the
    /// `new_version_rx` channel.
    refresh_version_check_tx: mpsc::UnboundedSender<()>,
    /// Channel used to receive updates from `version_check`
    new_version_rx: mpsc::UnboundedReceiver<VersionCache>,
    /// Channels that receive responses to `get_latest_version`
    version_request_channels: Vec<oneshot::Sender<Result<AppVersionInfo>>>,
    /// Broadcast channel for app upgrade events
    #[cfg(in_app_upgrade)]
    app_upgrade_broadcast: AppUpgradeBroadcast,
    #[cfg(in_app_upgrade)]
    cache_dir: PathBuf,
    /// Type used to spawn the downloader task, replaced when testing
    _phantom: std::marker::PhantomData<D>,
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
    #[cfg(in_app_upgrade)]
    UpdateApplication { result_tx: oneshot::Sender<()> },
    /// Cancel the ongoing update
    #[cfg(in_app_upgrade)]
    CancelUpdate { result_tx: oneshot::Sender<()> },
    /// Get the cache dir
    #[cfg(in_app_upgrade)]
    GetCacheDir { result_tx: oneshot::Sender<PathBuf> },
}

#[derive(Debug)]
enum State {
    /// There is no version available yet
    NoVersion,
    /// Running version checker, no upgrade in progress
    HasVersion { version_cache: VersionCache },
    /// Download is in progress, so we don't forward version checks
    #[cfg(in_app_upgrade)]
    Downloading {
        /// Version info received from `HasVersion`
        version_cache: VersionCache,
        /// The version being upgraded to, derived from `version_info` and beta program state
        upgrading_to_version: mullvad_update::version::Version,
        /// Tokio task for the downloader handle
        downloader_handle: downloader::DownloaderHandle,
    },
    /// Download is complete. We have a verified binary
    #[cfg(in_app_upgrade)]
    Downloaded {
        /// Version info received from `HasVersion`
        version_cache: VersionCache,
        /// Path to verified installer
        verified_installer_path: PathBuf,
    },
}

struct AppVersionInfoEvent {
    app_version_info: AppVersionInfo,
    is_new: bool,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::NoVersion => write!(f, "NoVersion"),
            State::HasVersion { .. } => write!(f, "HasVersion"),
            #[cfg(in_app_upgrade)]
            State::Downloading {
                upgrading_to_version,
                ..
            } => write!(f, "Downloading '{}'", upgrading_to_version.version),
            #[cfg(in_app_upgrade)]
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
            #[cfg(in_app_upgrade)]
            State::Downloading { version_cache, .. } | State::Downloaded { version_cache, .. } => {
                Some(version_cache)
            }
        }
    }
}

#[cfg_attr(not(in_app_upgrade), allow(unused_variables))]
pub(crate) fn spawn_version_router(
    api_handle: MullvadRestHandle,
    availability_handle: ApiAvailability,
    cache_dir: PathBuf,
    version_event_sender: DaemonEventSender<AppVersionInfo>,
    beta_program: bool,
    rollout: Rollout,
    app_upgrade_broadcast: AppUpgradeBroadcast,
) -> VersionRouterHandle {
    let (tx, rx) = mpsc::unbounded();

    tokio::spawn(async move {
        let (new_version_tx, new_version_rx) = mpsc::unbounded();
        let (refresh_version_check_tx, refresh_version_check_rx) = mpsc::unbounded();

        #[cfg(in_app_upgrade)]
        let _ = downloader::clear_download_dir().await.inspect_err(|err| {
            log::error!(
                "{}",
                err.display_chain_with_msg("Failed to clean up download directory")
            )
        });

        VersionUpdater::spawn(
            api_handle,
            availability_handle,
            cache_dir.clone(),
            new_version_tx,
            refresh_version_check_rx,
            rollout,
        )
        .await;

        VersionRouter {
            daemon_rx: rx,
            state: State::NoVersion,
            beta_program,
            version_event_sender,
            new_version_rx,
            version_request_channels: vec![],
            #[cfg(in_app_upgrade)]
            app_upgrade_broadcast,
            #[cfg(in_app_upgrade)]
            cache_dir,
            refresh_version_check_tx,
            _phantom: std::marker::PhantomData::<DefaultDownloader>,
        }
        .run()
        .await;
    });
    VersionRouterHandle { tx }
}

impl<S, D> VersionRouter<S, D>
where
    S: Sender<AppVersionInfo> + Send + 'static,
    D: Downloader,
{
    async fn run(mut self) {
        log::debug!("Version router started");
        // Loop until the router is closed
        while self.run_step().await.is_continue() {}
        log::debug!("Version router closed");
    }

    /// Run a single step of the router, handling messages from the daemon and version events
    async fn run_step(&mut self) -> ControlFlow<()> {
        tokio::select! {
            // Received version event from `check`
            Some(new_version) = self.new_version_rx.next() => {
                let AppVersionInfoEvent { app_version_info, is_new } = self.on_new_version(new_version);
                self.notify_version_requesters(app_version_info.clone());
                if is_new {
                    // Notify the daemon about new version
                    let _ = self.version_event_sender.send(app_version_info);
                }
            }
            res = wait_for_update(&mut self.state) => {
                // If the download was successful, we send the new version, which contains the
                // verified installer path
                if let Some(app_update_info) = res {
                    let _ = self.version_event_sender.send(app_update_info);
                }
            },
            Some(message) = self.daemon_rx.next() => self.handle_message(message),
            else => return ControlFlow::Break(()),
        }
        ControlFlow::Continue(())
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
            #[cfg(in_app_upgrade)]
            Message::UpdateApplication { result_tx } => {
                self.update_application();
                let _ = result_tx.send(());
            }
            #[cfg(in_app_upgrade)]
            Message::CancelUpdate { result_tx } => {
                self.cancel_upgrade();
                let _ = result_tx.send(());
            }
            #[cfg(in_app_upgrade)]
            Message::GetCacheDir { result_tx } => {
                let _ = result_tx.send(self.cache_dir.clone());
            }
        }
    }

    /// Handle new version info
    ///
    /// If the router is in the process of upgrading, it will not propagate versions, but only
    /// remember it for when it transitions back into the "idle" (version check) state.
    fn on_new_version(&mut self, version_cache: VersionCache) -> AppVersionInfoEvent {
        let new_app_version_info = match &mut self.state {
            State::NoVersion => {
                // Receive first version
                let app_version_info = to_app_version_info(&version_cache, self.beta_program, None);

                AppVersionInfoEvent {
                    app_version_info,
                    is_new: true,
                }
            }
            // Already have version info, just update it
            State::HasVersion {
                version_cache: prev_cache,
            } => {
                let prev_app_version = to_app_version_info(prev_cache, self.beta_program, None);
                let new_app_version = to_app_version_info(&version_cache, self.beta_program, None);

                AppVersionInfoEvent {
                    is_new: new_app_version != prev_app_version,
                    app_version_info: new_app_version,
                }
            }
            #[cfg(in_app_upgrade)]
            State::Downloading {
                version_cache: prev_cache,
                ..
            } => {
                let prev_app_version_info =
                    to_app_version_info(prev_cache, self.beta_program, None);
                let app_version_info = to_app_version_info(&version_cache, self.beta_program, None);

                let event = AppVersionInfoEvent {
                    is_new: prev_app_version_info != app_version_info,
                    app_version_info,
                };

                if !event.is_new {
                    log::trace!("Ignoring same version in downloading state");
                    // Return here to avoid resetting the state to `HasVersion`
                    // We update the cache because ignored information (eg available beta if beta
                    // program is off) may have changed
                    *prev_cache = version_cache.clone();
                    return event;
                }

                log::warn!("Received new version while downloading. Aborting download");

                event
            }
            #[cfg(in_app_upgrade)]
            State::Downloaded {
                version_cache: prev_cache,
                verified_installer_path,
                ..
            } => {
                let prev_app_version_info = to_app_version_info(
                    prev_cache,
                    self.beta_program,
                    Some(verified_installer_path.clone()),
                );
                let app_version_info = to_app_version_info(
                    &version_cache,
                    self.beta_program,
                    Some(verified_installer_path.clone()),
                );

                let event = AppVersionInfoEvent {
                    is_new: prev_app_version_info != app_version_info,
                    app_version_info,
                };

                if !event.is_new {
                    log::trace!("Ignoring same version in downloaded state");
                    // Return here to avoid resetting the state to `HasVersion`
                    // We update the cache because ignored information (eg available beta if beta
                    // program is off) may have changed
                    *prev_cache = version_cache.clone();
                    return event;
                }

                log::warn!("Received new version in downloaded state. Aborting download");

                event
            }
        };
        self.state = State::HasVersion { version_cache };
        new_app_version_info
    }

    fn notify_version_requesters(&mut self, new_app_version_info: AppVersionInfo) {
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
        let Some(version_cache) = self.state.get_version_cache() else {
            return;
        };
        let prev_app_version = to_app_version_info(version_cache, previous_state, None);
        let new_app_version = to_app_version_info(version_cache, new_state, None);
        if new_app_version == prev_app_version {
            return;
        };

        // Always cancel download if the suggested upgrade changes
        let version_cache = match mem::replace(&mut self.state, State::NoVersion) {
            #[cfg(in_app_upgrade)]
            State::Downloaded { version_cache, .. } | State::Downloading { version_cache, .. } => {
                log::warn!(
                    "Switching beta after updating resulted in new suggested upgrade: {:?}, aborting",
                    new_app_version.suggested_upgrade
                );
                version_cache
            }
            State::HasVersion { version_cache } => version_cache,
            State::NoVersion => {
                unreachable!("Can't get recommended upgrade on beta change without version")
            }
        };

        self.state = State::HasVersion { version_cache };
        let _ = self.version_event_sender.send(new_app_version.clone());

        self.notify_version_requesters(new_app_version);
    }

    fn get_latest_version(
        &mut self,
        result_tx: oneshot::Sender<std::result::Result<AppVersionInfo, Error>>,
    ) {
        // Start a version request unless already in progress
        match self
            .refresh_version_check_tx
            .unbounded_send(())
            .map_err(|_e| Error::VersionRouterClosed)
        {
            // Append to response channels
            Ok(()) => self.version_request_channels.push(result_tx),
            Err(err) => result_tx
                .send(Err(err))
                .unwrap_or_else(|e| log::warn!("Failed to send version request result: {e:?}")),
        }
    }

    #[cfg(in_app_upgrade)]
    fn update_application(&mut self) {
        use crate::version::downloader::spawn_downloader;

        match mem::replace(&mut self.state, State::NoVersion) {
            State::HasVersion { version_cache } => {
                let Some(upgrading_to_version) =
                    recommended_version_upgrade(&version_cache.version_info, self.beta_program)
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

                let downloader_handle = spawn_downloader::<D>(
                    upgrading_to_version.clone(),
                    self.app_upgrade_broadcast.clone(),
                );

                self.state = State::Downloading {
                    version_cache,
                    upgrading_to_version,
                    downloader_handle,
                };
            }
            state => {
                log::debug!("Ignoring update request while in state {:?}", state);
                self.state = state;
            }
        }
    }

    #[cfg(in_app_upgrade)]
    fn cancel_upgrade(&mut self) {
        use mullvad_types::version::AppUpgradeEvent;

        match mem::replace(&mut self.state, State::NoVersion) {
            // If we're upgrading, emit an event if a version was received during the upgrade
            // Otherwise, just reset upgrade info to last known state
            State::Downloading { version_cache, .. } => {
                self.state = State::HasVersion { version_cache };
            }
            State::Downloaded { version_cache, .. } => {
                let app_version_info = to_app_version_info(&version_cache, self.beta_program, None);
                self.state = State::HasVersion { version_cache };

                // Send "Aborted" here, since there's no "Downloader" to do it for us
                let _ = self.app_upgrade_broadcast.send(AppUpgradeEvent::Aborted);

                // Notify the daemon and version requesters about new version
                self.notify_version_requesters(app_version_info.clone());
                let _ = self.version_event_sender.send(app_version_info);
            }
            // No-op unless we're downloading something right now
            // In the `Downloaded` state, we also do nothing
            state => self.state = state,
        };

        debug_assert!(matches!(
            self.state,
            State::HasVersion { .. } | State::NoVersion
        ));
    }
}

/// Wait for the update to finish. In case no update is in progress (or the platform does not
/// support in-app upgrades), then the future will never resolve as to not escape the select statement.
#[allow(clippy::unused_async, unused_variables)]
async fn wait_for_update(state: &mut State) -> Option<AppVersionInfo> {
    #[cfg(in_app_upgrade)]
    match state {
        State::Downloading {
            version_cache,
            downloader_handle,
            upgrading_to_version,
            ..
        } => match downloader_handle.await {
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
                log::warn!("{}", err.display_chain_with_msg("Downloader task ended"));
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
    #[cfg(not(in_app_upgrade))]
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
        recommended_version_upgrade(&cache.version_info, beta_program).map(|version| {
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

#[cfg(all(test, in_app_upgrade))]
mod test {
    use std::time::SystemTime;

    use super::downloader::ProgressUpdater;
    use futures::channel::mpsc::unbounded;
    use mullvad_types::version::{AppUpgradeDownloadProgress, AppUpgradeEvent};
    use mullvad_update::{
        app::{DownloadError, DownloadedInstaller, VerifiedInstaller},
        fetch::ProgressUpdater as _,
    };
    use tokio::sync::broadcast::error::TryRecvError;

    use super::*;

    /// To be able to test events occurring during the download process, we need to
    /// call `tokio::time::sleep` in the downloader. This will not affect the runtime
    /// of the tests, as set `start_paused = true`.
    const DOWNLOAD_DURATION: std::time::Duration = std::time::Duration::from_millis(1000);

    /// Mock downloader that simulates a successful download
    struct SuccessfulAppDownloader(AppDownloaderParameters<ProgressUpdater>);

    impl AppDownloader for SuccessfulAppDownloader {
        async fn download_executable(
            mut self,
        ) -> std::result::Result<impl DownloadedInstaller, DownloadError> {
            tokio::time::sleep(DOWNLOAD_DURATION).await;
            self.0.app_progress.set_progress(1.0);
            Ok(self)
        }
    }

    impl DownloadedInstaller for SuccessfulAppDownloader {
        fn version(&self) -> &mullvad_version::Version {
            &self.0.app_version
        }

        async fn verify(self) -> std::result::Result<impl VerifiedInstaller, DownloadError> {
            Ok(self)
        }
    }

    impl VerifiedInstaller for SuccessfulAppDownloader {
        async fn install(self) -> std::result::Result<(), DownloadError> {
            Ok(())
        }
    }

    impl From<AppDownloaderParameters<ProgressUpdater>> for SuccessfulAppDownloader {
        fn from(parameters: AppDownloaderParameters<ProgressUpdater>) -> Self {
            Self(parameters)
        }
    }

    impl Downloader for SuccessfulAppDownloader {}

    /// Mock downloader that simulates a failed download
    struct FailingAppDownloader;

    impl AppDownloader for FailingAppDownloader {
        async fn download_executable(
            self,
        ) -> std::result::Result<impl DownloadedInstaller, DownloadError> {
            Err::<Self, _>(DownloadError::FetchApp(anyhow::anyhow!("Download failed")))
        }
    }

    impl DownloadedInstaller for FailingAppDownloader {
        fn version(&self) -> &mullvad_version::Version {
            unreachable!()
        }

        async fn verify(self) -> std::result::Result<impl VerifiedInstaller, DownloadError> {
            Ok(self)
        }
    }

    impl VerifiedInstaller for FailingAppDownloader {
        async fn install(self) -> std::result::Result<(), DownloadError> {
            unreachable!()
        }
    }

    impl From<AppDownloaderParameters<ProgressUpdater>> for FailingAppDownloader {
        fn from(_parameters: AppDownloaderParameters<ProgressUpdater>) -> Self {
            Self
        }
    }

    impl Downloader for FailingAppDownloader {}

    /// Mock downloader that simulates a failed verification, but a successful download
    struct FailingAppVerifier;

    impl AppDownloader for FailingAppVerifier {
        async fn download_executable(
            self,
        ) -> std::result::Result<impl DownloadedInstaller, DownloadError> {
            Ok(self)
        }
    }

    impl DownloadedInstaller for FailingAppVerifier {
        fn version(&self) -> &mullvad_version::Version {
            &mullvad_version::Version {
                year: 2042,
                incremental: 1337,
                pre_stable: None,
                dev: None,
            }
        }

        async fn verify(self) -> std::result::Result<impl VerifiedInstaller, DownloadError> {
            Err::<Self, _>(DownloadError::Verification(anyhow::anyhow!(
                "Verification failed"
            )))
        }
    }

    impl VerifiedInstaller for FailingAppVerifier {
        async fn install(self) -> std::result::Result<(), DownloadError> {
            unreachable!()
        }
    }

    impl From<AppDownloaderParameters<ProgressUpdater>> for FailingAppVerifier {
        fn from(_parameters: AppDownloaderParameters<ProgressUpdater>) -> Self {
            Self
        }
    }

    impl Downloader for FailingAppVerifier {}

    /// Channels used to communicate with the version router and receive version events.
    /// This is used in the tests to simulate the daemon and `VersionUpdater`.
    struct VersionRouterChannels {
        daemon_tx: futures::channel::mpsc::UnboundedSender<Message>,
        new_version_tx: futures::channel::mpsc::UnboundedSender<VersionCache>,
        refresh_version_check_rx: futures::channel::mpsc::UnboundedReceiver<()>,
        version_event_receiver: futures::channel::mpsc::UnboundedReceiver<AppVersionInfo>,
    }

    fn make_version_router<D>() -> (
        VersionRouter<futures::channel::mpsc::UnboundedSender<AppVersionInfo>, D>,
        VersionRouterChannels,
    ) {
        let (version_event_sender, version_event_receiver) = unbounded();
        let (daemon_tx, daemon_rx) = unbounded();
        let (app_upgrade_broadcast, _) = tokio::sync::broadcast::channel(10);
        let (refresh_version_check_tx, refresh_version_check_rx) = unbounded();
        let (new_version_tx, new_version_rx) = unbounded();
        (
            VersionRouter {
                daemon_rx,
                state: State::NoVersion,
                beta_program: false,
                version_event_sender,
                new_version_rx,
                version_request_channels: vec![],
                app_upgrade_broadcast,
                refresh_version_check_tx,
                cache_dir: PathBuf::new(),
                _phantom: std::marker::PhantomData::<D>,
            },
            VersionRouterChannels {
                daemon_tx,
                new_version_tx,
                refresh_version_check_rx,
                version_event_receiver,
            },
        )
    }

    /// Create a version cache with a stable version that is newer than the current version
    fn get_new_stable_version_cache() -> VersionCache {
        let mut version: mullvad_version::Version = mullvad_version::VERSION.parse().unwrap();
        version.incremental += 1;
        VersionCache {
            cache_version: version.clone(),
            current_version_supported: true,
            version_info: VersionInfo {
                beta: None,
                stable: mullvad_update::version::Version {
                    version,
                    urls: vec!["https://example.com".to_string()],
                    size: 123456,
                    changelog: "Changelog".to_string(),
                    sha256: [0; 32],
                },
            },
            last_platform_header_check: SystemTime::now(),
            metadata_version: 0,
            etag: None,
        }
    }

    /// Create a version cache with a beta version that is newer than the current version
    fn get_new_beta_version_cache() -> VersionCache {
        let stable = mullvad_update::version::Version {
            version: mullvad_version::VERSION.parse().unwrap(),
            urls: vec!["https://example.com".to_string()],
            size: 123456,
            changelog: "Changelog".to_string(),
            sha256: [0; 32],
        };
        let mut beta = stable.clone();
        beta.version.pre_stable = Some(mullvad_version::PreStableType::Beta(1));
        beta.version.incremental += 1;
        VersionCache {
            cache_version: stable.version.clone(),
            current_version_supported: true,
            version_info: VersionInfo {
                beta: Some(beta),
                stable,
            },
            last_platform_header_check: SystemTime::now(),
            metadata_version: 0,
            etag: None,
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_upgrade_with_no_version() {
        let (mut version_router, _channels) = make_version_router::<SuccessfulAppDownloader>();
        let upgrade_events = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        assert!(
            matches!(version_router.state, State::NoVersion),
            "State should stay as NoVersion after calling update_application"
        );
        assert!(
            upgrade_events.is_empty(),
            "No upgrade events should be sent"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_new_beta() {
        let (mut version_router, mut channels) = make_version_router::<SuccessfulAppDownloader>();
        let version_cache = get_new_beta_version_cache();

        // Test that new beta version is ignored if beta program is off
        version_router.set_beta_program(false); // This is default value, but set it for clarity
        assert!(
            matches!(version_router.state, State::NoVersion),
            "State should not transition"
        );
        version_router.on_new_version(version_cache);
        assert!(matches!(version_router.state, State::HasVersion { .. }));
        assert!(
            channels.version_event_receiver.try_next().is_err(),
            "No version event should be sent on beta program change"
        );
        version_router.update_application();
        assert!(
            matches!(version_router.state, State::HasVersion { .. }),
            "State should not transition to Downloading as the beta version is ignored"
        );

        // Test that switching to beta program sends version event for the previously received beta
        // version and allows upgrades.
        version_router.set_beta_program(true);
        assert!(
            channels.version_event_receiver.try_next().is_ok(),
            "Version event should be sent on beta program change"
        );
        version_router.update_application();
        assert!(
            matches!(version_router.state, State::Downloading { .. }),
            "State should transition to Downloading as the beta version is accepted"
        );
    }

    /// Test that when the daemon calls `get_latest_version`, it will trigger a version check
    /// and send the result back to the daemon, both on the response channel and in the
    /// version event stream.
    #[tokio::test(start_paused = true)]
    async fn test_get_latest_version() {
        let (mut version_router, mut channels) = make_version_router::<SuccessfulAppDownloader>();
        let version_cache_test = get_new_stable_version_cache();

        // Make a request to the router to get the latest version
        // Note that we could as well call `version_router.get_latest_version()`,
        // but this way we test the actual message passing between the router and
        // the daemon.
        let (tx, mut get_latest_version_rx) = oneshot::channel();
        channels
            .daemon_tx
            .unbounded_send(Message::GetLatestVersion(tx))
            .unwrap();
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));

        // Here, we play the role of `VersionUpdater`.
        // It should receive a version check request and send a version in response
        assert!(
            matches!(channels.refresh_version_check_rx.try_next(), Ok(Some(()))),
            "Version check should be triggered"
        );
        channels
            .new_version_tx
            .unbounded_send(version_cache_test.clone())
            .unwrap();

        // On the next step, the router should receive the version info
        // and send it to as a response to the oneshot from `GetLatestVersion`
        // and to the daemon in the `version_event_receiver` channel.
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        let version_info = get_latest_version_rx
            .try_recv()
            .expect("Sender should not be dropped")
            .expect("Version info should have been sent")
            .expect("Version request should be successful");
        match &version_router.state {
            State::HasVersion { version_cache } => assert_eq!(version_cache, &version_cache_test),
            other => panic!("State should be HasVersion, was {other:?}"),
        }
        assert_eq!(
            version_info,
            channels
                .version_event_receiver
                .try_next()
                .expect("Version event sender should not be closed")
                .expect("Version event should be sent"),
            "Version event sent to the daemon should be the same as the one sent to the requester"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_upgrade() {
        let (mut version_router, mut channels) = make_version_router::<SuccessfulAppDownloader>();
        let version_cache_test = get_new_stable_version_cache();

        version_router.on_new_version(version_cache_test.clone());
        match &version_router.state {
            State::HasVersion { version_cache } => assert_eq!(version_cache, &version_cache_test),
            other => panic!("State should be HasVersion, was {other:?}"),
        }

        // Start upgrading
        let mut app_upgrade_listener = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        // Check that the state is now downloading
        match &version_router.state {
            State::Downloading {
                version_cache,
                upgrading_to_version,
                ..
            } => {
                assert_eq!(version_cache, &version_cache_test);
                assert_eq!(
                    upgrading_to_version.version,
                    version_cache_test.version_info.stable.version
                );
            }
            other => panic!("State should be Downloading, was {other:?}"),
        }

        version_router.update_application();
        assert!(
            matches!(version_router.state, State::Downloading { .. }),
            "Triggering an update while in the downloading shout be ignored"
        );

        // Drive the download to completion, and get the verified installer path
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        let verified_installer_path = match &version_router.state {
            State::Downloaded {
                version_cache,
                verified_installer_path,
                ..
            } => {
                assert_eq!(version_cache, &version_cache_test);
                verified_installer_path
            }
            other => panic!("State should be Downloaded, was {other:?}"),
        };

        // Check that the app upgrade events were sent
        let events = [
            Ok(AppUpgradeEvent::DownloadStarting),
            Ok(AppUpgradeEvent::DownloadProgress(
                AppUpgradeDownloadProgress {
                    progress: 100,
                    server: "example.com".to_string(),
                    time_left: None,
                },
            )),
            Ok(AppUpgradeEvent::VerifyingInstaller),
            Ok(AppUpgradeEvent::VerifiedInstaller),
            Err(TryRecvError::Empty), // No more events should be sent
        ];
        for event in events {
            assert_eq!(app_upgrade_listener.try_recv(), event);
        }

        // Check that the version event was sent with the verified installer path
        let version_info = channels
            .version_event_receiver
            .try_next()
            .expect("Version event channel should contain message")
            .expect("Version event should be sent");
        assert_eq!(
            version_info
                .suggested_upgrade
                .as_ref()
                .unwrap()
                .verified_installer_path,
            Some(verified_installer_path.clone())
        );
        channels
            .version_event_receiver
            .try_next()
            .expect_err("Channel should not have any messages");

        version_router.update_application();
        assert!(
            matches!(version_router.state, State::Downloaded { .. }),
            "Triggering an update while in the downloaded shout be ignored"
        );

        version_router.cancel_upgrade();
        assert!(
            matches!(version_router.state, State::HasVersion { .. }),
            "State should be HasVersion after cancelling the upgrade"
        );

        assert_eq!(
            app_upgrade_listener.try_recv(),
            Ok(AppUpgradeEvent::Aborted),
            "The `AppUpgradeEvent::Aborted` should be sent when cancelling a finished download"
        );
        assert_eq!(
            app_upgrade_listener.try_recv(),
            Err(TryRecvError::Empty),
            "No more events should be sent",
        );

        let version_info = channels
            .version_event_receiver
            .try_next()
            .expect("Version event channel should contain message")
            .expect("Version event should be sent");
        assert_eq!(
            version_info
                .suggested_upgrade
                .as_ref()
                .unwrap()
                .verified_installer_path,
            None,
            "Aborting should send a new `AppVersionInfo` without a verified installer path"
        );
    }

    /// Test that the update is aborted if a new version is received while downloading
    #[tokio::test(start_paused = true)]
    async fn test_abort_on_new_version() {
        let (mut version_router, _channels) = make_version_router::<SuccessfulAppDownloader>();
        let upgrade_version = get_new_stable_version_cache();
        let mut upgrade_version_newer = upgrade_version.clone();
        upgrade_version_newer
            .version_info
            .stable
            .version
            .incremental += 1;

        version_router.on_new_version(upgrade_version.clone());

        // Start upgrading
        let mut app_upgrade_listener = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        // Check that the state is now downloading
        assert!(matches!(version_router.state, State::Downloading { .. }),);

        // Advance the download to the point where we have started downloading
        tokio::time::sleep(DOWNLOAD_DURATION / 2).await;
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::DownloadStarting
        );
        assert_eq!(app_upgrade_listener.try_recv(), Err(TryRecvError::Empty));

        // Now, send a new version while the download is in progress
        version_router.on_new_version(upgrade_version_newer);
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::Aborted
        );
        assert_eq!(app_upgrade_listener.try_recv(), Err(TryRecvError::Empty));
    }

    #[tokio::test]
    async fn test_failed_download() {
        let (mut version_router, _channels) = make_version_router::<FailingAppDownloader>();
        let version_cache_test = get_new_stable_version_cache();

        version_router.on_new_version(version_cache_test.clone());

        // Start upgrading
        let mut app_upgrade_listener = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        // Check that the state is now downloading
        assert!(matches!(version_router.state, State::Downloading { .. }),);

        // Drive the download to completion
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::DownloadStarting
        );
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::Error(mullvad_types::version::AppUpgradeError::DownloadFailed)
        );
        assert_eq!(app_upgrade_listener.try_recv(), Err(TryRecvError::Empty));
        version_router.update_application();

        // Verify that we can restart the download again
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::DownloadStarting
        );
    }

    #[tokio::test]
    async fn test_update_in_downloaded_state() {
        let (mut version_router, _channels) = make_version_router::<SuccessfulAppDownloader>();
        let mut version_cache_test = get_new_stable_version_cache();

        version_router.on_new_version(version_cache_test.clone());

        // Start upgrading
        version_router.update_application();
        // Check that the state is now downloading
        assert!(matches!(version_router.state, State::Downloading { .. }),);

        // Should remain in downloading state if same version is received
        version_router.on_new_version(version_cache_test.clone());
        assert!(
            matches!(version_router.state, State::Downloading { .. }),
            "state should be Downloading, was {:?}",
            version_router.state,
        );

        // Unless the version is different
        version_cache_test.version_info.stable.version.incremental += 1;
        version_router.on_new_version(version_cache_test.clone());
        assert!(
            matches!(version_router.state, State::HasVersion { .. }),
            "state should be HasVersion, was {:?}",
            version_router.state,
        );

        // Restart upgrade
        version_router.update_application();

        // Drive the download to completion
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        assert!(
            matches!(version_router.state, State::Downloaded { .. }),
            "state should be Downloaded, was {:?}",
            version_router.state,
        );

        // Should remain in downloaded state if same version is received
        version_router.on_new_version(version_cache_test.clone());
        assert!(
            matches!(version_router.state, State::Downloaded { .. }),
            "state should be Downloaded, was {:?}",
            version_router.state,
        );

        // Unless the version is different
        version_cache_test.version_info.stable.version.incremental += 1;
        version_router.on_new_version(version_cache_test.clone());
        assert!(
            matches!(version_router.state, State::HasVersion { .. }),
            "state should be HasVersion, was {:?}",
            version_router.state,
        );
    }

    #[tokio::test]
    async fn test_failed_verification() {
        let (mut version_router, _channels) = make_version_router::<FailingAppVerifier>();
        let version_cache_test = get_new_stable_version_cache();

        version_router.on_new_version(version_cache_test.clone());

        // Start upgrading
        let mut app_upgrade_listener = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        // Check that the state is now downloading
        assert!(matches!(version_router.state, State::Downloading { .. }),);

        // Drive the download to completion
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::DownloadStarting
        );
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::VerifyingInstaller
        );
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::Error(mullvad_types::version::AppUpgradeError::VerificationFailed)
        );
        assert_eq!(app_upgrade_listener.try_recv(), Err(TryRecvError::Empty));
        version_router.update_application();

        // Verify that we can restart the download again
        assert_eq!(version_router.run_step().await, ControlFlow::Continue(()));
        assert_eq!(
            app_upgrade_listener.try_recv().unwrap(),
            AppUpgradeEvent::DownloadStarting
        );
    }
}
