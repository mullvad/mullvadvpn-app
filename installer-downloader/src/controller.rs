//! This module implements the actual logic performed by different UI components.

use crate::delegate::{AppDelegate, AppDelegateQueue};
use crate::environment::Environment;
use crate::resource;
use crate::temp::DirectoryProvider;
use crate::ui_downloader::{UiAppDownloader, UiAppDownloaderParameters, UiProgressUpdater};

use mullvad_update::{
    api::VersionInfoProvider,
    app::{self, AppDownloader},
    version::{Version, VersionInfo, VersionParameters, ROLLOUT_ANY_VERSION},
};
use rand::seq::SliceRandom;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

/// ed25519 pubkey used to verify metadata from the Mullvad (stagemole) API
const VERSION_PROVIDER_PUBKEY: &str = include_str!("../../mullvad-update/stagemole-pubkey");

/// Pinned root certificate used when fetching version metadata
const PINNED_CERTIFICATE: &[u8] = include_bytes!("../../mullvad-api/le_root_cert.pem");

/// Actions handled by an async worker task in [ActionMessageHandler].
enum TaskMessage {
    SetVersionInfo(VersionInfo),
    BeginDownload,
    Cancel,
    TryBeta,
    TryStable,
}

/// See the [module-level docs](self).
pub struct AppController {}

/// Public entry function for registering a [AppDelegate].
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T, environment: Environment) {
    use mullvad_update::{api::HttpVersionInfoProvider, app::HttpAppDownloader};

    // App downloader to use
    type Downloader<T> = HttpAppDownloader<UiProgressUpdater<T>>;
    // Directory provider to use
    type DirProvider = crate::temp::TempDirProvider;

    // Version info provider to use
    let verifying_key =
        mullvad_update::format::key::VerifyingKey::from_hex(VERSION_PROVIDER_PUBKEY)
            .expect("valid key");
    let cert = reqwest::Certificate::from_pem(PINNED_CERTIFICATE).expect("invalid cert");
    let version_provider = HttpVersionInfoProvider {
        url: get_metadata_url(),
        pinned_certificate: Some(cert),
        verifying_key,
    };

    AppController::initialize::<_, Downloader<T>, _, DirProvider>(
        delegate,
        version_provider,
        environment,
    )
}

/// JSON files should be stored at `<base url>/<platform>.json`.
fn get_metadata_url() -> String {
    const PLATFORM: &str = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        panic!("Unsupported platform")
    };
    format!("https://releases.stagemole.eu/desktop/metadata/{PLATFORM}.json")
}

