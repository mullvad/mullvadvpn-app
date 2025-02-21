//! This module implements the actual logic performed by different UI components.

use crate::delegate::{AppDelegate, AppDelegateQueue, Button};
use crate::resource;
use crate::ui_downloader::{UiAppDownloader, UiAppDownloaderParameters, UiProgressUpdater};

use mullvad_update::{
    api::VersionInfoProvider,
    app::{self, AppDownloader},
    version::{Version, VersionArchitecture, VersionInfo, VersionParameters},
};
use rand::seq::SliceRandom;

use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

/// Actions handled by an async worker task in [handle_action_messages].
enum TaskMessage {
    SetVersionInfo(VersionInfo),
    BeginDownload,
    Cancel,
    TryBeta,
    TryStable,
}

/// Provide a directory to use for [AppDownloader]
pub trait DirectoryProvider: 'static {
    /// Provide a directory to use for [AppDownloader]
    fn create_download_dir() -> impl Future<Output = anyhow::Result<PathBuf>> + Send;
}

struct TempDirProvider;

impl DirectoryProvider for TempDirProvider {
    /// Create a locked-down directory to store downloads in
    fn create_download_dir() -> impl Future<Output = anyhow::Result<PathBuf>> + Send {
        mullvad_update::dir::admin_temp_dir()
    }
}

/// See the [module-level docs](self).
pub struct AppController {}

/// Public entry function for registering a [AppDelegate].
pub fn initialize_controller<T: AppDelegate + 'static>(delegate: &mut T) {
    use mullvad_update::{api::HttpVersionInfoProvider, app::HttpAppDownloader};

    // App downloader to use
    type Downloader<T> = HttpAppDownloader<UiProgressUpdater<T>>;
    // Directory provider to use
    type DirProvider = TempDirProvider;

    // Version info provider to use
    const TEST_PUBKEY: &str = include_str!("../../mullvad-update/test-pubkey");
    let verifying_key =
        mullvad_update::format::key::VerifyingKey::from_hex(TEST_PUBKEY).expect("valid key");
    let version_provider = HttpVersionInfoProvider {
        url: "https://releases.mullvad.net/thing".to_owned(),
        pinned_certificate: None,
        verifying_key,
    };
    let Some(architecture) = get_arch().ok().flatten() else {
        // Could not retrieve the host's CPU architecture for whatever reason
        delegate.queue_main(|self_| {
            self_.show_error_message(crate::delegate::ErrorMessage {
                status_text: "Could not detect your CPU architecture".to_owned(),
                cancel_button_text: resource::CANCEL_BUTTON_TEXT.to_owned(),
                retry_button: Button {
                    enabled: false,
                    ..Default::default()
                },
            });
        });
        return;
    };

    AppController::initialize::<_, Downloader<T>, _, DirProvider>(
        delegate,
        version_provider,
        architecture,
    )
}

