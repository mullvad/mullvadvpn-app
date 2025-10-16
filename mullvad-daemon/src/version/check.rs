use futures::{
    FutureExt, StreamExt, TryFutureExt,
    channel::mpsc,
    future::{BoxFuture, FusedFuture},
};
use mullvad_api::{
    availability::ApiAvailability, rest::MullvadRestHandle, version::AppVersionProxy,
};
use mullvad_update::version::{Rollout, VersionInfo};
use mullvad_version::Version;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    str::FromStr,
    sync::LazyLock,
    time::{Duration, SystemTime},
};
use talpid_core::mpsc::Sender;
use talpid_future::retry::{ConstantInterval, retry_future};
use talpid_types::ErrorExt;

use super::Error;

const VERSION_INFO_FILENAME: &str = "version-info.json";

static APP_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::from_str(mullvad_version::VERSION).unwrap());
static CHECK_ENABLED: LazyLock<bool> = LazyLock::new(|| {
    !APP_VERSION.is_dev()
        || std::env::var("MULLVAD_ENABLE_DEV_UPDATES")
            .map(|v| v != "0")
            .unwrap_or(false)
});

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// How long to wait before making the first version check after starting.
/// After this one, we wait [UPDATE_INTERVAL] between checks.
const FIRST_CHECK_INTERVAL: Duration = Duration::from_secs(5);
/// How long to wait between version checks, regardless of whether they succeed
#[cfg(not(target_os = "android"))]
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);
/// How long to wait between version checks, regardless of whether they succeed
// On Android, be more conservative since we use old endpoint. Retry at most once per 6 hours.
#[cfg(target_os = "android")]
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 6);
/// Wait this long before sending platform metadata in check
/// `M-Platform-Version` should only be sent once per 24h to make statistics predictable.
const PLATFORM_HEADER_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);
/// Retry strategy for `GetVersionInfo`.
const IMMEDIATE_RETRY_STRATEGY: ConstantInterval = ConstantInterval::new(Duration::ZERO, Some(3));

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(target_os = "android")]
const PLATFORM: &str = "android";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct VersionCache {
    /// Version used for the [VersionCache]. This is needed to ensure that
    /// `current_version_supported` refers to the installed app.
    pub cache_version: mullvad_version::Version,
    /// Whether the current (installed) version is supported or an upgrade is required
    pub current_version_supported: bool,
    /// The latest available versions
    pub version_info: mullvad_update::version::VersionInfo,
    /// When we last checked with platform headers
    pub last_platform_header_check: SystemTime,
    #[cfg(not(target_os = "android"))]
    pub metadata_version: usize,
    /// HTTP ETag associated with this metadata
    pub etag: Option<String>,
}

pub(crate) struct VersionUpdater(());

#[derive(Default)]
struct VersionUpdaterInner {
    /// The last known [AppVersionInfo]
    last_app_version_info: Option<VersionCache>,
}

impl VersionUpdater {
    pub(super) async fn spawn(
        mut api_handle: MullvadRestHandle,
        availability_handle: ApiAvailability,
        cache_dir: PathBuf,
        update_sender: mpsc::UnboundedSender<VersionCache>,
        refresh_rx: mpsc::UnboundedReceiver<()>,
        rollout: Rollout,
    ) {
        // load the last known AppVersionInfo from cache
        let last_app_version_info = load_cache(&cache_dir).await;

        api_handle.factory = api_handle.factory.default_timeout(DOWNLOAD_TIMEOUT);
        let version_proxy = AppVersionProxy::new(api_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        let platform_version = talpid_platform_metadata::short_version();

        tokio::spawn(
            VersionUpdaterInner {
                last_app_version_info,
            }
            .run(
                refresh_rx,
                UpdateContext {
                    cache_path,
                    update_sender,
                },
                ApiContext {
                    api_handle: availability_handle,
                    version_proxy,
                    platform_version,
                },
                rollout,
            ),
        );
    }
}

impl VersionUpdaterInner {
    #[cfg(not(target_os = "android"))]
    pub fn get_min_metadata_version(&self) -> usize {
        self.last_app_version_info
            .as_ref()
            // Reject version responses with a lower metadata version
            // than the newest version we know about. This is
            // important to prevent downgrade attacks.
            .map(|info| info.metadata_version)
            .unwrap_or(mullvad_update::version::MIN_VERIFY_METADATA_VERSION)
    }