impl AppController {
    /// Initialize [AppController] using the provided delegate.
    ///
    /// Providing the downloader and version info fetcher as type arguments, they're decoupled from
    /// the logic of [AppController], allowing them to be mocked.
    pub fn initialize<D, A, V, DirProvider>(
        delegate: &mut D,
        version_provider: V,
        environment: Environment,
    ) where
        D: AppDelegate + 'static,
        V: VersionInfoProvider + Send + 'static,
        A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
        DirProvider: DirectoryProvider,
    {
        delegate.hide_download_progress();
        delegate.show_download_button();
        delegate.disable_download_button();
        delegate.hide_cancel_button();
        delegate.hide_beta_text();
        delegate.hide_stable_text();

        let (task_tx, task_rx) = mpsc::channel(1);
        tokio::spawn(ActionMessageHandler::<D, A>::run::<DirProvider>(
            delegate.queue(),
            task_tx.clone(),
            task_rx,
        ));
        delegate.set_status_text(resource::FETCH_VERSION_DESC);
        tokio::spawn(fetch_app_version_info::<D, V>(
            delegate.queue(),
            task_tx.clone(),
            version_provider,
            environment,
        ));
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
    download_tx: mpsc::Sender<TaskMessage>,
    version_provider: VersionProvider,
    Environment { architecture }: Environment,
) where
    Delegate: AppDelegate + 'static,
    VersionProvider: VersionInfoProvider + Send,
{
    loop {
        let version_params = VersionParameters {
            architecture,
            // For the downloader, the rollout version is always preferred
            rollout: ROLLOUT_ANY_VERSION,
            // The downloader allows any version
            lowest_metadata_version: 0,
        };

        let err = match version_provider.get_version_info(version_params).await {
            Ok(version_info) => {
                let _ = download_tx.try_send(TaskMessage::SetVersionInfo(version_info));
                return;
            }
            Err(err) => err,
        };

        log::error!("Failed to get version info: {err:?}");

        enum Action {
            Retry,
            Cancel,
        }

        let (action_tx, mut action_rx) = mpsc::channel(1);

        // show error message (needs to happen on the UI thread)
        // send Action when user presses a button to continue
        queue.queue_main(move |self_| {
            self_.hide_download_button();

            let (retry_tx, cancel_tx) = (action_tx.clone(), action_tx);

            self_.set_status_text("");
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
        let Some(action) = action_rx.recv().await else {
            panic!("channel was dropped? argh")
        };

        match action {
            Action::Retry => {
                continue;
            }
            Action::Cancel => {
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
    version_info: Option<VersionInfo>,
    active_download: Option<JoinHandle<()>>,
    target_version: TargetVersion,
    temp_dir: anyhow::Result<PathBuf>,

    _marker: std::marker::PhantomData<A>,
}

impl<D: AppDelegate + 'static, A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static>
    ActionMessageHandler<D, A>
{
    async fn run<DP: DirectoryProvider>(
        queue: D::Queue,
        tx: mpsc::Sender<TaskMessage>,
        mut rx: mpsc::Receiver<TaskMessage>,
    ) {
        let temp_dir = DP::create_download_dir().await;

        let mut handler = Self {
            queue,
            tx,
            version_info: None,
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
            TaskMessage::SetVersionInfo(new_version_info) => {
                self.handle_set_version_info(new_version_info);
            }
            TaskMessage::TryBeta => self.handle_try_beta(),
            TaskMessage::TryStable => self.handle_try_stable(),
            TaskMessage::BeginDownload => self.begin_download().await,
            TaskMessage::Cancel => self.cancel().await,
        }
    }

    fn handle_set_version_info(&mut self, new_version_info: &VersionInfo) {
        let version_label = format_latest_version(&new_version_info.stable);
        let has_beta = new_version_info.beta.is_some();
        self.queue.queue_main(move |self_| {
            self_.set_status_text(&version_label);
            self_.enable_download_button();
            if has_beta {
                self_.show_beta_text();
            }
        });
        self.version_info = Some(new_version_info.to_owned());
    }

    fn handle_try_beta(&mut self) {
        let Some(version_info) = self.version_info.as_ref() else {
            log::error!("Attempted 'try beta' before having version info");
            return;
        };
        let Some(beta_info) = version_info.beta.as_ref() else {
            log::error!("Attempted 'try beta' without beta version");
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
        let Some(version_info) = self.version_info.as_ref() else {
            log::error!("Attempted 'try stable' before having version info");
            return;
        };
        let stable_info = &version_info.stable;

        self.target_version = TargetVersion::Stable;
        let version_label = format_latest_version(stable_info);

        self.queue.queue_main(move |self_| {
            self_.hide_stable_text();
            self_.show_beta_text();
            self_.set_status_text(&version_label);
        });
    }

    async fn begin_download(&mut self) {
        if self.active_download.take().is_some() {
            log::debug!("Interrupting ongoing download");
        }
        let Some(version_info) = self.version_info.clone() else {
            log::error!("Attempted 'begin download' before having version info");
            return;
        };

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
                    self_.set_status_text("");
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

            self_.set_download_text("");
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
        if let Some(active_download) = self.active_download.take() {
            active_download.abort();
            let _ = active_download.await;
        }

        let Some(version_info) = self.version_info.as_ref() else {
            log::error!("Attempted 'cancel' before having version info");
            return;
        };

        let selected_version = match self.target_version {
            TargetVersion::Stable => &version_info.stable,
            TargetVersion::Beta => version_info.beta.as_ref().expect("selected version exists"),
        };

        let version_label = format_latest_version(selected_version);
        let has_beta = version_info.beta.is_some();
        let target_version = self.target_version;

        self.queue.queue_main(move |self_| {
            self_.set_status_text(&version_label);
            self_.set_download_text("");
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
            self_.set_download_progress(0);
        });
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