impl AppController {
    /// Initialize [AppController] using the provided delegate.
    ///
    /// Providing the downloader and version info fetcher as type arguments, they're decoupled from
    /// the logic of [AppController], allowing them to be mocked.
    pub fn initialize<D, A, V, DirProvider>(
        delegate: &mut D,
        version_provider: V,
        architecture: VersionArchitecture,
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
        tokio::spawn(handle_action_messages::<D, A, DirProvider>(
            delegate.queue(),
            task_tx.clone(),
            task_rx,
        ));
        delegate.set_status_text(resource::FETCH_VERSION_DESC);
        tokio::spawn(fetch_app_version_info::<D, V>(
            delegate.queue(),
            task_tx.clone(),
            version_provider,
            architecture,
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
fn fetch_app_version_info<Delegate, VersionProvider>(
    queue: Delegate::Queue,
    download_tx: mpsc::Sender<TaskMessage>,
    version_provider: VersionProvider,
    architecture: VersionArchitecture,
) -> impl Future<Output = ()>
where
    Delegate: AppDelegate + 'static,
    VersionProvider: VersionInfoProvider + Send,
{
    async move {
        loop {
            let version_params = VersionParameters {
                architecture,
                // For the downloader, the rollout version is always preferred
                rollout: 1.,
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

            eprintln!("Failed to get version info: {err}");

            enum Action {
                Retry,
                Cancel,
            }

            let (action_tx, mut action_rx) = mpsc::channel(1);

            // show error message (needs to happen on the UI thread)
            // send Action when user presses a button to contin
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
                    retry_button: Button {
                        text: resource::FETCH_VERSION_ERROR_RETRY_BUTTON_TEXT.to_owned(),
                        ..Default::default()
                    },
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
}

#[derive(Clone, Copy, PartialEq)]
enum TargetVersion {
    Beta,
    Stable,
}

/// Async worker that handles actions such as initiating a download, cancelling it, and updating
/// labels.
async fn handle_action_messages<D, A, DirProvider>(
    queue: D::Queue,
    tx: mpsc::Sender<TaskMessage>,
    mut rx: mpsc::Receiver<TaskMessage>,
) where
    D: AppDelegate + 'static,
    A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
    DirProvider: DirectoryProvider,
{
    let mut version_info = None;
    let mut active_download = None;

    let mut target_version = TargetVersion::Stable;

    while let Some(msg) = rx.recv().await {
        match msg {
            TaskMessage::SetVersionInfo(new_version_info) => {
                let version_label = format_latest_version(&new_version_info.stable);
                let has_beta = new_version_info.beta.is_some();
                queue.queue_main(move |self_| {
                    self_.set_status_text(&version_label);
                    self_.enable_download_button();
                    if has_beta {
                        self_.show_beta_text();
                    }
                });
                version_info = Some(new_version_info);
            }
            TaskMessage::TryBeta => {
                let Some(version_info) = version_info.as_ref() else {
                    continue;
                };
                let Some(beta_info) = version_info.beta.as_ref() else {
                    continue;
                };

                target_version = TargetVersion::Beta;
                let version_label = format_latest_version(beta_info);

                queue.queue_main(move |self_| {
                    self_.show_stable_text();
                    self_.hide_beta_text();
                    self_.set_status_text(&version_label);
                });
            }
            TaskMessage::TryStable => {
                let Some(version_info) = version_info.as_ref() else {
                    continue;
                };
                let stable_info = &version_info.stable;

                target_version = TargetVersion::Stable;
                let version_label = format_latest_version(stable_info);

                queue.queue_main(move |self_| {
                    self_.hide_stable_text();
                    self_.show_beta_text();
                    self_.set_status_text(&version_label);
                });
            }
            TaskMessage::BeginDownload => {
                if let Some(_) = active_download.take() {
                    println!("Interrupting ongoing download");
                }
                let Some(version_info) = version_info.clone() else {
                    continue;
                };

                let (retry_tx, cancel_tx) = (tx.clone(), tx.clone());
                queue.queue_main(move |self_| {
                    self_.hide_error_message();
                    self_.on_error_message_retry(move || {
                        let _ = retry_tx.try_send(TaskMessage::BeginDownload);
                    });
                    self_.on_error_message_cancel(move || {
                        let _ = cancel_tx.try_send(TaskMessage::Cancel);
                    });
                });

                // Create temporary dir
                let download_dir = match DirProvider::create_download_dir().await {
                    Ok(dir) => dir,
                    Err(_err) => {
                        queue.queue_main(move |self_| {
                            self_.set_status_text("");
                            self_.hide_download_button();
                            self_.hide_beta_text();
                            self_.hide_stable_text();

                            self_.show_error_message(crate::delegate::ErrorMessage {
                                status_text: "Failed to create download directory".to_owned(),
                                cancel_button_text: "Cancel".to_owned(),
                                retry_button: Button {
                                    text: "Try again".to_owned(),
                                    ..Default::default()
                                },
                            });
                        });
                        continue;
                    }
                };

                // Begin download
                let (tx, rx) = oneshot::channel();
                queue.queue_main(move |self_| {
                    let selected_version = match target_version {
                        TargetVersion::Stable => &version_info.stable,
                        TargetVersion::Beta => {
                            version_info.beta.as_ref().expect("selected version exists")
                        }
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
                            eprintln!("install_and_upgrade failed: {err}");
                            let mut source = err.source();
                            while let Some(error) = source {
                                eprintln!("caused by: {error}");
                                source = error.source();
                            }
                        }
                    }));
                });
                active_download = rx.await.ok();
            }
            TaskMessage::Cancel => {
                if let Some(active_download) = active_download.take() {
                    active_download.abort();
                    let _ = active_download.await;
                }

                let Some(version_info) = version_info.as_ref() else {
                    continue;
                };

                let selected_version = match target_version {
                    TargetVersion::Stable => &version_info.stable,
                    TargetVersion::Beta => {
                        version_info.beta.as_ref().expect("selected version exists")
                    }
                };

                let version_label = format_latest_version(&selected_version);
                let has_beta = version_info.beta.is_some();

                queue.queue_main(move |self_| {
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

/// Try to map the host's CPU architecture to one of the CPU architectures the Mullvad VPN app
/// supports.
fn get_arch() -> Result<Option<VersionArchitecture>, std::io::Error> {
    match talpid_platform_metadata::get_native_arch()?? {
        talpid_platform_metadata::Architecture::X86 => VersionArchitecture::X86,
        talpid_platform_metadata::Architecture::Arm64 => VersionArchitecture::Arm64,
    }
}
