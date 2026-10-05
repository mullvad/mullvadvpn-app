use futures::{
    FutureExt, StreamExt, TryFutureExt,
    channel::mpsc,
    future::{BoxFuture, FusedFuture},
};
use mullvad_api::{
    availability::ApiAvailability, rest::MullvadRestHandle, version::AppVersionProxy,
};
#[cfg(not(target_os = "android"))]
use mullvad_update::version::rollout::Rollout;
#[cfg(not(target_os = "android"))]
use mullvad_update::version::{MIN_VERIFY_METADATA_VERSION, Metadata, VersionInfo};
use mullvad_version::Version;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    sync::LazyLock,
    time::{Duration, SystemTime},
};
use talpid_core::mpsc::Sender;
use talpid_error::ErrorExt;
use talpid_future::retry::{ConstantInterval, retry_future};

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
const UPDATE_INTERVAL: Duration = Duration::from_hours(1);

/// Wait this long before sending platform metadata in check
/// `M-Platform-Version` should only be sent once per 24h to make statistics predictable.
const PLATFORM_HEADER_INTERVAL: Duration = Duration::from_hours(24);
/// Retry strategy for `GetVersionInfo`.
const IMMEDIATE_RETRY_STRATEGY: ConstantInterval = ConstantInterval::new(Duration::ZERO, Some(3));

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct VersionCache {
    /// Version used for the [VersionCache]. This is needed to ensure that
    /// `current_version_supported` refers to the installed app.
    pub cache_version: Version,
    /// Whether the current (installed) version is supported or an upgrade is required
    pub current_version_supported: bool,
    #[cfg(not(target_os = "android"))]
    /// The latest available versions
    pub version_info: VersionInfo,
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
    /// The last known [VersionCache]
    last_app_version_info: Option<VersionCache>,
}

/// Initial update state recovered from a cache written by any app version.
struct VersionCheckState {
    last_app_version_info: Option<VersionCache>,
    #[cfg(not(target_os = "android"))]
    min_metadata_version: usize,
}

impl VersionCheckState {
    fn from_cache(cache: Option<VersionCache>) -> Self {
        #[cfg(not(target_os = "android"))]
        let min_metadata_version = min_metadata_version(cache.as_ref());
        let last_app_version_info = cache.filter(|cache| {
            let stale = cache_is_stale(cache, &APP_VERSION);
            if stale {
                log::trace!("Ignoring outdated version cache");
            }
            !stale
        });
        Self {
            last_app_version_info,
            #[cfg(not(target_os = "android"))]
            min_metadata_version,
        }
    }
}

impl VersionUpdater {
    pub(super) async fn spawn(
        mut api_handle: MullvadRestHandle,
        availability_handle: ApiAvailability,
        cache_dir: PathBuf,
        update_sender: mpsc::UnboundedSender<VersionCache>,
        refresh_rx: mpsc::UnboundedReceiver<()>,
        #[cfg(not(target_os = "android"))] rollout: Rollout,
    ) {
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        // load the last known AppVersionInfo from cache
        let VersionCheckState {
            last_app_version_info,
            #[cfg(not(target_os = "android"))]
            min_metadata_version,
        } = VersionCheckState::from_cache(load_cache(&cache_path).await);

        api_handle.set_default_timeout(DOWNLOAD_TIMEOUT);
        let version_proxy = AppVersionProxy::new(api_handle);
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
                    #[cfg(not(target_os = "android"))]
                    rollout,
                    #[cfg(not(target_os = "android"))]
                    min_metadata_version,
                },
            ),
        );
    }
}

impl VersionUpdaterInner {
    /// Update [Self::last_app_version_info] and write it to disk cache, and notify the `update`
    /// callback.
    #[cfg_attr(target_os = "android", expect(unused_mut))]
    async fn update_version_info(
        &mut self,
        update: &impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        mut new_version_info: VersionCache,
    ) {
        #[cfg(not(target_os = "android"))]
        {
            new_version_info = self.ignore_cache_if_same_version(new_version_info);
        }

        if let Err(err) = update(new_version_info.clone()).await {
            log::error!("Failed to save version cache to disk: {}", err);
        }
        self.last_app_version_info = Some(new_version_info);
    }

