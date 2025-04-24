use futures::{
    channel::{mpsc, oneshot},
    future::{BoxFuture, FusedFuture},
    FutureExt, StreamExt, TryFutureExt,
};
use mullvad_api::{
    availability::ApiAvailability, rest::MullvadRestHandle, version::AppVersionProxy,
};

use mullvad_update::version::VersionInfo;
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
use talpid_future::retry::{retry_future, ConstantInterval};
use talpid_types::ErrorExt;
use tokio::{fs::File, io::AsyncReadExt};

use super::Error;

const VERSION_INFO_FILENAME: &str = "version-info.json";

static APP_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::from_str(mullvad_version::VERSION).unwrap());
static IS_DEV_BUILD: LazyLock<bool> = LazyLock::new(|| APP_VERSION.is_dev());

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// Wait this long until next check after a successful check
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);
/// Wait this long until next try if an update failed
const UPDATE_INTERVAL_ERROR: Duration = Duration::from_secs(60 * 60 * 6);
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
    /// Whether the current (installed) version is supported or an upgrade is required
    pub current_version_supported: bool,
    /// The latest available versions
    pub latest_version: mullvad_update::version::VersionInfo,
    pub metadata_version: usize,
}

pub(crate) struct VersionUpdater(());

#[derive(Default)]
struct VersionUpdaterInner {
    /// The last known [AppVersionInfo], along with the time it was determined.
    last_app_version_info: Option<(VersionCache, SystemTime)>,
    /// Oneshot channels for responding to [VersionUpdaterCommand::GetVersionInfo].
    get_version_info_responders: Vec<oneshot::Sender<VersionCache>>,
}

impl VersionUpdater {
    pub(super) async fn spawn(
        mut api_handle: MullvadRestHandle,
        availability_handle: ApiAvailability,
        cache_dir: PathBuf,
        update_sender: mpsc::UnboundedSender<VersionCache>,
        refresh_rx: mpsc::UnboundedReceiver<()>,
    ) {
        // load the last known AppVersionInfo from cache
        let last_app_version_info = load_cache(&cache_dir).await;

        api_handle.factory = api_handle.factory.default_timeout(DOWNLOAD_TIMEOUT);
        let version_proxy = AppVersionProxy::new(api_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        let platform_version = talpid_platform_metadata::short_version();

        let architecture = match talpid_platform_metadata::get_native_arch()
            .expect("IO error while getting native architecture")
            .expect("Failed to get native architecture")
        {
            talpid_platform_metadata::Architecture::X86 => {
                mullvad_update::format::Architecture::X86
            }
            talpid_platform_metadata::Architecture::Arm64 => {
                mullvad_update::format::Architecture::Arm64
            }
        };

        tokio::spawn(
            VersionUpdaterInner {
                last_app_version_info,
                get_version_info_responders: vec![],
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
                    architecture,
                    rollout: 1.0, // TODO: set reasonable rollout,
                },
            ),
        );
    }
}

impl VersionUpdaterInner {
    /// Get the last known [AppVersionInfo]. May be stale.
    pub fn last_app_version_info(&self) -> Option<&VersionCache> {
        self.last_app_version_info.as_ref().map(|(info, _)| info)
    }

    pub fn get_min_metadata_version(&self) -> usize {
        self.last_app_version_info
            .as_ref()
            // Reject version responses with a lower metadata version
            // than the newest version we know about. This is
            // important to prevent downgrade attacks.
            .map(|(info, _)| info.metadata_version)
            .unwrap_or(mullvad_update::MIN_VERIFY_METADATA_VERSION)
    }

    /// Update [Self::last_app_version_info] and write it to disk cache, and notify the `update`
    /// callback.
    async fn update_version_info(
        &mut self,
        update: &impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        new_version_info: VersionCache,
    ) {
        if let Err(err) = update(new_version_info.clone()).await {
            log::error!("Failed to save version cache to disk: {}", err);
        }
        self.last_app_version_info = Some((new_version_info, SystemTime::now()));
    }

