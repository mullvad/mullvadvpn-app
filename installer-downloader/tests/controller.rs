//! Tests for integrations between UI controller and other components
//!
//! The tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
//! then most likely the snapshots need to be updated. The most convenient way to review
//! changes to, and update, snapshots are by running `cargo insta review`.

use insta::assert_yaml_snapshot;
use installer_downloader::controller::{AppController, DirectoryProvider};
use installer_downloader::delegate::{AppDelegate, AppDelegateQueue};
use installer_downloader::ui_downloader::UiAppDownloaderParameters;
use mullvad_update::api::{Version, VersionInfo, VersionInfoProvider, VersionParameters};
use mullvad_update::app::{AppDownloader, DownloadError};
use mullvad_update::fetch::ProgressUpdater;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use std::vec::Vec;

pub struct FakeVersionInfoProvider {}

static FAKE_VERSION: LazyLock<VersionInfo> = LazyLock::new(|| VersionInfo {
    stable: Version {
        version: "2025.1".parse().unwrap(),
        urls: vec!["https://mullvad.net/fakeapp".to_owned()],
        size: 1234,
        changelog: "a changelog".to_owned(),
        sha256: [0u8; 32],
    },
    beta: None,
});

#[async_trait::async_trait]
impl VersionInfoProvider for FakeVersionInfoProvider {
    async fn get_version_info(_params: VersionParameters) -> anyhow::Result<VersionInfo> {
        Ok(FAKE_VERSION.clone())
    }
}

pub struct FakeDirectoryProvider<const SUCCEED: bool> {}

impl<const SUCCEEDED: bool> DirectoryProvider for FakeDirectoryProvider<SUCCEEDED> {
    async fn create_download_dir() -> anyhow::Result<PathBuf> {
        if SUCCEEDED {
            Ok(Path::new("/tmp/fake").to_owned())
        } else {
            anyhow::bail!("Failed to create directory");
        }
    }
}

/// Downloader for which all steps immediately succeed
pub type FakeAppDownloaderHappyPath = FakeAppDownloader<true, true, true>;

/// Downloader for which the download step fails
pub type FakeAppDownloaderDownloadFail = FakeAppDownloader<false, false, false>;

/// Downloader for which the verification step fails
pub type FakeAppDownloaderVerifyFail = FakeAppDownloader<true, false, false>;

impl<const A: bool, const B: bool, const C: bool> From<UiAppDownloaderParameters<FakeAppDelegate>>
    for FakeAppDownloader<A, B, C>
{
    fn from(params: UiAppDownloaderParameters<FakeAppDelegate>) -> Self {
        FakeAppDownloader { params }
    }
}

/// Fake app downloader
///
/// Parameters:
/// * EXE_SUCCEED - whether fetching the binary succeeds
/// * VERIFY_SUCCEED - whether verifying the binary succeeds
/// * LAUNCH_SUCCEED - whether launching the binary succeeds
pub struct FakeAppDownloader<
    const EXE_SUCCEED: bool,
    const VERIFY_SUCCEED: bool,
    const LAUNCH_SUCCEED: bool,
> {
    params: UiAppDownloaderParameters<FakeAppDelegate>,
}

#[async_trait::async_trait]
impl<const EXE_SUCCEED: bool, const VERIFY_SUCCEED: bool, const LAUNCH_SUCCEED: bool> AppDownloader
    for FakeAppDownloader<EXE_SUCCEED, VERIFY_SUCCEED, LAUNCH_SUCCEED>
{
    async fn download_executable(&mut self) -> Result<(), DownloadError> {
        self.params.app_progress.set_url(&self.params.app_url);
        self.params.app_progress.set_progress(0.);
        if EXE_SUCCEED {
            self.params.app_progress.set_progress(1.);
            Ok(())
        } else {
            Err(DownloadError::FetchApp(anyhow::anyhow!(
                "fetching app failed"
            )))
        }
    }

    async fn verify(&mut self) -> Result<(), DownloadError> {
        if VERIFY_SUCCEED {
            Ok(())
        } else {
            Err(DownloadError::Verification(anyhow::anyhow!(
                "verification failed"
            )))
        }
    }

    async fn install(&mut self) -> Result<(), DownloadError> {
        if LAUNCH_SUCCEED {
            Ok(())
        } else {
            Err(DownloadError::InstallFailed(anyhow::anyhow!(
                "install failed"
            )))
        }
    }
}

/// A fake queue that stores callbacks so that tests can run them later.
#[derive(Clone, Default)]
pub struct FakeQueue {
    callbacks: Arc<Mutex<Vec<Box<dyn FnOnce(&mut FakeAppDelegate) + Send>>>>,
}

impl FakeQueue {
    /// Run all queued callbacks on the given delegate.
    fn run_callbacks(&self, delegate: &mut FakeAppDelegate) {
        let mut callbacks = self.callbacks.lock().unwrap();
        for cb in callbacks.drain(..) {
            cb(delegate);
        }
    }
}

impl AppDelegateQueue<FakeAppDelegate> for FakeQueue {
    fn queue_main<F: FnOnce(&mut FakeAppDelegate) + 'static + Send>(&self, callback: F) {
        self.callbacks.lock().unwrap().push(Box::new(callback));
    }
}

/// A fake [AppDelegate]
#[derive(Default)]
pub struct FakeAppDelegate {
    /// Callback registered by `on_download`
    pub download_callback: Option<Box<dyn Fn() + Send>>,
    /// Callback registered by `on_cancel`
    pub cancel_callback: Option<Box<dyn Fn() + Send>>,
    /// State of delegate
    pub state: DelegateState,
    /// Queue used to simulate the main thread
    pub queue: FakeQueue,
}

