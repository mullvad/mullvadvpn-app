//! This module implements the actual logic performed by different UI components.

use crate::{
    delegate::{AppDelegate, AppDelegateQueue},
    environment::Environment,
    resource::{self, VERIFYING_CACHED},
    temp::DirectoryProvider,
    ui_downloader::{UiAppDownloader, UiAppDownloaderParameters, UiProgressUpdater},
};

use mullvad_update::{
    api::{HttpVersionInfoProvider, MetaRepositoryPlatform},
    app::{self, AppCache, AppDownloader, DownloadedInstaller, HttpAppDownloader},
    local::METADATA_FILENAME,
    version::{Metadata, SUPPORTED_VERSION, VersionInfo, VersionParameters},
    version_provider::VersionInfoProvider,
};
use rand::seq::IndexedRandom;
use std::{cmp::Ordering, path::PathBuf};
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

struct WorkingDirectory {
    pub directory: PathBuf,
}

impl WorkingDirectory {
    pub async fn new<D: DirectoryProvider>() -> anyhow::Result<WorkingDirectory> {
        let directory = D::create_download_dir().await?;
        Ok(Self { directory })
    }
}

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

    let platform = MetaRepositoryPlatform::current().expect("current platform must be supported");
    let version_provider = HttpVersionInfoProvider::from(platform);

    #[cfg(target_os = "windows")]
    type CacheDir = mullvad_update::local::AppCacheDir;

    #[cfg(target_os = "macos")]
    type CacheDir = mullvad_update::local::NoopAppCacheDir;

    AppController::initialize::<_, Downloader<T>, CacheDir, DirProvider>(
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
    pub fn initialize<D, A, C, DirProvider>(
        delegate: &mut D,
        mut version_provider: impl VersionInfoProvider + Send + 'static,
        environment: Environment,
    ) where
        D: AppDelegate + 'static,
        A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
        C: AppCache + 'static,
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
            let working_dir = loop {
                match WorkingDirectory::new::<DirProvider>().await {
                    Ok(directory) => break directory,
                    Err(err) => {
                        log::error!("Failed to create temporary directory: {err:?}");

                        let (retry_tx, mut retry_rx) = mpsc::channel(1);

                        queue.queue_main(move |self_| {
                            self_.clear_status_text();
                            self_.hide_download_button();
                            self_.hide_beta_text();
                            self_.hide_stable_text();

                            let cancel_tx = retry_tx.clone();
                            self_.on_error_message_cancel(move || {
                                let _ = cancel_tx.try_send(false);
                            });
                            self_.on_error_message_retry(move || {
                                let _ = retry_tx.try_send(true);
                            });

                            self_.show_error_message(crate::delegate::ErrorMessage {
                                status_text: resource::CREATE_TEMPDIR_FAILED.to_owned(),
                                cancel_button_text:
                                    resource::CREATE_TEMPDIR_FAILED_CANCEL_BUTTON_TEXT.to_owned(),
                                retry_button_text:
                                    resource::CREATE_TEMPDIR_FAILED_RETRY_BUTTON_TEXT.to_owned(),
                            });
                        });

                        if !retry_rx.recv().await.unwrap() {
                            queue.queue_main(|self_| {
                                self_.quit();
                            });
                            std::future::pending::<()>().await;
                        }
                    }
                }
            };

            let metadata_path = working_dir.directory.join(METADATA_FILENAME);
            version_provider.set_metadata_dump_path(metadata_path);

            let version_info = fetch_app_version_info::<D, C>(
                queue.clone(),
                version_provider,
                &working_dir,
                environment,
            )
            .await;
            let version_label = format_latest_version(&version_info.stable);
            let has_beta = version_info.beta.is_some();
            queue.queue_main(move |self_| {
                self_.set_status_text(&version_label);
                self_.enable_download_button();
                if has_beta {
                    self_.show_beta_text();
                }
            });

            ActionMessageHandler::<D, A>::run(
                queue,
                task_tx_clone,
                task_rx,
                working_dir,
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
async fn fetch_app_version_info<Delegate, Cache>(
    queue: Delegate::Queue,
    version_provider: impl VersionInfoProvider + Send,
    working_directory: &WorkingDirectory,
    Environment { architecture }: Environment,
) -> VersionInfo
where
    Delegate: AppDelegate + 'static,
    Cache: AppCache + 'static,
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
            rollout: SUPPORTED_VERSION,
            allow_empty: false,
            // The downloader allows any version
            lowest_metadata_version: mullvad_update::version::MIN_VERIFY_METADATA_VERSION,
        };

        let err = match version_provider.get_version_info(&version_params).await {
            Ok(version_info) => return version_info,
            Err(err) => err,
        };

        log::error!("Failed to get version info: {err:?}");

        enum Action<Cache: AppCache> {
            Retry,
            Cancel,
            InstallExistingVersion {
                cached_app_installer: Cache::Installer,
            },
        }

        let (action_tx, mut action_rx) = mpsc::channel::<Action<Cache>>(1);

        // Check if we've already downloaded an installer.
        // If so, the user will be given the option to run it.
        let cache = Cache::new(working_directory.directory.clone(), version_params);
        // Present the 'first' available installer. In this case, we will always present
        // the latest app version installer available, as we suspect that this is the app
        // version the user wants to install. This could be made more dynamic (i.e. a user
        // interaction instead).
        let installer = cache.get_metadata().await.ok().and_then(|m| {
            cache.get_cached_installers(m).into_iter().max_by(|a, b| {
                a.version()
                    .partial_cmp(b.version())
                    .unwrap_or(Ordering::Equal)
            })
        });

        match installer {
            Some(cached_app_installer) => {
                queue.queue_main(move |self_| {
                    self_.hide_download_button();

                    let (retry_tx, cancel_tx) = (action_tx.clone(), action_tx);

                    self_.clear_status_text();
                    self_.on_error_message_retry(move || {
                        let _ = retry_tx.try_send(Action::Retry);
                    });

                    let on_cancel = {
                        let cached_app_installer = cached_app_installer.clone();
                        move || {
                            let cancel_action = Action::InstallExistingVersion {
                                cached_app_installer: cached_app_installer.clone(),
                            };
                            let _ = cancel_tx.try_send(cancel_action);
                        }
                    };
                    self_.on_error_message_cancel(on_cancel);

                    self_.show_error_message(crate::delegate::ErrorMessage {
                        status_text: resource::FETCH_VERSION_ERROR_DESC_WITH_EXISTING_DOWNLOAD
                            .replace("%s", &cached_app_installer.version().to_string()),
                        cancel_button_text: resource::FETCH_VERSION_ERROR_INSTALL_BUTTON_TEXT
                            .to_owned(),
                        retry_button_text: resource::FETCH_VERSION_ERROR_RETRY_BUTTON_TEXT
                            .to_owned(),
                    });
                });
            }
            _ => {
                log::info!("Couldn't find a downloaded installer");
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
                        cancel_button_text: resource::FETCH_VERSION_ERROR_CANCEL_BUTTON_TEXT
                            .to_owned(),
                        retry_button_text: resource::FETCH_VERSION_ERROR_RETRY_BUTTON_TEXT
                            .to_owned(),
                    });
                })
            }
        };

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
                std::future::pending::<()>().await;
            }
            Action::InstallExistingVersion {
                cached_app_installer: installer,
            } => {
                let (action_tx, mut action_rx) = mpsc::channel::<Option<Action<Cache>>>(1);
                let (retry_tx, cancel_tx) = (action_tx.clone(), action_tx.clone());

                queue.queue_main(|self_| {
                    self_.hide_error_message();
                    self_.clear_download_text();
                    self_.hide_download_button();
                    self_.hide_beta_text();
                    self_.hide_stable_text();
                    self_.show_cancel_button();
                    self_.disable_cancel_button();
                    self_.hide_download_progress();
                    self_.set_status_text(VERIFYING_CACHED);

                    let ui_installer = UiAppDownloader::new(&*self_, installer);

                    self_.on_error_message_retry(move || {
                        let _ = retry_tx.try_send(Some(Action::Retry));
                    });
                    self_.on_error_message_cancel(move || {
                        let _ = cancel_tx.try_send(Some(Action::Cancel));
                    });

                    tokio::spawn(async move {
                        if let Err(err) = app::install_and_upgrade(ui_installer).await {
                            log::error!("install_and_upgrade failed: {err:?}");
                        } else {
                            let _ = action_tx.send(None).await;
                        }
                    });
                });

                match action_rx.recv().await.unwrap() {
                    None => {
                        // If the verification was successful, wait for the app to shut down instead of looping
                        let () = std::future::pending().await;
                    }
                    Some(Action::Retry) => {
                        log::debug!("Retrying to fetch version info");
                        continue;
                    }
                    Some(Action::Cancel) => {
                        log::debug!("Cancelling fetching version info");
                        queue.queue_main(|self_| {
                            self_.quit();
                        });
                        std::future::pending::<()>().await;
                    }
                    Some(Action::InstallExistingVersion { .. }) => unreachable!(),
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
struct ActionMessageHandler<
    D: AppDelegate + 'static,
    A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static,
> {
    queue: D::Queue,
    tx: mpsc::Sender<TaskMessage>,
    version_info: VersionInfo,
    active_download: Option<JoinHandle<()>>,
    target_version: TargetVersion,
    working_directory: WorkingDirectory,

    _marker: std::marker::PhantomData<A>,
}

impl<D: AppDelegate + 'static, A: From<UiAppDownloaderParameters<D>> + AppDownloader + 'static>
    ActionMessageHandler<D, A>
{
    /// Run the [ActionMessageHandler] actor until the end of the program/execution
    async fn run(
        queue: D::Queue,
        tx: mpsc::Sender<TaskMessage>,
        mut rx: mpsc::Receiver<TaskMessage>,
        working_directory: WorkingDirectory,
        version_info: VersionInfo,
    ) {
        let mut handler = Self {
            queue,
            tx,
            version_info,
            active_download: None,
            target_version: TargetVersion::Stable,
            working_directory,

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
        let Some(beta_info) = self.version_info.beta.as_ref() else {
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

        let download_dir = self.working_directory.directory.clone();
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
                if let Err(err) = app::download_install_and_upgrade(ui_downloader).await {
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
    urls.choose(&mut rand::rng()).map(String::as_str)
}

fn format_latest_version(metadata: &Metadata) -> String {
    format!("{}: {}", resource::LATEST_VERSION_PREFIX, metadata.version)
}
