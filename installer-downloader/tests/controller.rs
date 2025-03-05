#![cfg(any(target_os = "windows", target_os = "macos"))]

//! Tests for integrations between UI controller and other components
//!
//! The tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
//! then most likely the snapshots need to be updated. The most convenient way to review
//! changes to, and update, snapshots is by running `cargo insta review`.

use insta::assert_yaml_snapshot;
use installer_downloader::controller::AppController;
use mock::{
    FakeAppDelegate, FakeAppDownloaderHappyPath, FakeAppDownloaderVerifyFail,
    FakeDirectoryProvider, FakeVersionInfoProvider, FAKE_ENVIRONMENT,
};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

mod mock;

/// Test that the flow starts by fetching app version data
#[tokio::test(start_paused = true)]
async fn test_fetch_version() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<_, FakeAppDownloaderHappyPath, _, FakeDirectoryProvider<true>>(
        &mut delegate,
        FakeVersionInfoProvider::default(),
        FAKE_ENVIRONMENT,
    );

    // The app should start out by fetching the current app version
    assert_yaml_snapshot!(delegate.state);

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run UI updates to display the fetched version
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    tokio::time::sleep(Duration::from_secs(1)).await;

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
    AppController::initialize::<_, FakeAppDownloaderHappyPath, _, FakeDirectoryProvider<true>>(
        &mut delegate,
        FakeVersionInfoProvider::default(),
        FAKE_ENVIRONMENT,
    );

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

/// Test that the flow of retrying the version fetch after a failure
#[tokio::test(start_paused = true)]
async fn test_failed_fetch_version() {
    let mut delegate = FakeAppDelegate::default();
    let fail_fetching = Arc::new(AtomicBool::new(true));
    AppController::initialize::<_, FakeAppDownloaderHappyPath, _, FakeDirectoryProvider<true>>(
        &mut delegate,
        FakeVersionInfoProvider {
            fail_fetching: fail_fetching.clone(),
        },
        FAKE_ENVIRONMENT,
    );

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run UI updates to display the fetched version
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    tokio::time::sleep(Duration::from_secs(1)).await;
    queue.run_callbacks(&mut delegate);

    // The fetch version failure screen with a retry and cancel button should be displayed
    assert_yaml_snapshot!(delegate.state);

    fail_fetching.store(false, std::sync::atomic::Ordering::SeqCst);

    // Retry fetching the version
    let cb = delegate
        .error_retry_callback
        .take()
        .expect("no retry callback registered");
    cb();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run UI updates to display the fetched version
    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);
    tokio::time::sleep(Duration::from_secs(1)).await;
    queue.run_callbacks(&mut delegate);

    // The download button and current version should be displayed
    assert_yaml_snapshot!(delegate.state);
}

/// Test that the install aborts if verification fails
#[tokio::test(start_paused = true)]
async fn test_failed_verification() {
    let mut delegate = FakeAppDelegate::default();
    AppController::initialize::<_, FakeAppDownloaderVerifyFail, _, FakeDirectoryProvider<true>>(
        &mut delegate,
        FakeVersionInfoProvider::default(),
        FAKE_ENVIRONMENT,
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
    AppController::initialize::<_, FakeAppDownloaderHappyPath, _, FakeDirectoryProvider<false>>(
        &mut delegate,
        FakeVersionInfoProvider::default(),
        FAKE_ENVIRONMENT,
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

    tokio::time::sleep(Duration::from_secs(1)).await;

    let queue = delegate.queue.clone();
    queue.run_callbacks(&mut delegate);

    // "Download failed"
    assert_yaml_snapshot!(delegate.state);
}