    #[cfg(target_os = "android")]
    pub fn get_min_metadata_version(&self) -> usize {
        mullvad_update::version::MIN_VERIFY_METADATA_VERSION
    }

    /// Update [Self::last_app_version_info] and write it to disk cache, and notify the `update`
    /// callback.
    #[allow(unused_mut)]
    async fn update_version_info(
        &mut self,
        update: &impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        mut new_version_info: VersionCache,
    ) {
        #[cfg(not(target_os = "android"))]
        if let Some(current_cache) = self.last_app_version_info.as_ref() {
            if current_cache.metadata_version == new_version_info.metadata_version {
                log::trace!("Ignoring version info with same metadata version");
                new_version_info = current_cache.clone();
            }
        }

        if let Err(err) = update(new_version_info.clone()).await {
            log::error!("Failed to save version cache to disk: {}", err);
        }
        self.last_app_version_info = Some(new_version_info);
    }

    /// Return when the last successful check including platform headers was made.
    ///
    /// This should occur every [PLATFORM_HEADER_INTERVAL].
    fn last_platform_check(&self) -> Option<SystemTime> {
        self.last_app_version_info
            .as_ref()
            .map(|info| info.last_platform_header_check)
    }

    /// Return the last etag received from the server
    fn etag(&self) -> Option<&str> {
        self.last_app_version_info
            .as_ref()
            .and_then(|info| info.etag.as_deref())
    }

    /// Return a future that resolves after [UPDATE_INTERVAL].
    fn update_interval() -> Pin<Box<impl FusedFuture<Output = ()> + use<>>> {
        // Boxed, pinned, and fused.
        // Alternate title: "We don't want to deal with the borrow checker."
        Box::pin(talpid_time::sleep(UPDATE_INTERVAL).fuse())
    }

    async fn run(
        self,
        mut refresh_rx: mpsc::UnboundedReceiver<()>,
        update: UpdateContext,
        api: ApiContext,
        rollout: Rollout,
    ) {
        // If this is a dev build, there's no need to pester the API for version checks.
        if !*CHECK_ENABLED {
            log::warn!(
                "Not checking for updates because this is a development build and MULLVAD_ENABLE_DEV_UPDATES is not set"
            );
            while let Some(()) = refresh_rx.next().await {}
            return;
        }

        let update = |info| Box::pin(update.update(info)) as BoxFuture<'static, _>;
        let do_version_check = |min_metadata_version, last_platform_check, etag| {
            do_version_check(
                api.clone(),
                min_metadata_version,
                last_platform_check,
                rollout,
                etag,
            )
        };
        let do_version_check_in_background = |min_metadata_version, last_platform_check, etag| {
            do_version_check_in_background(
                api.clone(),
                min_metadata_version,
                last_platform_check,
                rollout,
                etag,
            )
        };

        self.run_inner(
            refresh_rx,
            update,
            do_version_check,
            do_version_check_in_background,
        )
        .await
    }