    #[cfg(not(target_os = "android"))]
    fn ignore_cache_if_same_version(&self, mut new_version_info: VersionCache) -> VersionCache {
        if let Some(current_cache) = self.last_app_version_info.as_ref()
            && current_cache.metadata_version == new_version_info.metadata_version
        {
            log::trace!("Ignoring version info with same metadata version");
            // Ignore everything except etag and platform timestamp
            new_version_info = VersionCache {
                last_platform_header_check: new_version_info.last_platform_header_check,
                etag: new_version_info.etag,
                ..current_cache.clone()
            };
        }
        new_version_info
    }

    /// Return when the last successful check including platform headers was made.
    ///
    /// This should occur every [PLATFORM_HEADER_INTERVAL].
    #[cfg(test)]
    fn last_platform_check(&self) -> Option<SystemTime> {
        self.last_app_version_info
            .as_ref()
            .map(|info| info.last_platform_header_check)
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
    ) {
        // If this is a dev build, there's no need to pester the API for version checks.
        if !*CHECK_ENABLED {
            log::warn!(
                "Not checking for updates because this is a development build and MULLVAD_ENABLE_DEV_UPDATES is not set"
            );
            // Send the initial dev cache so the router has a version immediately
            let _ = update.update_sender.send(dev_version_cache());
            // Respond to every refresh request with the same dev cache
            while let Some(()) = refresh_rx.next().await {
                let _ = update.update_sender.send(dev_version_cache());
            }
            return;
        }

        let update = |info| Box::pin(update.update(info)) as BoxFuture<'static, _>;
        let do_version_check = |prev_cache| do_version_check(api.clone(), prev_cache);
        let do_version_check_in_background =
            |prev_cache| do_version_check_in_background(api.clone(), prev_cache);

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
            Option<VersionCache>,
        ) -> BoxFuture<'static, Result<VersionCache, Error>>,
        do_version_check_in_background: impl Fn(
            Option<VersionCache>,
        )
            -> BoxFuture<'static, Result<VersionCache, Error>>,
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
                        version_check_fg = do_version_check(self.last_app_version_info.clone()).fuse();
                    }
                    None => {
                        break;
                    }
                },

                _ = run_next_check_bg => {
                    version_check_bg = do_version_check_in_background(self.last_app_version_info.clone()).fuse();
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
        response: Result<VersionCache, Error>,
    ) {
        let version_info = match response {
            Ok(version_info) => version_info,
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
            log::trace!("Writing version check cache to {}", cache_path.display());
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
    #[cfg(not(target_os = "android"))]
    rollout: Rollout,
    /// Lowest metadata version to accept when there is no cache for this app version
    #[cfg(not(target_os = "android"))]
    min_metadata_version: usize,
}

/// Immediately query the API for the latest [VersionCache].
fn do_version_check(
    api: ApiContext,
    prev_cache: Option<VersionCache>,
) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let api_handle = api.api_handle.clone();

    let download_future_factory = move || version_check_inner(api.clone(), prev_cache.clone());

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

/// Query the API for the latest [VersionCache] once, without retrying.
///
/// This function waits until background calls are enabled in [ApiAvailability].
fn do_version_check_in_background(
    api: ApiContext,
    cache: Option<VersionCache>,
) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let when_available = api.api_handle.wait_background();
    let version_cache = version_check_inner(api, cache);
    Box::pin(async move {
        when_available.await.map_err(Error::ApiCheck)?;
        version_cache.await
    })
}

