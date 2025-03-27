//! This module implements the actual logic performed by different UI components.

use crate::{
    delegate::{AppDelegate, AppDelegateQueue},
    environment::Environment,
    resource,
    temp::DirectoryProvider,
    ui_downloader::{UiAppDownloader, UiAppDownloaderParameters, UiProgressUpdater},
};

use mullvad_update::{
    api::{HttpVersionInfoProvider, VersionInfoProvider},
    app::{self, AppDownloader, HttpAppDownloader},
    version::{Version, VersionInfo, VersionParameters},
};
use rand::seq::SliceRandom;
use std::path::PathBuf;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

/// Actions handled by an async worker task in [ActionMessageHandler].
enum TaskMessage {
    BeginDownload,
    Cancel,
    TryBeta,
    TryStable,
}

/// See the [module-level docs](self).
pub struct AppController {}

/// Public entry function for registering a [AppDelegate].
///
/// This function uses the Mullvad API to fetch the current releases, a hardcoded public key to
/// verify the metadata, and the default HTTP client from `mullvad-update` and stores the files
/// in a temporary directory.
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T, environment: Environment) {
    // App downloader to use
    type Downloader<T> = HttpAppDownloader<UiProgressUpdater<T>>;
    // Directory provider to use
    type DirProvider = crate::temp::TempDirProvider;
    let version_provider = HttpVersionInfoProvider::trusted_provider();

    AppController::initialize::<_, Downloader<T>, _, DirProvider>(
        delegate,
        version_provider,
        environment,
    )
}

impl AppController {
    /// Initialize [AppController] using the provided delegate.
    ///
    /// This function lets the caller provide a version information provider, download client, etc.,
    /// which is useful for testing.
    pub fn initialize<D, A, V, DirProvider>(
        delegate: &mut D,
        version_provider: V,
        environment: Environment,
    ) where
        D: AppDelegate + 'static,
        V: VersionInfoProvider + Send + 'static,
        A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
        DirProvider: DirectoryProvider + 'static,
    {
        delegate.hide_download_progress();
        delegate.show_download_button();
        delegate.disable_download_button();
        delegate.hide_cancel_button();
        delegate.hide_beta_text();
        delegate.hide_stable_text();

        let (task_tx, task_rx) = mpsc::channel(1);
        let queue = delegate.queue();
        let task_tx_clone = task_tx.clone();
        tokio::spawn(async move {
            let version_info =
                fetch_app_version_info::<D, V>(queue.clone(), version_provider, environment).await;
            let version_label = format_latest_version(&version_info.stable);
            let has_beta = version_info.beta.is_some();
            queue.queue_main(move |self_| {
                self_.set_status_text(&version_label);
                self_.enable_download_button();
                if has_beta {
                    self_.show_beta_text();
                }
            });

            ActionMessageHandler::<D, A>::run::<DirProvider>(
                queue,
                task_tx_clone,
                task_rx,
                version_info,
            )
            .await;
        });

        Self::register_user_action_callbacks(delegate, task_tx);
    }

    fn register_user_action_callbacks<T: AppDelegate + 'static>(
        delegate: &mut T,
        task_tx: mpsc::Sender<TaskMessage>,
    ) {
        let tx = task_tx.clone();
        delegate.on_download(move || {
            let _ = tx.try_send(TaskMessage::BeginDownload);
        });
        let tx = task_tx.clone();
        delegate.on_cancel(move || {
            let _ = tx.try_send(TaskMessage::Cancel);
        });
        let tx = task_tx.clone();
        delegate.on_beta_link(move || {
            let _ = tx.try_send(TaskMessage::TryBeta);
        });
        let tx = task_tx.clone();
        delegate.on_stable_link(move || {
            let _ = tx.try_send(TaskMessage::TryStable);
        });
    }
}