    async fn run_inner(
        mut self,
        mut refresh_rx: mpsc::UnboundedReceiver<()>,
        update: impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        do_version_check: impl Fn(
            usize,
            Option<SystemTime>,
            Option<String>,
        ) -> BoxFuture<'static, Result<Option<VersionCache>, Error>>,
        do_version_check_in_background: impl Fn(
            usize,
            Option<SystemTime>,
            Option<String>,
        ) -> BoxFuture<
            'static,
            Result<Option<VersionCache>, Error>,
        >,
    ) {
        let mut run_next_check_bg: Pin<Box<dyn FusedFuture<Output = ()> + Send>> =
            Box::pin(talpid_time::sleep(FIRST_CHECK_INTERVAL).fuse());
        let mut version_check_fg = futures::future::Fuse::terminated();
        let mut version_check_bg = futures::future::Fuse::terminated();

        loop {
            futures::select! {
                command = refresh_rx.next() => match command {
                    Some(()) => {
                        if !version_check_fg.is_terminated() {
                            // Check already running
                            continue;
                        }

                        // On Android, avoid polling the API unless necessary as we're using the old endpoint
                        // Only poll when bg check runs
                        if cfg!(target_os = "android") && let Some(info) = self.last_app_version_info.as_ref() {
                            log::trace!("Skipping version check on Android");
                            self.update_version_info(&update, info.clone()).await;
                            continue;
                        }

                        version_check_fg = do_version_check(self.get_min_metadata_version(), self.last_platform_check(), self.etag().map(str::to_string)).fuse();
                    }
                    None => {
                        break;
                    }
                },

                _ = run_next_check_bg => {
                    // On Android, avoid polling the API unless necessary as we're using the old endpoint
                    // Only poll when collecting platform headers
                    if cfg!(target_os = "android") && !should_include_platform_headers(self.last_platform_check()) {
                        log::trace!("Skipping version check on Android");
                        run_next_check_bg = Self::update_interval();
                        continue;
                    }

                    version_check_bg = do_version_check_in_background(self.get_min_metadata_version(), self.last_platform_check(), self.etag().map(str::to_string)).fuse();
                },

                response = version_check_bg => {
                    self.handle_version_response(&update, response).await;
                    run_next_check_bg = Self::update_interval();
                },
                response = version_check_fg => self.handle_version_response(&update, response).await,
            }
        }
    }

    async fn handle_version_response(
        &mut self,
        update: &impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        response: Result<Option<VersionCache>, Error>,
    ) {
        let version_info = match response {
            Ok(Some(version_info)) => version_info,
            Ok(None) => {
                // Repeat the existing info, since requesters may expect a response
                log::debug!("Version data was unchanged");
                self.last_app_version_info
                    .clone()
                    .expect("have version data since we have etag")
            }
            Err(err) => {
                log::error!("Failed to fetch version info: {err:#}");
                // FIXME: HACK: `update` is broken because we cannot return a result.
                // This means foreground requests will just receive no response on error.
                // As a workaround, we repeat the last known version info, if any.
                match self.last_app_version_info.clone() {
                    Some(version_info) => version_info,
                    None => return,
                }
            }
        };
        self.update_version_info(update, version_info).await;
    }
}

/// Return whether platform headers should be returned in a version check,
/// based on the last time `time` that they were.
fn should_include_platform_headers(time: Option<SystemTime>) -> bool {
    time.and_then(|t| t.elapsed().ok())
        .map(|t| t >= PLATFORM_HEADER_INTERVAL)
        .unwrap_or(true)
}

struct UpdateContext {
    cache_path: PathBuf,
    update_sender: mpsc::UnboundedSender<VersionCache>,
}

impl UpdateContext {
    /// Write [VersionUpdaterInner::last_app_version_info], if any, to the cache file
    /// ([VERSION_INFO_FILENAME]). Also, notify `self.update_sender`
    fn update(
        &self,
        last_app_version: VersionCache,
    ) -> impl Future<Output = Result<(), Error>> + use<> {
        let _ = self.update_sender.send(last_app_version.clone());
        let cache_path = self.cache_path.clone();

        async move {
            log::debug!("Writing version check cache to {}", cache_path.display());
            let buf = serde_json::to_vec_pretty(&last_app_version).map_err(Error::Serialize)?;
            tokio::fs::write(cache_path, buf)
                .await
                .map_err(Error::WriteVersionCache)
        }
    }
}

#[derive(Clone)]
struct ApiContext {
    api_handle: ApiAvailability,
    version_proxy: AppVersionProxy,
    platform_version: String,
}

/// Immediately query the API for the latest [AppVersionInfo].
fn do_version_check(
    api: ApiContext,
    min_metadata_version: usize,
    last_platform_check: Option<SystemTime>,
    _rollout: Rollout,
    etag: Option<String>,
) -> BoxFuture<'static, Result<Option<VersionCache>, Error>> {
    let api_handle = api.api_handle.clone();