#[cfg(not(target_os = "android"))]
/// Fetch new version endpoint
async fn version_check_inner(
    api: ApiContext,
    cache: Option<VersionCache>,
) -> Result<VersionCache, Error> {
    let architecture = match talpid_platform_metadata::get_native_arch()
        .expect("IO error while getting native architecture")
        .expect("Failed to get native architecture")
    {
        talpid_platform_metadata::Architecture::X86 => mullvad_update::format::Architecture::X86,
        talpid_platform_metadata::Architecture::Arm64 => {
            mullvad_update::format::Architecture::Arm64
        }
    };

    version_check_with(
        cache,
        api.min_metadata_version,
        api.platform_version,
        move |minimum, platform_version, etag| {
            api.version_proxy.version_check(
                PLATFORM,
                architecture,
                minimum,
                platform_version,
                api.rollout,
                etag,
            )
        },
    )
    .await
}

/// Apply cache and platform-header policy around a version API request.
#[cfg(not(target_os = "android"))]
async fn version_check_with<F>(
    cache: Option<VersionCache>,
    min_metadata_version: usize,
    platform_version: String,
    version_check: impl FnOnce(usize, Option<String>, Option<String>) -> F,
) -> Result<VersionCache, Error>
where
    F: Future<
        Output = Result<Option<mullvad_api::version::AppVersionResponse>, mullvad_api::rest::Error>,
    >,
{
    let (response, last_platform_header_check) = match cache {
        // Cache available
        Some(prev_cache) => {
            let add_platform_headers =
                should_include_platform_headers(Some(prev_cache.last_platform_header_check));
            let get_last_platform_header_check = || {
                if add_platform_headers {
                    SystemTime::now()
                } else {
                    prev_cache.last_platform_header_check
                }
            };

            let Some(response) = version_check(
                prev_cache.metadata_version,
                add_platform_headers.then_some(platform_version),
                prev_cache.etag.clone(),
            )
            .await
            .map_err(Error::Download)?
            else {
                // ETag is up to date
                log::trace!("Version data unchanged");
                return Ok(VersionCache {
                    last_platform_header_check: get_last_platform_header_check(),
                    ..prev_cache
                });
            };
            (response, get_last_platform_header_check())
        }
        // No cache available
        None => {
            let response = version_check(min_metadata_version, Some(platform_version), None)
                .await
                .map_err(Error::Download)?
                .expect("function must return body if no etag was set");
            (response, SystemTime::now())
        }
    };

    Ok(VersionCache {
        cache_version: APP_VERSION.clone(),
        current_version_supported: response.current_version_supported,
        version_info: response.version_info,
        last_platform_header_check,
        metadata_version: response.metadata_version,
        etag: response.etag,
    })
}

#[cfg(target_os = "android")]
/// Fetch new version endpoint
async fn version_check_inner(
    api: ApiContext,
    cache: Option<VersionCache>,
) -> Result<VersionCache, Error> {
    let (response, last_platform_header_check) = match cache {
        // Cache available
        Some(prev_cache) => {
            let add_platform_headers =
                should_include_platform_headers(Some(prev_cache.last_platform_header_check));
            let get_last_platform_header_check = || {
                if add_platform_headers {
                    SystemTime::now()
                } else {
                    prev_cache.last_platform_header_check
                }
            };

            let Some(response) = api
                .version_proxy
                .version_check_android(
                    add_platform_headers.then(|| api.platform_version.clone()),
                    prev_cache.etag.clone(),
                )
                .await
                .map_err(Error::Download)?
            else {
                // ETag is up to date
                log::trace!("Version data unchanged");
                return Ok(VersionCache {
                    last_platform_header_check: get_last_platform_header_check(),
                    ..prev_cache
                });
            };
            (response, get_last_platform_header_check())
        }
        // No cache available
        None => {
            let response = api
                .version_proxy
                .version_check_android(Some(api.platform_version), None)
                .await
                .map_err(Error::Download)?
                .expect("function must return body if no etag was set");
            (response, SystemTime::now())
        }
    };

    Ok(VersionCache {
        cache_version: APP_VERSION.clone(),
        current_version_supported: response.current_version_supported,
        last_platform_header_check,
        etag: response.etag,
    })
}

