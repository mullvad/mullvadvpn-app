use std::mem;
use std::path::PathBuf;

use futures::channel::{mpsc, oneshot};
use futures::stream::StreamExt;
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
    pub async fn set_beta_program(&self, state: bool) -> Result<()> {
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

    pub async fn new_upgrade_event_listener(
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
    latest_notified_version: Option<AppVersionInfo>,
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
    Forwarding,
    Paused {
        /// Version check update received while paused
        new_version: Option<VersionCache>,
        //update_progress: mullvad_types::version::UpdateProgress,
    },
}

impl VersionRouter {
    pub fn spawn(
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

            Self {
                rx,
                state: RoutingState::Forwarding,
                beta_program,
                version_check,
                version_event_sender,
                new_version_rx,
                latest_notified_version: None,
            }
            .run();
        });
        VersionRouterHandle { tx }
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(new_version) = self.new_version_rx.next() => self.on_new_version(new_version).await,
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
                if let Ok(new_version) = self.version_check.get_version_info().await {
                    // Suggested upgrade might change with beta status even if the version is the same
                    self.on_new_version(new_version).await;
                }
                result_tx.send(()).unwrap();

                // TODO: figure out suggested upgrade. if it changed, send AppVersionInfo to `version_event_sender`
            }
            Message::GetLatestVersion(result_tx) => {
                let res = match self.version_check.get_version_info().await {
                    Ok(version) => {
                        self.on_new_version(version.clone()).await;
                        Ok(to_app_version_info(version, self.beta_program))
                    }
                    Err(e) => Err(e),
                };

                result_tx.send(res).unwrap();
            }
            Message::UpdateApplication { result_tx } => {
                let RoutingState::Forwarding = self.state else {
                    result_tx.send(()).unwrap();
                    return;
                };
                let last_version = match &self.latest_notified_version {
                    Some(version) => version,
                    None => {
                        let new_version = self
                            .version_check
                            .get_version_info()
                            .await
                            .expect("Failed to get version info");
                        let app_version_info = to_app_version_info(new_version, self.beta_program);
                        self.version_event_sender.send(app_version_info.clone());
                        self.latest_notified_version = Some(app_version_info);
                        self.latest_notified_version.as_ref().unwrap()
                    }
                };
                self.state = RoutingState::Paused { new_version: None };
                // TODO: start update
                result_tx.send(()).unwrap();
            }
            Message::CancelUpdate { result_tx } => {
                let state = mem::replace(&mut self.state, RoutingState::Forwarding);
                let RoutingState::Paused { new_version } = state else {
                    log::warn!("Cancel update called while not updating");
                    result_tx.send(()).unwrap();
                    return;
                };
                // TODO: Cancel update
                if let Some(new_version) = new_version {
                    self.on_new_version(new_version).await;
                }

                result_tx.send(()).unwrap();
            }
            Message::NewUpgradeEventListener { result_tx } => {
                todo!();
            }
        }
    }

    /// Handle new version info
    ///
    /// If the router is forwarding and the version is newer than the last, it will propagate the
    /// version info. If the router is paused, it will store the new version info until the router
    /// is resumed.
    async fn on_new_version(&mut self, version: VersionCache) {
        match self.state {
            RoutingState::Forwarding => {
                let app_version_info = to_app_version_info(version, self.beta_program);
                if self.latest_notified_version.as_ref() != Some(&app_version_info) {
                    self.latest_notified_version = Some(app_version_info.clone());
                    self.version_event_sender.send(app_version_info);
                }
            }
            RoutingState::Paused {
                ref mut new_version,
            } => {
                *new_version = Some(version);
            }
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

    let app_version_info = mullvad_types::version::AppVersionInfo {
        current_version_supported: cache.current_version_supported,
        suggested_upgrade: Some(SuggestedUpgrade {
            version: version.version,
            changelog: Some(version.changelog),
            // TODO: This should return the downloaded & verified path, if it exists
            verified_installer_path: None,
        }),
    };
    app_version_info
}