    let download_future_factory = move || {
        version_check_inner(
            &api,
            min_metadata_version,
            last_platform_check,
            #[cfg(not(target_os = "android"))]
            _rollout,
            etag.clone(),
        )
    };

    // retry immediately on network errors (unless we're offline)
    let should_retry_immediate = move |result: &Result<_, Error>| {
        !api_handle.is_offline()
            && matches!(result, Err(Error::Download(error)) if error.is_network_error())
    };

    Box::pin(retry_future(
        download_future_factory,
        should_retry_immediate,
        IMMEDIATE_RETRY_STRATEGY,
    ))
}

/// Query the API for the latest [AppVersionInfo] once, without retrying.
///
/// This function waits until background calls are enabled in
/// [ApiAvailability](mullvad_api::availability::ApiAvailability).
fn do_version_check_in_background(
    api: ApiContext,
    min_metadata_version: usize,
    last_platform_check: Option<SystemTime>,
    _rollout: Rollout,
    etag: Option<String>,
) -> BoxFuture<'static, Result<Option<VersionCache>, Error>> {
    let when_available = api.api_handle.wait_background();
    let version_cache = version_check_inner(
        &api,
        min_metadata_version,
        last_platform_check,
        #[cfg(not(target_os = "android"))]
        _rollout,
        etag,
    );
    Box::pin(async move {
        when_available.await.map_err(Error::ApiCheck)?;
        version_cache.await
    })
}

/// Fetch new version endpoint
#[cfg(not(target_os = "android"))]
fn version_check_inner(
    api: &ApiContext,
    min_metadata_version: usize,
    last_platform_check: Option<SystemTime>,
    rollout: Rollout,
    etag: Option<String>,
) -> impl Future<Output = Result<Option<VersionCache>, Error>> + use<> {
    let add_platform_headers = should_include_platform_headers(last_platform_check);

    let architecture = match talpid_platform_metadata::get_native_arch()
        .expect("IO error while getting native architecture")
        .expect("Failed to get native architecture")
    {
        talpid_platform_metadata::Architecture::X86 => mullvad_update::format::Architecture::X86,
        talpid_platform_metadata::Architecture::Arm64 => {
            mullvad_update::format::Architecture::Arm64
        }
    };
    let endpoint = api.version_proxy.version_check_2(
        PLATFORM,
        architecture,
        min_metadata_version,
        add_platform_headers.then(|| api.platform_version.clone()),
        rollout,
        etag,
    );

    async move {
        let Some(result) = endpoint.await.map_err(Error::Download)? else {
            // ETag is up to date
            return Ok(None);
        };
        let last_platform_check = if add_platform_headers {
            SystemTime::now()
        } else {
            last_platform_check.expect("must be set if not adding headers")
        };

        Ok(Some(VersionCache {
            cache_version: APP_VERSION.clone(),
            current_version_supported: result.current_version_supported,
            version_info: result.version_info,
            last_platform_header_check: last_platform_check,
            metadata_version: result.metadata_version,
            etag: result.etag,
        }))
    }
}