/// Read the app version cache from the provided directory.
///
/// Returns the [VersionCache], or `None` on any error. The cache may have been written by
/// another app version, see [cache_is_stale].
async fn load_cache(cache_path: &PathBuf) -> Option<VersionCache> {
    if !*CHECK_ENABLED {
        return Some(dev_version_cache());
    }

    try_load_cache(cache_path)
        .await
        .inspect_err(|error| {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Unable to load cached version info")
            );
        })
        .ok()
}

async fn try_load_cache(cache_path: &PathBuf) -> Result<VersionCache, Error> {
    log::debug!("Loading version check cache from {}", cache_path.display());

    let content = tokio::fs::read_to_string(&cache_path)
        .map_err(Error::ReadVersionCache)
        .await?;

    serde_json::from_str(&content).map_err(Error::Deserialize)
}

/// Check if the cache is left over from another version of the app. If so, discard it.
fn cache_is_stale(cache: &VersionCache, current_version: &Version) -> bool {
    &cache.cache_version != current_version
}

/// Return the lowest metadata version to accept, given a cache from any app version.
///
/// The metadata version is kept even if the rest of the cache is stale, since it is used
/// as a floor to avoid accepting downgraded metadata.
#[cfg(not(target_os = "android"))]
fn min_metadata_version(cache: Option<&VersionCache>) -> usize {
    cache
        .map(|cache| cache.metadata_version)
        .unwrap_or(0)
        .max(MIN_VERIFY_METADATA_VERSION)
}

