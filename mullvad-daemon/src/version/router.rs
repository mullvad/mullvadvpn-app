use std::path::PathBuf;

use futures::channel::{mpsc, oneshot};
use futures::stream::StreamExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::{AppVersionInfo, SuggestedUpgrade};
use mullvad_update::app::{AppDownloader, AppDownloaderParameters, HttpAppDownloader};
use mullvad_update::version::VersionInfo;
use talpid_core::mpsc::Sender;

use crate::management_interface::AppUpgradeBroadcast;
use crate::DaemonEventSender;

use super::downloader::ProgressUpdater;
use super::{
    check::{VersionCache, VersionUpdater},
    Error,
};

#[cfg(update)]
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
struct VersionRouter<S = DaemonEventSender<AppVersionInfo>, D = HttpAppDownloader<ProgressUpdater>>
{
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
    #[cfg(update)]
    app_upgrade_broadcast: AppUpgradeBroadcast,
    phantom: std::marker::PhantomData<D>, // Can we get around this?
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

struct AppVersionInfoEvent {
    app_version_info: AppVersionInfo,
    is_new: bool,
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
        let (refresh_version_check_tx, refresh_version_check_rx) = mpsc::unbounded();
        VersionUpdater::spawn(
            api_handle,
            availability_handle,
            cache_dir,
            new_version_tx,
            refresh_version_check_rx,
        )
        .await;

        VersionRouter {
            daemon_rx: rx,
            state: State::NoVersion,
            beta_program,
            version_event_sender,
            new_version_rx,
            version_request_channels: vec![],
            #[cfg(update)]
            app_upgrade_broadcast,
            refresh_version_check_tx,
            phantom: std::marker::PhantomData::<HttpAppDownloader<ProgressUpdater>>,
        }
        .run()
        .await;
    });
    VersionRouterHandle { tx }
}