#[cfg(target_os = "android")]
fn version_check_inner(
    api: &ApiContext,
    // NOTE: This is unused when `update` is disabled
    _min_metadata_version: usize,
    last_platform_check: Option<SystemTime>,
    etag: Option<String>,
) -> impl Future<Output = Result<Option<VersionCache>, Error>> + use<> {
    let add_platform_headers = should_include_platform_headers(last_platform_check);

    let v1_endpoint = api.version_proxy.version_check(
        mullvad_version::VERSION.to_owned(),
        PLATFORM,
        add_platform_headers.then(|| api.platform_version.clone()),
        etag,
    );
    async move {
        let Some(response) = v1_endpoint.await.map_err(Error::Download)? else {
            // ETag is up to date
            return Ok(None);
        };
        let latest_stable = response.latest_stable()
            .and_then(|version| version.parse().ok())
            // Suggested stable must actually be stable
            .filter(|version: &mullvad_version::Version| version.pre_stable.is_none())
            .ok_or_else(|| Error::MissingStable)?;
        let latest_beta = response.latest_beta()
            .and_then(|version| version.parse().ok())
            // Suggested beta must actually be non-stable
            .filter(|version: &mullvad_version::Version| version.pre_stable.is_some());
        let last_platform_check = if add_platform_headers {
            SystemTime::now()
        } else {
            last_platform_check.expect("must be set if not adding headers")
        };

        Ok(Some(VersionCache {
            cache_version: APP_VERSION.clone(),
            current_version_supported: response.supported(),
            etag: response.etag,
            last_platform_header_check: last_platform_check,
            // Note: We're pretending that this is complete information,
            // but on Android and Linux, most of the information is missing
            version_info: VersionInfo {
                stable: mullvad_update::version::Version {
                    version: latest_stable,
                    changelog: "".to_owned(),
                    urls: vec![],
                    sha256: [0u8; 32],
                    size: 0,
                },
                beta: latest_beta.map(|version| mullvad_update::version::Version {
                    version,
                    changelog: "".to_owned(),
                    urls: vec![],
                    sha256: [0u8; 32],
                    size: 0,
                }),
            },
        }))
    }
}

/// Read the app version cache from the provided directory.
///
/// Returns the [AppVersionInfo] along with the modification time of the cache file,
/// or `None` on any error.
async fn load_cache(cache_dir: &Path) -> Option<VersionCache> {
    try_load_cache(cache_dir)
        .await
        .inspect_err(|error| {
            if matches!(error, Error::OutdatedVersion) {
                log::trace!("Ignoring outdated version cache");
            } else {
                log::warn!(
                    "{}",
                    error.display_chain_with_msg("Unable to load cached version info")
                );
            }
        })
        .ok()
}

async fn try_load_cache(cache_dir: &Path) -> Result<VersionCache, Error> {
    if !*CHECK_ENABLED {
        return Ok(dev_version_cache());
    }

    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());

    let content = tokio::fs::read_to_string(&path)
        .map_err(Error::ReadVersionCache)
        .await?;

    let cache: VersionCache = serde_json::from_str(&content).map_err(Error::Deserialize)?;

    if cache_is_stale(&cache, &APP_VERSION) {
        return Err(Error::OutdatedVersion);
    }

    Ok(cache)
}

/// Check if the cache is left over from another version of the app. If so, discard it.
fn cache_is_stale(cache: &VersionCache, current_version: &mullvad_version::Version) -> bool {
    &cache.cache_version != current_version
}

fn dev_version_cache() -> VersionCache {
    VersionCache {
        cache_version: mullvad_version::VERSION.parse().unwrap(),
        current_version_supported: false,
        version_info: VersionInfo {
            stable: mullvad_update::version::Version {
                version: mullvad_version::VERSION.parse().unwrap(),
                changelog: "".to_owned(),
                urls: vec![],
                sha256: [0u8; 32],
                size: 0,
            },
            beta: None,
        },
        last_platform_header_check: SystemTime::now(),
        #[cfg(not(target_os = "android"))]
        metadata_version: 0,
        etag: None,
    }
}

#[cfg(test)]
mod test {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use futures::SinkExt;
    use mullvad_update::version::Version;

    use super::*;