/// A complete state of the UI, including its call history
#[derive(Default, serde::Serialize)]
pub struct DelegateState {
    pub status_text: String,
    pub download_text: String,
    pub download_button_visible: bool,
    pub cancel_button_visible: bool,
    pub cancel_button_enabled: bool,
    pub download_button_enabled: bool,
    pub download_progress: u32,
    pub download_progress_visible: bool,
    pub beta_text_visible: bool,
    pub quit: bool,
    /// Record of method calls.
    pub call_log: Vec<String>,
}

impl AppDelegate for FakeAppDelegate {
    type Queue = FakeQueue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.state.call_log.push("on_download".into());
        self.download_callback = Some(Box::new(callback));
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.state.call_log.push("on_cancel".into());
        self.cancel_callback = Some(Box::new(callback));
    }

    fn set_status_text(&mut self, text: &str) {
        self.state
            .call_log
            .push(format!("set_status_text: {}", text));
        self.state.status_text = text.to_owned();
    }

    fn set_download_text(&mut self, text: &str) {
        self.state
            .call_log
            .push(format!("set_download_text: {}", text));
        self.state.download_text = text.to_owned();
    }

    fn show_download_progress(&mut self) {
        self.state.call_log.push("show_download_progress".into());
        self.state.download_progress_visible = true;
    }

    fn hide_download_progress(&mut self) {
        self.state.call_log.push("hide_download_progress".into());
        self.state.download_progress_visible = false;
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.state
            .call_log
            .push(format!("set_download_progress: {}", complete));
        self.state.download_progress = complete;
    }

    fn show_download_button(&mut self) {
        self.state.call_log.push("show_download_button".into());
        self.state.download_button_visible = true;
    }

    fn hide_download_button(&mut self) {
        self.state.call_log.push("hide_download_button".into());
        self.state.download_button_visible = false;
    }

    fn enable_download_button(&mut self) {
        self.state.call_log.push("enable_download_button".into());
        self.state.download_button_enabled = true;
    }

    fn disable_download_button(&mut self) {
        self.state.call_log.push("disable_download_button".into());
        self.state.download_button_enabled = false;
    }

    fn show_cancel_button(&mut self) {
        self.state.call_log.push("show_cancel_button".into());
        self.state.cancel_button_visible = true;
    }

    fn hide_cancel_button(&mut self) {
        self.state.call_log.push("hide_cancel_button".into());
        self.state.cancel_button_visible = false;
    }

    fn enable_cancel_button(&mut self) {
        self.state.call_log.push("enable_cancel_button".into());
        self.state.cancel_button_enabled = true;
    }

    fn disable_cancel_button(&mut self) {
        self.state.call_log.push("disable_cancel_button".into());
        self.state.cancel_button_enabled = false;
    }

    fn show_beta_text(&mut self) {
        self.state.call_log.push("show_beta_text".into());
        self.state.beta_text_visible = true;
    }

    fn hide_beta_text(&mut self) {
        self.state.call_log.push("hide_beta_text".into());
        self.state.beta_text_visible = false;
    }

    fn quit(&mut self) {
        self.state.call_log.push("quit".into());
        self.state.quit = true;
    }

    fn queue(&self) -> Self::Queue {
        self.queue.clone()
    }
}

/// Test that the flow starts by fetching app version data
#[tokio::test(start_paused = true)]
async fn test_fetch_version() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<
        _,
        FakeAppDownloaderHappyPath,
        FakeVersionInfoProvider,
        FakeDirectoryProvider<true>,
    >(&mut delegate);

    // The app should start out by fetching the current app version
    assert_yaml_snapshot!(delegate.state);

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run UI updates to display the fetched version
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // The download button and current version should be displayed
    assert_yaml_snapshot!(delegate.state);
}

/// Test that the on_download callback gets registered and, when invoked,
/// properly updates the UI.
#[tokio::test(start_paused = true)]
async fn test_download() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<
        _,
        FakeAppDownloaderHappyPath,
        FakeVersionInfoProvider,
        FakeDirectoryProvider<true>,
    >(&mut delegate);

    // Wait for the version info
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // The download button should be available
    assert_yaml_snapshot!(delegate.state);

    // Initiate download
    let cb = delegate
        .download_callback
        .take()
        .expect("no download callback registered");
    cb();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run queued actions
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // We should see download progress, and cancellation
    assert_yaml_snapshot!(delegate.state);

    // Wait for download
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // Everything including verification should have succeeded
    // Downloader should have quit
    assert_yaml_snapshot!(delegate.state);
}

/// Test that the install aborts if verification fails
#[tokio::test(start_paused = true)]
async fn test_failed_verification() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<
        _,
        FakeAppDownloaderVerifyFail,
        FakeVersionInfoProvider,
        FakeDirectoryProvider<true>,
    >(&mut delegate);

    // Wait for the version info
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // Initiate download
    let cb = delegate
        .download_callback
        .take()
        .expect("no download callback registered");
    cb();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Wait for queued actions to complete
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // Verification failed
    assert_yaml_snapshot!(delegate.state);
}

/// Test failing to create the download directory
#[tokio::test(start_paused = true)]
async fn test_failed_directory_creation() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<
        _,
        FakeAppDownloaderHappyPath,
        FakeVersionInfoProvider,
        FakeDirectoryProvider<false>,
    >(&mut delegate);

    // Wait for the version info
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // Initiate download
    let cb = delegate
        .download_callback
        .take()
        .expect("no download callback registered");
    cb();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Wait for queued actions to complete
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // Verification failed
    assert_yaml_snapshot!(delegate.state);
}
