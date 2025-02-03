//! Tests for integrations between UI controller and other components

use installer_downloader::controller::AppController;
use installer_downloader::delegate::{AppDelegate, AppDelegateQueue, UiProgressUpdater};
use mullvad_update::api::{Version, VersionInfo, VersionInfoProvider};
use mullvad_update::app::{
    AppDownloader, AppDownloaderFactory, AppDownloaderParameters, DownloadError,
};
use mullvad_update::fetch::ProgressUpdater;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use std::vec::Vec;

pub struct FakeVersionInfoProvider {}

static FAKE_VERSION: LazyLock<VersionInfo> = LazyLock::new(|| VersionInfo {
    stable: Version {
        version: "2025.1".to_owned(),
        urls: vec!["https://mullvad.net/fakeapp".to_owned()],
        size: 1234,
        signature_urls: vec!["https://mullvad.net/fakesig".to_owned()],
    },
    beta: None,
});

#[async_trait::async_trait]
impl VersionInfoProvider for FakeVersionInfoProvider {
    async fn get_version_info() -> anyhow::Result<VersionInfo> {
        Ok(FAKE_VERSION.clone())
    }
}

/// Creates an app downloader based on a few parameters:
/// * SigSucceed - whether fetching the signature succeeds
/// * ExeSucceed - whether fetching the app succeeds
/// * VerifySucceed - whether verifying the signature succeeds
pub struct FakeAppDownloaderFactory<
    const SIG_SUCCEED: bool,
    const EXE_SUCCEED: bool,
    const VERIFY_SUCCEED: bool,
> {}

/// Downloader for which all steps immediately succeed
pub type FakeAppDownloaderFactoryHappyPath = FakeAppDownloaderFactory<true, true, true>;

/// Downloader for which all but the final verification step succeed
pub type FakeAppDownloaderFactoryVerifyFail = FakeAppDownloaderFactory<true, true, false>;

impl<const SIG_SUCCEED: bool, const EXE_SUCCEED: bool, const VERIFY_SUCCEED: bool>
    AppDownloaderFactory for FakeAppDownloaderFactory<SIG_SUCCEED, EXE_SUCCEED, VERIFY_SUCCEED>
{
    type Downloader = FakeAppDownloader<Self::SigProgress, Self::AppProgress>;
    type SigProgress = UiProgressUpdater<FakeAppDelegate>;
    type AppProgress = UiProgressUpdater<FakeAppDelegate>;

    fn new_downloader(
        params: AppDownloaderParameters<Self::SigProgress, Self::AppProgress>,
    ) -> Self::Downloader {
        FakeAppDownloader {
            params,
            sig_succeed: SIG_SUCCEED,
            exe_succeed: EXE_SUCCEED,
            verify_succeed: VERIFY_SUCCEED,
        }
    }
}

pub struct FakeAppDownloader<SigProgress, AppProgress> {
    params: AppDownloaderParameters<SigProgress, AppProgress>,
    sig_succeed: bool,
    exe_succeed: bool,
    verify_succeed: bool,
}