    /// Test whether mismatching version caches are ignored.
    /// This prevents old versions from being suggested as updates,
    /// and the current version from being labeled unsupported.
    #[test]
    fn test_invalid_cache() {
        assert!(!cache_is_stale(
            &version_cache("2025.5", "2025.5", None),
            &"2025.5".parse().unwrap()
        ));
        assert!(cache_is_stale(
            &version_cache("2025.5", "2025.5", None),
            &"2025.6".parse().unwrap()
        ));
        assert!(!cache_is_stale(
            &version_cache("2025.5-beta1", "2025.5", Some("2025.5-beta1")),
            &"2025.5-beta1".parse().unwrap()
        ));
        assert!(cache_is_stale(
            &version_cache("2025.5-beta1", "2025.5", Some("2025.5-beta1")),
            &"2025.5-beta2".parse().unwrap()
        ));
    }

    fn version_cache(cache_version: &str, stable: &str, beta: Option<&str>) -> VersionCache {
        VersionCache {
            cache_version: cache_version.parse().unwrap(),
            current_version_supported: false,
            version_info: VersionInfo {
                stable: Version {
                    version: stable.parse().unwrap(),
                    urls: vec![],
                    size: 0,
                    changelog: "".to_owned(),
                    sha256: [0u8; 32],
                },
                beta: beta.map(|beta| Version {
                    version: beta.parse().unwrap(),
                    urls: vec![],
                    size: 0,
                    changelog: "".to_owned(),
                    sha256: [0u8; 32],
                }),
            },
            last_platform_header_check: SystemTime::now(),
            #[cfg(not(target_os = "android"))]
            metadata_version: 0,
            etag: None,
        }
    }

    /// If there's no cached version, we should perform a check now and include platform headers
    #[test]
    fn test_version_unknown_is_stale() {
        let checker = VersionUpdaterInner::default();
        assert!(checker.last_app_version_info.is_none());
        assert!(should_include_platform_headers(
            checker.last_platform_check()
        ));
    }