fn dev_version_cache() -> VersionCache {
    VersionCache {
        cache_version: mullvad_version::VERSION.parse().unwrap(),
        current_version_supported: false,
        #[cfg(not(target_os = "android"))]
        version_info: VersionInfo {
            stable: Metadata {
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

    /// The metadata version of a cache must be used as a floor, even if the cache is stale
    #[test]
    #[cfg(not(target_os = "android"))]
    fn test_min_metadata_version() {
        let stale_cache = VersionCache {
            metadata_version: MIN_VERIFY_METADATA_VERSION + 10,
            ..version_cache("2025.5", "2025.7", None)
        };
        assert!(cache_is_stale(&stale_cache, &APP_VERSION));
        assert_eq!(
            min_metadata_version(Some(&stale_cache)),
            MIN_VERIFY_METADATA_VERSION + 10
        );

        // A metadata version lower than the minimum must be raised to the minimum
        let old_cache = VersionCache {
            metadata_version: 0,
            ..version_cache("2025.5", "2025.7", None)
        };
        assert_eq!(
            min_metadata_version(Some(&old_cache)),
            MIN_VERIFY_METADATA_VERSION
        );
        assert_eq!(min_metadata_version(None), MIN_VERIFY_METADATA_VERSION);
    }

    /// An upgrade retains the metadata floor, but a response at that floor must
    /// replace all app-specific information and survive a daemon restart.
    /// The API response is mocked at the request boundary; signature verification
    /// is covered by `mullvad-update`'s tests.
    #[tokio::test]
    #[cfg(not(target_os = "android"))]
    async fn test_equal_counter_response_after_app_upgrade() {
        use mullvad_api::version::AppVersionResponse;

        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join(VERSION_INFO_FILENAME);
        let counter = MIN_VERIFY_METADATA_VERSION + 10;
        let previous = VersionCache {
            current_version_supported: true,
            metadata_version: counter,
            etag: Some("previous-app-etag".to_owned()),
            ..version_cache("2025.5", "2025.6", None)
        };
        assert_ne!(previous.cache_version, *APP_VERSION);
        tokio::fs::write(&cache_path, serde_json::to_vec(&previous).unwrap())
            .await
            .unwrap();

        // Read the real on-disk cache even when update checks are disabled in a
        // development build, then use the same startup policy as `spawn`.
        let state = VersionCheckState::from_cache(Some(try_load_cache(&cache_path).await.unwrap()));
        assert!(state.last_app_version_info.is_none());
        assert_eq!(state.min_metadata_version, counter);
        let mut checker = VersionUpdaterInner {
            last_app_version_info: state.last_app_version_info,
        };

        let expected_info =
            version_cache(mullvad_version::VERSION, "2025.7", Some("2025.8-beta1")).version_info;
        let response = AppVersionResponse {
            metadata_version: counter,
            current_version_supported: false,
            version_info: expected_info.clone(),
            etag: Some("current-app-etag".to_owned()),
        };
        let fresh_cache = version_check_with(
            checker.last_app_version_info.clone(),
            state.min_metadata_version,
            "test-platform".to_owned(),
            |minimum, platform_version, etag| {
                assert_eq!(minimum, counter, "forward the recovered floor to the API");
                assert_eq!(platform_version.as_deref(), Some("test-platform"));
                assert!(etag.is_none(), "do not reuse the previous app's ETag");
                async { Ok(Some(response)) }
            },
        )
        .await
        .unwrap();
        assert_eq!(fresh_cache.cache_version, *APP_VERSION);
        assert_eq!(fresh_cache.metadata_version, counter);
        assert_eq!(fresh_cache.version_info, expected_info);
        assert!(!fresh_cache.current_version_supported);
        assert_eq!(fresh_cache.etag.as_deref(), Some("current-app-etag"));

        let (update_sender, mut update_rx) = mpsc::unbounded();
        let update = UpdateContext {
            cache_path: cache_path.clone(),
            update_sender,
        };
        checker
            .handle_version_response(
                &|cache| Box::pin(update.update(cache)),
                Ok(fresh_cache.clone()),
            )
            .await;
        assert_eq!(checker.last_app_version_info.as_ref(), Some(&fresh_cache));
        assert_eq!(update_rx.try_recv().unwrap(), fresh_cache);

        let persisted = try_load_cache(&cache_path).await.unwrap();
        assert_eq!(persisted, fresh_cache);
        let restarted = VersionCheckState::from_cache(Some(persisted));
        assert_eq!(restarted.last_app_version_info, Some(fresh_cache));
        assert_eq!(restarted.min_metadata_version, counter);
    }

    /// Missing caches and stale caches below the compiled minimum must still
    /// forward that minimum to the API without a conditional request.
    #[tokio::test]
    #[cfg(not(target_os = "android"))]
    async fn test_uncached_request_uses_compiled_metadata_minimum() {
        for cache in [
            None,
            Some(VersionCache {
                metadata_version: 0,
                etag: Some("stale-etag".to_owned()),
                ..version_cache("2025.5", "2025.6", None)
            }),
        ] {
            let state = VersionCheckState::from_cache(cache);
            assert!(state.last_app_version_info.is_none());
            let result = version_check_with(
                state.last_app_version_info,
                state.min_metadata_version,
                "test-platform".to_owned(),
                |minimum, _, etag| {
                    assert_eq!(minimum, MIN_VERIFY_METADATA_VERSION);
                    assert!(etag.is_none());
                    async { Err(mullvad_api::rest::Error::Aborted) }
                },
            )
            .await;
            assert!(matches!(
                result,
                Err(Error::Download(mullvad_api::rest::Error::Aborted))
            ));
        }
    }

    /// A cache for the installed app still supplies the counter and ETag, and a
    /// not-modified response keeps its complete version information.
    #[tokio::test]
    #[cfg(not(target_os = "android"))]
    async fn test_current_cache_handles_not_modified_response() {
        let current = VersionCache {
            metadata_version: MIN_VERIFY_METADATA_VERSION + 10,
            etag: Some("current-etag".to_owned()),
            ..version_cache(mullvad_version::VERSION, "2025.7", None)
        };
        let state = VersionCheckState::from_cache(Some(current.clone()));
        assert_eq!(state.last_app_version_info, Some(current.clone()));

        let updated = version_check_with(
            state.last_app_version_info,
            MIN_VERIFY_METADATA_VERSION,
            "test-platform".to_owned(),
            |minimum, platform_version, etag| {
                assert_eq!(minimum, current.metadata_version);
                assert!(platform_version.is_none());
                assert_eq!(etag, current.etag);
                async { Ok(None) }
            },
        )
        .await
        .unwrap();
        assert_eq!(updated, current);
    }

    #[cfg_attr(target_os = "android", expect(unused_variables))]
    fn version_cache(cache_version: &str, stable: &str, beta: Option<&str>) -> VersionCache {
        VersionCache {
            cache_version: cache_version.parse().unwrap(),
            current_version_supported: false,
            #[cfg(not(target_os = "android"))]
            version_info: VersionInfo {
                stable: Metadata {
                    version: stable.parse().unwrap(),
                    urls: vec![],
                    size: 0,
                    changelog: "".to_owned(),
                    sha256: [0u8; 32],
                },
                beta: beta.map(|beta| Metadata {
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

    /// Platform timestamp and etag must be updated even if metadata version is unchanged
    #[tokio::test]
    #[cfg(not(target_os = "android"))]
    async fn test_platform_timestamp_update() {
        // If the metadata version is unchanged, we should keep the existing metadata
        // But update the etag and platform timestamp anyway
        let prev_cache = VersionCache {
            last_platform_header_check: SystemTime::now() - PLATFORM_HEADER_INTERVAL,
            current_version_supported: true,
            metadata_version: 11,
            ..dev_version_cache()
        };
        let new_cache = VersionCache {
            last_platform_header_check: SystemTime::now(),
            etag: Some("etag2".to_owned()),
            current_version_supported: false,
            metadata_version: 11,
            ..dev_version_cache()
        };

        let mut checker = VersionUpdaterInner {
            last_app_version_info: Some(prev_cache),
        };
        checker
            .update_version_info(&fake_updater(Default::default()), new_cache.clone())
            .await;
        let updated_cache = checker.last_app_version_info.as_ref().unwrap();
        assert_eq!(
            updated_cache.last_platform_header_check, new_cache.last_platform_header_check,
            "timestamp should be updated"
        );
        assert_eq!(updated_cache.etag, new_cache.etag, "etag should be updated");
        assert!(
            updated_cache.current_version_supported,
            "other metadata should be unchanged"
        );

        // If the metadata version is higher, we should update everything
        let prev_cache = VersionCache {
            last_platform_header_check: SystemTime::now() - PLATFORM_HEADER_INTERVAL,
            current_version_supported: true,
            metadata_version: 11,
            ..dev_version_cache()
        };
        let new_cache = VersionCache {
            last_platform_header_check: SystemTime::now(),
            etag: Some("etag2".to_owned()),
            current_version_supported: false,
            metadata_version: 12,
            ..dev_version_cache()
        };

        let mut checker = VersionUpdaterInner {
            last_app_version_info: Some(prev_cache),
        };
        checker
            .update_version_info(&fake_updater(Default::default()), new_cache.clone())
            .await;
        let updated_cache = checker.last_app_version_info.as_ref().unwrap();
        assert_eq!(updated_cache, &new_cache, "cache should be fully updated");
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
        _cache: Option<VersionCache>,
    ) -> BoxFuture<'static, Result<VersionCache, Error>> {
        Box::pin(async { Ok(fake_version_response()) })
    }

    fn fake_version_response() -> VersionCache {
        // TODO: The tests pass, but check that this is a sane fake version cache anyway
        VersionCache {
            cache_version: mullvad_version::VERSION.parse().unwrap(),
            current_version_supported: true,
            #[cfg(not(target_os = "android"))]
            version_info: VersionInfo {
                stable: Metadata {
                    version: "2025.5".parse().unwrap(),
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