    /// Get the time left until [Self::last_app_version_info] becomes stale, and should be
    /// refreshed, or [Duration::ZERO] if it already is stale.
    ///
    /// This happens [UPDATE_INTERVAL] after the last version check.
    fn time_until_version_is_stale(&self) -> Duration {
        let now = SystemTime::now();
        self.last_update_time()
            .and_then(|&last_update_time| now.duration_since(last_update_time).ok())
            .map(|time_since_last_update| UPDATE_INTERVAL.saturating_sub(time_since_last_update))
            // if there is no last_app_version_info, or if clocks are being weird,
            // assume that the version is stale
            .unwrap_or(Duration::ZERO)
    }

    fn last_update_time(&self) -> Option<&SystemTime> {
        self.last_app_version_info
            .as_ref()
            .map(|(_, last_update_time)| last_update_time)
    }

    /// Is [Self::last_app_version_info] stale?
    fn version_is_stale(&self) -> bool {
        self.time_until_version_is_stale().is_zero()
    }

    /// Wait until [Self::last_app_version_info] becomes stale and needs to be refreshed.
    ///
    /// This happens [UPDATE_INTERVAL] after the last version check.
    fn wait_until_version_is_stale(&self) -> Pin<Box<impl FusedFuture<Output = ()> + use<>>> {
        let time_until_stale = self.time_until_version_is_stale();

        // Boxed, pinned, and fused.
        // Alternate title: "We don't want to deal with the borrow checker."
        Box::pin(talpid_time::sleep(time_until_stale).fuse())
    }

    /// Returns true if we are currently handling one or more `GetVersionInfo` commands.
    fn is_running_version_check(&self) -> bool {
        !self.get_version_info_responders.is_empty()
    }

    async fn run(
        self,
        mut refresh_rx: mpsc::UnboundedReceiver<()>,
        update: UpdateContext,
        api: ApiContext,
    ) {
        // If this is a dev build, there's no need to pester the API for version checks.
        if *IS_DEV_BUILD {
            log::warn!("Not checking for updates because this is a development build");
            while let Some(()) = refresh_rx.next().await {
                log::info!("Version check is disabled in dev builds");
            }
            return;
        }

        let update = |info| Box::pin(update.update(info)) as BoxFuture<'static, _>;
        let do_version_check =
            |min_metadata_version| do_version_check(api.clone(), min_metadata_version);
        let do_version_check_in_background = |min_metadata_version| {
            do_version_check_in_background(api.clone(), min_metadata_version)
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
        do_version_check: impl Fn(usize) -> BoxFuture<'static, Result<VersionCache, Error>>,
        do_version_check_in_background: impl Fn(
            usize,
        )
            -> BoxFuture<'static, Result<VersionCache, Error>>,
    ) {
        let mut version_is_stale = self.wait_until_version_is_stale();
        let mut version_check = futures::future::Fuse::terminated();

        loop {
            futures::select! {
                command = refresh_rx.next() => match command {

                    Some(()) => {
                        match (self.version_is_stale(), self.last_app_version_info()) {
                            (false, Some(version_cache)) => {
                                // if the version_info isn't stale, return it immediately.
                                if let Err(err) = update(version_cache.clone()).await {
                                    log::error!("Failed to save version cache to disk: {}", err);
                                }
                            }
                            _ => {
                                // otherwise, start a foreground query to get the latest version_info.
                                if !self.is_running_version_check() {
                                    version_check = do_version_check(self.get_min_metadata_version()).fuse();
                                }

                            }
                        }
                    }

                    // time to shut down
                    None => {
                        break;
                    }
                },

                _ = version_is_stale => {
                    if self.is_running_version_check() {
                        continue;
                    }
                    version_check = do_version_check_in_background(self.get_min_metadata_version()).fuse();
                },

                response = version_check => {
                    match response {
                        Ok(version_info) => {
                            self.update_version_info(&update, version_info).await;

                        }
                        Err(err) => {
                            log::error!("Failed to fetch version info: {err:#}");
                        }
                    }

                    version_is_stale = self.wait_until_version_is_stale();
                },
            }
        }
    }
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
    architecture: mullvad_update::format::Architecture,
    rollout: f32,
}

/// Immediately query the API for the latest [AppVersionInfo].
fn do_version_check(
    api: ApiContext,
    min_metadata_version: usize,
) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let api_handle = api.api_handle.clone();