    /// If the last checked time is in the future, the version is stale
    #[test]
    fn test_version_cache_in_future_is_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some(VersionCache {
                last_platform_header_check: SystemTime::now() + Duration::from_secs(1),
                ..dev_version_cache()
            }),
        };
        assert!(should_include_platform_headers(
            checker.last_platform_check()
        ));
    }

    /// If we have a cached version that's less than `PLATFORM_HEADER_INTERVAL` old, do not include platform headers
    #[test]
    fn test_version_actual_non_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some(VersionCache {
                last_platform_header_check: SystemTime::now() - PLATFORM_HEADER_INTERVAL
                    + Duration::from_secs(1),
                ..dev_version_cache()
            }),
        };
        assert!(!should_include_platform_headers(
            checker.last_platform_check()
        ));
    }

    /// If `PLATFORM_HEADER_INTERVAL` has elapsed, the check should include platform headers
    #[test]
    fn test_version_actual_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some(VersionCache {
                last_platform_header_check: SystemTime::now() - PLATFORM_HEADER_INTERVAL,
                ..dev_version_cache()
            }),
        };
        assert!(should_include_platform_headers(
            checker.last_platform_check()
        ));
    }

    /// Test whether check actually runs first after `FIRST_CHECK_INTERVAL` and then every `UPDATE_INTERVAL`
    #[tokio::test(start_paused = true)]
    async fn test_version_check_run() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some(dev_version_cache()),
        };

        let updated = Arc::new(AtomicBool::new(false));
        let update = fake_updater(updated.clone());

        let (_tx, rx) = mpsc::unbounded();
        tokio::spawn(checker.run_inner(rx, update, fake_version_check, fake_version_check));

        talpid_time::sleep(FIRST_CHECK_INTERVAL - Duration::from_millis(100)).await;
        assert!(
            !updated.load(Ordering::SeqCst),
            "no check until `FIRST_CHECK_INTERVAL` has elapsed"
        );

        talpid_time::sleep(Duration::from_millis(101)).await;
        assert!(
            updated.load(Ordering::SeqCst),
            "check when `FIRST_CHECK_INTERVAL` has elapsed"
        );

        updated.store(false, Ordering::SeqCst);

        talpid_time::sleep(Duration::from_secs(10)).await;
        assert!(
            !updated.load(Ordering::SeqCst),
            "should see no check until `UPDATE_INTERVAL` has elapsed"
        );

        talpid_time::sleep(UPDATE_INTERVAL).await;
        assert!(
            updated.load(Ordering::SeqCst),
            "check should have run after `UPDATE_INTERVAL` or more"
        );
    }

    /// Test whether check runs immediately when requested
    #[tokio::test(start_paused = true)]
    async fn test_version_check_manual() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some(VersionCache {
                last_platform_header_check: SystemTime::now() - Duration::from_secs(1),
                ..dev_version_cache()
            }),
        };

        let updated = Arc::new(AtomicBool::new(false));
        let update = fake_updater(updated.clone());

        let (mut tx, rx) = mpsc::unbounded();
        tokio::spawn(checker.run_inner(rx, update, fake_version_check, fake_version_check));

        // Automatic update should not run until `FIRST_CHECK_INTERVAL` has elapsed
        talpid_time::sleep(FIRST_CHECK_INTERVAL - Duration::from_secs(1)).await;
        assert!(
            !updated.load(Ordering::SeqCst),
            "check did not run automatically"
        );

        // Requesting version should trigger an immediate update
        send_version_request(&mut tx).await.unwrap();
        talpid_time::sleep(Duration::from_secs(1)).await;
        assert!(
            updated.load(Ordering::SeqCst),
            "expected immediate update from stale"
        );

        updated.store(false, Ordering::SeqCst);

        // The next request should trigger an update, even if the version has not changed
        send_version_request(&mut tx).await.unwrap();
        talpid_time::sleep(Duration::from_secs(1)).await;
        assert!(updated.load(Ordering::SeqCst), "expected cached version");

        // Automatic update should run again after `UPDATE_INTERVAL`
        updated.store(false, Ordering::SeqCst);
        talpid_time::sleep(UPDATE_INTERVAL - Duration::from_secs(1)).await;
        assert!(
            !updated.load(Ordering::SeqCst),
            "expected no automatic update yet"
        );
        talpid_time::sleep(Duration::from_secs(1)).await;
        assert!(
            updated.load(Ordering::SeqCst),
            "expected automatic update yet"
        );
    }

    async fn send_version_request(
        tx: &mut mpsc::UnboundedSender<()>,
    ) -> Result<(), futures::channel::mpsc::SendError> {
        tx.send(()).await?;
        Ok(())
    }

    fn fake_updater(
        updated: Arc<AtomicBool>,
    ) -> impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>> {
        move |_new_version| {
            updated.store(true, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }

    fn fake_version_check(
        _min_metadata_version: usize,
        _last_platform_check: Option<SystemTime>,
        _etag: Option<String>,
    ) -> BoxFuture<'static, Result<Option<VersionCache>, Error>> {
        Box::pin(async { Ok(Some(fake_version_response())) })
    }

    fn fake_version_response() -> VersionCache {
        // TODO: The tests pass, but check that this is a sane fake version cache anyway
        VersionCache {
            cache_version: mullvad_version::VERSION.parse().unwrap(),
            current_version_supported: true,
            version_info: VersionInfo {
                stable: Version {
                    version: "2025.5".parse::<mullvad_version::Version>().unwrap(),
                    urls: vec![],
                    size: 0,
                    changelog: "".to_owned(),
                    sha256: [0u8; 32],
                },
                beta: None,
            },
            last_platform_header_check: SystemTime::now(),
            #[cfg(not(target_os = "android"))]
            metadata_version: 0,
            etag: None,
        }
    }
}
