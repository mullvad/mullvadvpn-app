use std::path::PathBuf;

use futures::channel::{mpsc, oneshot};
use futures::stream::StreamExt;
use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle};
use mullvad_types::version::SuggestedUpgrade;
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

pub struct VersionRouter {
    rx: mpsc::UnboundedReceiver<Message>,
    state: UpdateState,
    beta_program: bool,
    version_event_sender: DaemonEventSender<mullvad_types::version::AppVersionInfo>,
    /// Version updater
    version_check: check::VersionUpdaterHandle,
    /// Channel used to receive updates from `version_check`
    new_version_rx: mpsc::UnboundedReceiver<VersionCache>,
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

enum UpdateState {
    Idle,
    Downloading {
        //version: mullvad_update::version::Version,
        //update_progress: mullvad_types::version::UpdateProgress,
    },
    Verifying,
    Verified,
    Error,
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
                state: UpdateState::Idle,
                beta_program,
                version_check,
                version_event_sender,
                new_version_rx,
            }
            .run();
        });
        VersionRouterHandle { tx }
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                new_version = self.new_version_rx.next() => {
                    // TODO: figure out suggested upgrade. if it changed, send AppVersionInfo to `version_event_sender`

                    todo!()
                },
                message = self.rx.next() => {
                    let Some(message) = message else {
                        break;
                    };

                    self.handle_message(message).await;
                },
                // TODO: wait for version updater msg
                // TODO: wait for user message (Message)
            }
        }

        todo!()
    }

    /// Handle [Message] sent by user
    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::SetBetaProgram { state, result_tx } => {
                self.beta_program = state;
                if let Ok(upgrade) = self.suggested_upgrade().await {
                    // Note: A bit wrong. We unconditionally send a version notification here
                    // Ideally, we'd only send it if the suggested version actually changed
                    self.version_event_sender.send(upgrade);
                }
                result_tx.send(()).unwrap();

                // TODO: figure out suggested upgrade. if it changed, send AppVersionInfo to `version_event_sender`
            }
            Message::GetLatestVersion(result_tx) => {
                let version = self.suggested_upgrade().await;
                if let Ok(upgrade) = &version {
                    self.version_event_sender.send(upgrade.clone());
                }
                result_tx.send(version).unwrap();
            }
            Message::UpdateApplication { result_tx } => {
                self.state = UpdateState::Downloading {
                    //version: self.version_check.get_version_info().await.version,
                };
                todo!();
                result_tx.send(()).unwrap();
            }
            Message::CancelUpdate { result_tx } => {
                todo!();
                result_tx.send(()).unwrap();
            }
            Message::NewUpgradeEventListener { result_tx } => {
                todo!();
            }
        }
    }

    /// Convert [VersionCache] and beta program setting to [mullvad_types::version::AppVersionInfo]
    async fn suggested_upgrade(&self) -> Result<mullvad_types::version::AppVersionInfo> {
        let cache = self.version_check.get_version_info().await?;

        let version = if self.beta_program {
            cache
                .latest_version
                .beta
                .unwrap_or(cache.latest_version.stable)
        } else {
            cache.latest_version.stable
        };

        Ok(mullvad_types::version::AppVersionInfo {
            supported: cache.supported,
            suggested_upgrade: Some(SuggestedUpgrade {
                version: version.version,
                changelog: Some(version.changelog),
                // TODO: This should return the downloaded & verified path, if it exists
                verified_installer_path: None,
            }),
        })
    }
}