/// Background task that fetches app version data.
async fn fetch_app_version_info<Delegate, VersionProvider>(
    queue: Delegate::Queue,
    version_provider: VersionProvider,
    Environment { architecture }: Environment,
) -> VersionInfo
where
    Delegate: AppDelegate,
    VersionProvider: VersionInfoProvider + Send,
{
    loop {
        queue.queue_main(|self_| {
            self_.show_download_button();
            self_.set_status_text(resource::FETCH_VERSION_DESC);
            self_.hide_error_message();
        });
        let version_params = VersionParameters {
            architecture,
            // For the downloader, the rollout version is always preferred
            rollout: mullvad_update::version::IGNORE,
            // The downloader allows any version
            lowest_metadata_version: 0,
        };

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let err = match version_provider.get_version_info(version_params).await {
            Ok(version_info) => {
                return version_info;
            }
            Err(err) => err,
        };

        log::error!("Failed to get version info: {err:?}");

        enum Action {
            Retry,
            Cancel,
        }

        let (action_tx, mut action_rx) = mpsc::channel(1);

        // show error message (needs to happen on the UI (main) thread)
        // send Action when user presses a button to continue
        queue.queue_main(move |self_| {
            self_.hide_download_button();

            let (retry_tx, cancel_tx) = (action_tx.clone(), action_tx);

            self_.clear_status_text();
            self_.on_error_message_retry(move || {
                let _ = retry_tx.try_send(Action::Retry);
            });
            self_.on_error_message_cancel(move || {
                let _ = cancel_tx.try_send(Action::Cancel);
            });
            self_.show_error_message(crate::delegate::ErrorMessage {
                status_text: resource::FETCH_VERSION_ERROR_DESC.to_owned(),
                cancel_button_text: resource::FETCH_VERSION_ERROR_CANCEL_BUTTON_TEXT.to_owned(),
                retry_button_text: resource::FETCH_VERSION_ERROR_RETRY_BUTTON_TEXT.to_owned(),
            });
        });

        // wait for user to press either button
        let action = action_rx.recv().await.expect("sender unexpectedly dropped");

        match action {
            Action::Retry => {
                log::debug!("Retrying to fetch version info");
                continue;
            }
            Action::Cancel => {
                log::debug!("Cancelling fetching version info");
                queue.queue_main(|self_| {
                    self_.quit();
                });
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TargetVersion {
    Beta,
    Stable,
}

/// Async worker that handles actions such as initiating a download, cancelling it, and updating
/// labels.
struct ActionMessageHandler<
    D: AppDelegate + 'static,
    A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
> {
    queue: D::Queue,
    tx: mpsc::Sender<TaskMessage>,
    version_info: VersionInfo,
    active_download: Option<JoinHandle<()>>,
    target_version: TargetVersion,
    temp_dir: anyhow::Result<PathBuf>,

    _marker: std::marker::PhantomData<A>,
}

impl<D: AppDelegate + 'static, A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static>
    ActionMessageHandler<D, A>
{
    /// Run the [ActionMessageHandler] actor until the end of the program/execution
    async fn run<DP: DirectoryProvider>(
        queue: D::Queue,
        tx: mpsc::Sender<TaskMessage>,
        mut rx: mpsc::Receiver<TaskMessage>,
        version_info: VersionInfo,
    ) {
        let temp_dir = DP::create_download_dir().await;

        let mut handler = Self {
            queue,
            tx,
            version_info,
            active_download: None,
            target_version: TargetVersion::Stable,
            temp_dir,

            _marker: std::marker::PhantomData,
        };

        while let Some(msg) = rx.recv().await {
            handler.handle_message(&msg).await;
        }
    }

    async fn handle_message(&mut self, msg: &TaskMessage) {
        match msg {
            TaskMessage::TryBeta => self.handle_try_beta(),
            TaskMessage::TryStable => self.handle_try_stable(),
            TaskMessage::BeginDownload => self.begin_download().await,
            TaskMessage::Cancel => self.cancel().await,
        }
    }

    fn handle_try_beta(&mut self) {
        log::error!("Attempted 'try beta' without beta version");
        let Some(beta_info) = self.version_info.beta.as_ref() else {
            return;
        };

        self.target_version = TargetVersion::Beta;
        let version_label = format_latest_version(beta_info);

        self.queue.queue_main(move |self_| {
            self_.show_stable_text();
            self_.hide_beta_text();
            self_.set_status_text(&version_label);
        });
    }

    fn handle_try_stable(&mut self) {
        let stable_info = &self.version_info.stable;

        self.target_version = TargetVersion::Stable;
        let version_label = format_latest_version(stable_info);

        self.queue.queue_main(move |self_| {
            self_.hide_stable_text();
            self_.show_beta_text();
            self_.set_status_text(&version_label);
        });
    }

    async fn begin_download(&mut self) {
        self.cancel_download().await;

        let (retry_tx, cancel_tx) = (self.tx.clone(), self.tx.clone());
        self.queue.queue_main(move |self_| {
            self_.hide_error_message();
            self_.on_error_message_retry(move || {
                let _ = retry_tx.try_send(TaskMessage::BeginDownload);
            });
            self_.on_error_message_cancel(move || {
                let _ = cancel_tx.try_send(TaskMessage::Cancel);
            });
        });

        // Create temporary dir
        let download_dir = match &self.temp_dir {
            Ok(dir) => dir.clone(),
            Err(error) => {
                log::error!("Failed to create temporary directory: {error:?}");

                self.queue.queue_main(move |self_| {
                    self_.clear_status_text();
                    self_.hide_download_button();
                    self_.hide_beta_text();
                    self_.hide_stable_text();

                    self_.show_error_message(crate::delegate::ErrorMessage {
                        status_text: resource::DOWNLOAD_FAILED_DESC.to_owned(),
                        cancel_button_text: resource::DOWNLOAD_FAILED_CANCEL_BUTTON_TEXT.to_owned(),
                        retry_button_text: resource::DOWNLOAD_FAILED_RETRY_BUTTON_TEXT.to_owned(),
                    });
                });
                return;
            }
        };

        log::debug!("Download directory: {}", download_dir.display());

        // Begin download
        let (tx, rx) = oneshot::channel();
        let target_version = self.target_version;
        let version_info = self.version_info.clone();
        self.queue.queue_main(move |self_| {
            let selected_version = match target_version {
                TargetVersion::Stable => &version_info.stable,
                TargetVersion::Beta => version_info.beta.as_ref().expect("selected version exists"),
            };

            let Some(app_url) = select_cdn_url(&selected_version.urls) else {
                return;
            };
            let app_version = selected_version.version.clone();
            let app_sha256 = selected_version.sha256;
            let app_size = selected_version.size;

            self_.clear_download_text();
            self_.hide_download_button();
            self_.hide_beta_text();
            self_.hide_stable_text();
            self_.show_cancel_button();
            self_.enable_cancel_button();
            self_.show_download_progress();

            let downloader = A::from(UiAppDownloaderParameters {
                app_version,
                app_url: app_url.to_owned(),
                app_size,
                app_progress: UiProgressUpdater::new(self_.queue()),
                app_sha256,
                cache_dir: download_dir,
            });

            let ui_downloader = UiAppDownloader::new(self_, downloader);
            let _ = tx.send(tokio::spawn(async move {
                if let Err(err) = app::install_and_upgrade(ui_downloader).await {
                    log::error!("install_and_upgrade failed: {err:?}");
                }
            }));
        });
        self.active_download = rx.await.ok();
    }

    async fn cancel(&mut self) {
        self.cancel_download().await;

        let selected_version = match self.target_version {
            TargetVersion::Stable => &self.version_info.stable,
            TargetVersion::Beta => self
                .version_info
                .beta
                .as_ref()
                .expect("selected version exists"),
        };

        let version_label = format_latest_version(selected_version);
        let has_beta = self.version_info.beta.is_some();
        let target_version = self.target_version;

        self.queue.queue_main(move |self_| {
            self_.set_status_text(&version_label);
            self_.clear_download_text();
            self_.show_download_button();
            self_.hide_error_message();

            if target_version == TargetVersion::Stable {
                if has_beta {
                    self_.show_beta_text();
                }
            } else {
                self_.show_stable_text();
            }

            self_.hide_cancel_button();
            self_.hide_download_progress();
            self_.clear_download_progress();
        });
    }

    async fn cancel_download(&mut self) {
        if let Some(active_download) = self.active_download.take() {
            log::debug!("Interrupting ongoing download");
            active_download.abort();
            let _ = active_download.await;
        }
    }
}

/// Select a mirror to download from
/// Currently, the selection is random
fn select_cdn_url(urls: &[String]) -> Option<&str> {
    urls.choose(&mut rand::thread_rng()).map(String::as_str)
}

fn format_latest_version(version: &Version) -> String {
    format!("{}: {}", resource::LATEST_VERSION_PREFIX, version.version)
}