    let download_future_factory = move || version_check_inner(&api, min_metadata_version);

    // retry immediately on network errors (unless we're offline)
    let should_retry_immediate = move |result: &Result<_, Error>| {
        if let Err(Error::Download(error)) = result {
            error.is_network_error() && !api_handle.is_offline()
        } else {
            false
        }
    };

    Box::pin(retry_future(
        download_future_factory,
        should_retry_immediate,
        IMMEDIATE_RETRY_STRATEGY,
    ))
}

/// Query the API for the latest [AppVersionInfo].
///
/// This function waits until background calls are enabled in
/// [ApiAvailability](mullvad_api::availability::ApiAvailability).
///
/// On any error, this function retries repeatedly every [UPDATE_INTERVAL_ERROR] until success.
fn do_version_check_in_background(
    api: ApiContext,
    min_metadata_version: usize,
) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let download_future_factory = move || {
        let when_available = api.api_handle.wait_background();
        let version_cache = version_check_inner(&api, min_metadata_version);
        async move {
            when_available.await.map_err(Error::ApiCheck)?;
            version_cache.await
        }
    };

    Box::pin(retry_future(
        download_future_factory,
        |result| result.is_err(),
        std::iter::repeat(UPDATE_INTERVAL_ERROR),
    ))
}

/// Combine the old version and new version endpoint
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn version_check_inner(
    api: &ApiContext,
    min_metadata_version: usize,
) -> impl Future<Output = Result<VersionCache, Error>> {
    let v1_endpoint = api.version_proxy.version_check(
        mullvad_version::VERSION.to_owned(),
        PLATFORM,
        api.platform_version.clone(),
    );

    let v2_endpoint = api.version_proxy.version_check_2(
        PLATFORM,
        api.architecture,
        api.rollout,
        min_metadata_version,
    );
    async move {
        let (v1_response, v2_response) =
            tokio::try_join!(v1_endpoint, v2_endpoint).map_err(Error::Download)?;
        Ok(VersionCache {
            current_version_supported: v1_response.supported,
            latest_version: v2_response.0,
            metadata_version: v2_response.1,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn version_check_inner(api: &ApiContext) -> impl Future<Output = Result<VersionCache, Error>> {
    let v1_endpoint = api.version_proxy.version_check(
        mullvad_version::VERSION.to_owned(),
        PLATFORM,
        api.platform_version.clone(),
    );
    async move {
        let response = v1_endpoint.await.map_err(Error::Download)?;
        let latest_stable = response.latest_stable
            .and_then(|version| version.parse().ok())
            // Suggested stable must actually be stable
            .filter(|version: &mullvad_version::Version| version.pre_stable.is_none())
            .ok_or_else(|| Error::MissingStable)?;
        let latest_beta = response.latest_beta
            .and_then(|version| version.parse().ok())
            // Suggested beta must actually be non-stable
            .filter(|version: &mullvad_version::Version| version.pre_stable.is_some());

        Ok(VersionCache {
            current_version_supported: response.supported,
            // Note: We're pretending that this is complete information,
            // but on Android and Linux, most of the information is missing
            latest_version: VersionInfo {
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
        })
    }
}

/// Read the app version cache from the provided directory.
///
/// Returns the [AppVersionInfo] along with the modification time of the cache file,
/// or `None` on any error.
async fn load_cache(cache_dir: &Path) -> Option<(VersionCache, SystemTime)> {
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

async fn try_load_cache(cache_dir: &Path) -> Result<(VersionCache, SystemTime), Error> {
    if *IS_DEV_BUILD {
        return Ok((dev_version_cache(), SystemTime::now()));
    }

    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());

    let mut file = File::open(&path).map_err(Error::ReadVersionCache).await?;
    let meta = file.metadata().map_err(Error::ReadVersionCache).await?;
    let mtime = meta
        .modified()
        .expect("Platforms without file modification times aren't supported");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(Error::ReadVersionCache)
        .await?;

    let cache: VersionCache = serde_json::from_str(&content).map_err(Error::Deserialize)?;

    if cache_is_old(&cache.latest_version, &APP_VERSION) {
        return Err(Error::OutdatedVersion);
    }

    Ok((cache, mtime))
}

/// Check if the cached version is older than the current version. If so, assume the cache is stale.
/// It could in principle mean that a version has been yanked, but we do not really support this,
/// and it should not cause any real issue to delete the cache anyway.
fn cache_is_old(cached_version: &VersionInfo, current_version: &mullvad_version::Version) -> bool {
    let last_version = if current_version.pre_stable.is_some() {
        // Discard suggested version if current beta is newer
        cached_version
            .beta
            .as_ref()
            .unwrap_or(&cached_version.stable)
    } else {
        // Discard suggested version if current stable is newer
        &cached_version.stable
    };
    current_version > &last_version.version
}

fn dev_version_cache() -> VersionCache {
    assert!(*IS_DEV_BUILD);

    VersionCache {
        current_version_supported: false,
        latest_version: VersionInfo {
            stable: mullvad_update::version::Version {
                version: mullvad_version::VERSION.parse().unwrap(),
                changelog: "".to_owned(),
                urls: vec![],
                sha256: [0u8; 32],
                size: 0,
            },
            beta: None,
        },
        metadata_version: 0,
    }
}

#[cfg(test)]
mod test {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use futures::SinkExt;
    use mullvad_update::version::Version;

    use super::*;

    /// Test whether outdated version caches are ignored correctly.
    /// This prevents old versions from being suggested as updates.
    #[test]
    fn test_old_cache() {
        assert!(cache_is_old(
            &version_info("2025.5", None),
            &"2025.6".parse().unwrap()
        ));
        assert!(!cache_is_old(
            &version_info("2025.5", None),
            &"2025.5".parse().unwrap()
        ));
        assert!(!cache_is_old(
            &version_info("2025.5", Some("2025.5-beta1")),
            &"2025.5-beta1".parse().unwrap()
        ));
        assert!(cache_is_old(
            &version_info("2025.5", Some("2025.5-beta1")),
            &"2025.5-beta2".parse().unwrap()
        ));
        assert!(!cache_is_old(
            &version_info("2025.5", None),
            &"2025.5-beta2".parse().unwrap()
        ));
        assert!(cache_is_old(
            &version_info("2025.5", None),
            &"2025.6-beta2".parse().unwrap()
        ));
    }

    fn version_info(stable: &str, beta: Option<&str>) -> VersionInfo {
        VersionInfo {
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
        }
    }

    /// If there's no cached version, it should count as stale
    #[test]
    fn test_version_unknown_is_stale() {
        let checker = VersionUpdaterInner::default();
        assert!(checker.last_app_version_info.is_none());
        assert!(checker.version_is_stale());
    }

    /// If the last checked time is in the future, the version is stale
    #[test]
    fn test_version_invalid_is_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some((
                dev_version_cache(),
                SystemTime::now() + Duration::from_secs(1),
            )),
            ..VersionUpdaterInner::default()
        };
        assert!(checker.version_is_stale());
    }

    /// If we have a cached version that's less than `UPDATE_INTERVAL` old, it should not be stale
    #[test]
    fn test_version_actual_non_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some((
                dev_version_cache(),
                SystemTime::now() - UPDATE_INTERVAL + Duration::from_secs(1),
            )),
            ..VersionUpdaterInner::default()
        };
        assert!(!checker.version_is_stale());
    }

    /// If `UPDATE_INTERVAL` has elapsed, the version should be stale
    #[test]
    fn test_version_actual_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some((dev_version_cache(), SystemTime::now() - UPDATE_INTERVAL)),
            ..VersionUpdaterInner::default()
        };
        assert!(checker.version_is_stale());
    }

    /// Test whether check immediately fetches version info if it's non-existent
    #[tokio::test(start_paused = true)]
    async fn test_version_check_run_immediate() {
        let checker = VersionUpdaterInner::default();

        let updated = Arc::new(AtomicBool::new(false));
        let update = fake_updater(updated.clone());

        let (_tx, rx) = mpsc::unbounded();
        tokio::spawn(checker.run_inner(rx, update, fake_version_check, fake_version_check));

        talpid_time::sleep(Duration::from_secs(10)).await;
        assert!(updated.load(Ordering::SeqCst), "expected immediate update");
    }

    /// Test whether check actually runs after `UPDATE_INTERVAL`
    #[tokio::test(start_paused = true)]
    async fn test_version_check_run_when_stale() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some((dev_version_cache(), SystemTime::now())),
            ..VersionUpdaterInner::default()
        };

        let updated = Arc::new(AtomicBool::new(false));
        let update = fake_updater(updated.clone());

        let (_tx, rx) = mpsc::unbounded();
        tokio::spawn(checker.run_inner(rx, update, fake_version_check, fake_version_check));

        assert!(!updated.load(Ordering::SeqCst));

        talpid_time::sleep(Duration::from_secs(10)).await;
        assert!(
            !updated.load(Ordering::SeqCst),
            "short interval: no update should have occurred"
        );

        talpid_time::sleep(UPDATE_INTERVAL).await;
        assert!(
            updated.load(Ordering::SeqCst),
            "check should have run after `UPDATE_INTERVAL`"
        );
    }

    /// Test whether check runs immediately when requested, if stale
    #[tokio::test(start_paused = true)]
    async fn test_version_check_manual() {
        let checker = VersionUpdaterInner {
            last_app_version_info: Some((dev_version_cache(), SystemTime::now() - UPDATE_INTERVAL)),
            ..VersionUpdaterInner::default()
        };

        let updated = Arc::new(AtomicBool::new(false));
        let update = fake_updater(updated.clone());

        let (mut tx, rx) = mpsc::unbounded();
        tokio::spawn(checker.run_inner(rx, update, fake_version_check, fake_version_check_err));

        // Fail automatic update
        talpid_time::sleep(Duration::from_secs(1)).await;
        assert!(!updated.load(Ordering::SeqCst), "check should fail");

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
    ) -> BoxFuture<'static, Result<VersionCache, Error>> {
        Box::pin(async { Ok(fake_version_response()) })
    }

    fn fake_version_check_err(
        _min_metadata_version: usize,
    ) -> BoxFuture<'static, Result<VersionCache, Error>> {
        Box::pin(retry_future(
            || async { Err(Error::Download(mullvad_api::rest::Error::TimeoutError)) },
            |_| true,
            std::iter::repeat(UPDATE_INTERVAL_ERROR),
        ))
    }

    fn fake_version_response() -> VersionCache {
        // TODO: The tests pass, but check that this is a sane fake version cache anyway
        VersionCache {
            current_version_supported: true,
            latest_version: VersionInfo {
                stable: Version {
                    version: "2025.5".parse::<mullvad_version::Version>().unwrap(),
                    urls: vec![],
                    size: 0,
                    changelog: "".to_owned(),
                    sha256: [0u8; 32],
                },
                beta: None,
            },
            metadata_version: 0,
        }
    }
}