#[async_trait::async_trait]
impl<SigProgress: ProgressUpdater, AppProgress: ProgressUpdater> AppDownloader
    for FakeAppDownloader<SigProgress, AppProgress>
{
    async fn download_signature(&mut self) -> Result<(), DownloadError> {
        if self.sig_succeed {
            self.params.sig_progress.set_url(&self.params.signature_url);
            self.params.sig_progress.set_progress(1.);
            Ok(())
        } else {
            Err(DownloadError::FetchSignature(anyhow::anyhow!(
                "fetching signature failed"
            )))
        }
    }

    async fn download_executable(&mut self) -> Result<(), DownloadError> {
        if self.exe_succeed {
            self.params.app_progress.set_url(&self.params.app_url);
            self.params.app_progress.set_progress(1.);
            Ok(())
        } else {
            Err(DownloadError::FetchApp(anyhow::anyhow!(
                "fetching app failed"
            )))
        }
    }

    async fn verify(&mut self) -> Result<(), DownloadError> {
        if self.verify_succeed {
            Ok(())
        } else {
            Err(DownloadError::Verification(anyhow::anyhow!(
                "verification failed"
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
    pub status_text: String,
    pub download_button_visible: bool,
    pub cancel_button_visible: bool,
    pub download_button_enabled: bool,
    pub download_progress: u32,
    pub download_progress_visible: bool,
    /// Callback registered by `on_download`
    pub download_callback: Option<Box<dyn Fn() + Send>>,
    /// Callback registered by `on_cancel`
    pub cancel_callback: Option<Box<dyn Fn() + Send>>,
    /// Record of method calls.
    pub call_log: Vec<String>,
    /// Queue used to simulate the main thread.
    pub queue: FakeQueue,
}

impl AppDelegate for FakeAppDelegate {
    type Queue = FakeQueue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.call_log.push("on_download".into());
        self.download_callback = Some(Box::new(callback));
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.call_log.push("on_cancel".into());
        self.cancel_callback = Some(Box::new(callback));
    }

    fn set_status_text(&mut self, text: &str) {
        self.call_log.push(format!("set_status_text: {}", text));
        self.status_text = text.to_owned();
    }

    fn show_download_progress(&mut self) {
        self.call_log.push("show_download_progress".into());
        self.download_progress_visible = true;
    }

    fn hide_download_progress(&mut self) {
        self.call_log.push("hide_download_progress".into());
        self.download_progress_visible = false;
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.call_log
            .push(format!("set_download_progress: {}", complete));
        self.download_progress = complete;
    }

    fn show_download_button(&mut self) {
        self.call_log.push("show_download_button".into());
        self.download_button_visible = true;
    }

    fn hide_download_button(&mut self) {
        self.call_log.push("hide_download_button".into());
        self.download_button_visible = false;
    }

    fn enable_download_button(&mut self) {
        self.call_log.push("enable_download_button".into());
        self.download_button_enabled = true;
    }

    fn disable_download_button(&mut self) {
        self.call_log.push("disable_download_button".into());
        self.download_button_enabled = false;
    }

    fn show_cancel_button(&mut self) {
        self.call_log.push("show_cancel_button".into());
        self.cancel_button_visible = true;
    }

    fn hide_cancel_button(&mut self) {
        self.call_log.push("hide_cancel_button".into());
        self.cancel_button_visible = false;
    }

    fn queue(&self) -> Self::Queue {
        self.queue.clone()
    }
}

/// Test that the flow starts by fetching app version data
#[tokio::test(start_paused = true)]
async fn test_fetch_version() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<_, FakeAppDownloaderFactoryHappyPath, FakeVersionInfoProvider>(
        &mut delegate,
    );

    // The app should start out by fetching the current app version
    assert_eq!(delegate.status_text, "Fetching app version...");
    assert!(!delegate.download_button_visible);
    assert!(!delegate.cancel_button_visible);
    assert!(!delegate.download_progress_visible);

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run UI updates to display the fetched version
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // The download button and current version should be displayed
    assert_eq!(
        delegate.status_text,
        format!("Latest version: {}", FAKE_VERSION.stable.version)
    );
    assert!(delegate.download_button_visible);
}

/// Test that the on_download callback gets registered and, when invoked,
/// properly updates the UI.
#[tokio::test(start_paused = true)]
async fn test_download() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<_, FakeAppDownloaderFactoryHappyPath, FakeVersionInfoProvider>(
        &mut delegate,
    );

    // Wait for the version info
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    assert!(delegate.download_button_visible);

    delegate.call_log.clear();

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
    delegate.call_log.clear();

    assert!(!delegate.download_button_visible);
    assert!(delegate.cancel_button_visible);
    assert!(delegate.download_progress_visible);

    // Wait for download
    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    assert_eq!(
        &delegate.call_log,
        &[
            // Download signature
            "set_download_progress: 100",
            "set_status_text: Downloading from mullvad.net... (100%)",
            // Download app
            "set_download_progress: 100",
            "set_status_text: Downloading from mullvad.net... (100%)",
            // Verification
            "set_status_text: Download complete! Verifying signature...",
            "hide_cancel_button",
            "set_status_text: Verification complete!",
        ]
    );
}

/// Test that the install aborts if verification fails
#[tokio::test(start_paused = true)]
async fn test_failed_verification() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<_, FakeAppDownloaderFactoryVerifyFail, FakeVersionInfoProvider>(
        &mut delegate,
    );

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
    delegate.call_log.clear();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    assert_eq!(delegate.status_text, "ERROR: Verification failed!");
}