impl<S, D> VersionRouter<S, D>
where
    S: Sender<AppVersionInfo> + Send + 'static,
    D: AppDownloader + Send + 'static,
    D: From<AppDownloaderParameters<ProgressUpdater>>,
{
    async fn run(mut self) {
        log::info!("Version router started");
        // Loop until the router is closed
        while self.run_step().await {}
        log::info!("Version router closed");
    }

    async fn run_step(&mut self) -> bool {
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
                // If the download was successful, we send the new version
                if let Some(app_update_info) =  res {
                    let _ = self.version_event_sender.send(app_update_info);
                }
            },
            Some(message) = self.daemon_rx.next() => self.handle_message(message),
            else => return false,
        }
        true
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
    fn on_new_version(&mut self, version_cache: VersionCache) -> AppVersionInfoEvent {
        let verified_installer_path = self.get_verified_installer_path();
        match &mut self.state {
            State::NoVersion => {
                // Receive first version
                let app_version_info = to_app_version_info(&version_cache, self.beta_program, None);
                self.state = State::HasVersion { version_cache };
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

                self.state = State::HasVersion { version_cache };

                AppVersionInfoEvent {
                    is_new: new_app_version != prev_app_version,
                    app_version_info: new_app_version,
                }
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
                let prev_app_version = to_app_version_info(
                    prev_cache,
                    self.beta_program,
                    verified_installer_path.clone(),
                );
                let new_app_version =
                    to_app_version_info(&version_cache, self.beta_program, verified_installer_path);

                let is_new = new_app_version != prev_app_version;
                // If version changed, cancel download by switching state
                if is_new {
                    log::warn!("Received new version while upgrading: {new_app_version:?}");
                    self.state = State::HasVersion { version_cache };
                } else {
                    *prev_cache = version_cache;
                };
                AppVersionInfoEvent {
                    app_version_info: new_app_version,
                    is_new,
                }
            }
        }
    }

    fn notify_version_requesters(&mut self, new_app_version_info: AppVersionInfo) {
        // Notify all requesters
        for tx in self.version_request_channels.drain(..) {
            let _ = tx.send(Ok(new_app_version_info.clone()));
        }
    }

    fn get_verified_installer_path(&self) -> Option<PathBuf> {
        match &self.state {
            #[cfg(update)]
            State::Downloaded {
                verified_installer_path,
                ..
            } => Some(verified_installer_path.clone()),
            _ => None,
        }
    }

    fn set_beta_program(&mut self, new_state: bool) {
        if new_state == self.beta_program {
            return;
        }
        let previous_state = self.beta_program;
        self.beta_program = new_state;
        let new_app_version_info = match self.state.get_version_cache() {
            Some(version_cache) => {
                let prev_app_version = to_app_version_info(version_cache, previous_state, None);
                let new_app_version = to_app_version_info(version_cache, new_state, None);

                // Update version info
                if new_app_version != prev_app_version {
                    new_app_version
                } else {
                    return;
                }
            }
            None => return,
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
        if let Err(_e) = self.refresh_version_check_tx.unbounded_send(()) {
            result_tx
                .send(Err(Error::VersionRouterClosed))
                .unwrap_or_else(|e| log::warn!("Failed to send version request result: {e:?}"));
        } else {
            // Append to response channels
            self.version_request_channels.push(result_tx);
        }
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

#[cfg(all(test, update))]
mod test {
    use super::downloader::ProgressUpdater;
    use futures::channel::mpsc::unbounded;
    use mullvad_types::version::AppUpgradeEvent;
    use mullvad_update::{app::DownloadError, fetch::ProgressUpdater as _};

    use super::*;

    // TODO: check if this needs to actually wrap the downloader parameters
    struct TestAppDownloader(AppDownloaderParameters<ProgressUpdater>);

    // TODO: Can we use normal async traits?
    #[async_trait::async_trait]
    impl AppDownloader for TestAppDownloader {
        async fn download_executable(&mut self) -> std::result::Result<(), DownloadError> {
            self.0.app_progress.set_progress(1.0);
            Ok(())
        }

        async fn verify(&mut self) -> std::result::Result<(), DownloadError> {
            Ok(())
        }

        async fn install(&mut self) -> std::result::Result<(), DownloadError> {
            Ok(())
        }
    }

    impl From<AppDownloaderParameters<ProgressUpdater>> for TestAppDownloader {
        fn from(parameters: AppDownloaderParameters<ProgressUpdater>) -> Self {
            Self(parameters)
        }
    }

    struct VersionRouterChannels {
        _daemon_tx: futures::channel::mpsc::UnboundedSender<Message>, // TODO: Test daemon commands?
        new_version_tx: futures::channel::mpsc::UnboundedSender<VersionCache>,
        refresh_version_check_rx: futures::channel::mpsc::UnboundedReceiver<()>,
        version_event_receiver: futures::channel::mpsc::UnboundedReceiver<AppVersionInfo>,
    }

    fn make_version_router() -> (
        VersionRouter<futures::channel::mpsc::UnboundedSender<AppVersionInfo>, TestAppDownloader>,
        VersionRouterChannels,
    ) {
        let (version_event_sender, version_event_receiver) = unbounded();
        let (daemon_tx, daemon_rx) = unbounded();
        let (app_upgrade_broadcast, _) = tokio::sync::broadcast::channel(1);
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
                phantom: std::marker::PhantomData::<TestAppDownloader>,
            },
            VersionRouterChannels {
                _daemon_tx: daemon_tx,
                new_version_tx,
                refresh_version_check_rx,
                version_event_receiver,
            },
        )
    }

    /// Create a version cache with a stable version that is newer than the current version
    fn get_stable_version_cache() -> VersionCache {
        let mut version: mullvad_version::Version = mullvad_version::VERSION.parse().unwrap();
        version.incremental += 1;
        VersionCache {
            current_version_supported: true,
            latest_version: VersionInfo {
                beta: None,
                stable: mullvad_update::version::Version {
                    version,
                    urls: vec!["https://example.com".to_string()],
                    size: 123456,
                    changelog: "Changelog".to_string(),
                    sha256: [0; 32],
                },
            },
        }
    }

    /// Create a version cache with a stable version that is newer than the current version
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
        VersionCache {
            current_version_supported: true,
            latest_version: VersionInfo {
                beta: Some(beta),
                stable,
            },
        }
    }

    #[test]
    fn test_upgrade_with_no_version() {
        let (mut version_router, _channels) = make_version_router::<TestAppDownloader>();
        let upgrade_events = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        assert!(
            matches!(version_router.state, State::NoVersion),
            "State should be not transition"
        );
        assert!(
            upgrade_events.is_empty(),
            "No upgrade events should be sent"
        );
    }

    #[test]
    fn test_new_beta_when_not_in_beta_program() {
        crate::logging::init_logger(log::LevelFilter::Debug, None, false)
            .expect("Failed to initialize logger");
        let (mut version_router, mut channels) = make_version_router::<TestAppDownloader>();
        let version_cache = get_new_beta_version_cache();
        version_router.set_beta_program(false); // This is default, but for clarity
        assert!(
            matches!(version_router.state, State::NoVersion),
            "State should be not transition"
        );
        version_router.on_new_version(version_cache);
        assert!(matches!(version_router.state, State::HasVersion { .. }));
        assert!(
            channels.version_event_receiver.try_next().is_err(),
            "No version event should be sent on beta program change"
        );
        let upgrade_events = version_router.app_upgrade_broadcast.subscribe();
        version_router.update_application();
        assert!(
            upgrade_events.is_empty(),
            "No upgrade events should be sent"
        );
    }

    /// Test that when the daemon calls `get_latest_version`, it will trigger a version check
    /// and send the result back to the daemon, both on the response channel and in the
    /// version event stream.
    #[tokio::test]
    async fn test_get_latest_version() {
        let (mut version_router, mut channels) = make_version_router::<TestAppDownloader>();
        let version_cache_test = get_new_stable_version_cache();
        let (tx, mut get_latest_version_rx) = oneshot::channel();
        channels
            .daemon_tx
            .unbounded_send(Message::GetLatestVersion(tx))
            .unwrap();
        version_router.run_step().await;

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

        // On the next cycle, the router should receive the version info
        // and send it to as a response to the oneshot from `get_latest_version`
        // and to the daemon (`version_event_receiver`)
        version_router.run_step().await;
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
        );
    }

    #[tokio::test]
    async fn test_upgrade() {
        let (mut version_router, mut channels) = make_version_router::<TestAppDownloader>();
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
                    version_cache_test.latest_version.stable.version
                );
            }
            other => panic!("State should be Downloading, was {other:?}"),
        }

        // Drive the download to completion, and get the verified installer path
        version_router.run_step().await;
        let verified_installer_path = match &version_router.state {
            State::Downloaded {
                version_cache,
                verified_installer_path,
                ..
            } => {
                assert_eq!(version_cache, &version_cache_test);
                verified_installer_path
            }
            other => panic!("State should be Downloading, was {other:?}"),
        };

        // Check that the app upgrade events were sent
        let events = [
            AppUpgradeEvent::DownloadStarting,
            AppUpgradeEvent::DownloadProgress(AppUpgradeDownloadProgress {
                progress: 100,
                server: "example.com".to_string(),
                time_left: None,
            }),
            AppUpgradeEvent::VerifyingInstaller,
            AppUpgradeEvent::VerifiedInstaller,
        ];
        for event in events {
            assert_eq!(app_upgrade_listener.try_recv().unwrap(), event);
        }
        assert_eq!(app_upgrade_listener.try_recv(), Err(TryRecvError::Empty));

        // Check that the version event was sent with the verified installer path
        let version_info = channels
            .version_event_receiver
            .try_next()
            .expect("Version event sender should not be closed")
            .expect("Version event should be sent");
        assert_eq!(
            version_info
                .suggested_upgrade
                .as_ref()
                .unwrap()
                .verified_installer_path,
            Some(verified_installer_path.clone())
        );
    }
}
