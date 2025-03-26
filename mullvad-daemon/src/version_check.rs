use crate::{version::is_beta_version, DaemonEventSender};
use futures::{
    channel::{mpsc, oneshot},
    future::{BoxFuture, FusedFuture},
    FutureExt, SinkExt, StreamExt, TryFutureExt,
};
use mullvad_api::{
    availability::ApiAvailability,
    rest::MullvadRestHandle,
    version::AppVersionProxy,
};
use mullvad_types::version::SuggestedUpgrade;
use mullvad_update::version::VersionInfo;
use mullvad_version::Version;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    io,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionCache {
    /// Whether the current (installed) version is supported
    pub supported: bool,
    /// The latest available versions
    pub latest_version: mullvad_update::version::VersionInfo,
}

impl VersionCache {
    /// Return the latest version that should be downloaded
    pub fn target_version(&self, beta_program: bool) -> mullvad_update::version::Version {
        if beta_program {
            self.latest_version.beta.as_ref().unwrap_or(&self.latest_version.stable).clone()
        } else {
            self.latest_version.stable.clone()
        }
    }

    pub fn suggested_upgrade(&self, beta_program: bool) -> mullvad_types::version::AppVersionInfo {
        let version = self.target_version(beta_program);

        mullvad_types::version::AppVersionInfo {
            supported: self.supported,
            suggested_upgrade: Some(SuggestedUpgrade {
                version: version.version,
                changelog: Some(version.changelog),
                // TODO: This should return the downloaded & verified path, if it exists
                verified_installer_path: None,
            })
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open app version cache file for reading")]
    ReadVersionCache(#[source] io::Error),

    #[error("Failed to open app version cache file for writing")]
    WriteVersionCache(#[source] io::Error),

    #[error("Failure in serialization of the version info")]
    Serialize(#[source] serde_json::Error),

    #[error("Failure in deserialization of the version info")]
    Deserialize(#[source] serde_json::Error),

    #[error("Failed to check the latest app version")]
    Download(#[source] mullvad_api::rest::Error),

    #[error("API availability check failed")]
    ApiCheck(#[source] mullvad_api::availability::Error),

    #[error("Clearing version check cache due to a version mismatch")]
    CacheVersionMismatch,

    #[error("Version updater is down")]
    VersionUpdaterDown,

    #[error("Version cache update was aborted")]
    UpdateAborted,
}

pub(crate) struct VersionUpdater;

#[derive(Default)]
struct VersionUpdaterInner {
    beta_program: bool,
    /// The last known [AppVersionInfo], along with the time it was determined.
    last_app_version_info: Option<(VersionCache, SystemTime)>,
    /// Oneshot channels for responding to [VersionUpdaterCommand::GetVersionInfo].
    get_version_info_responders: Vec<oneshot::Sender<VersionCache>>,
}

#[derive(Clone)]
pub(crate) struct VersionUpdaterHandle {
    tx: mpsc::Sender<VersionUpdaterCommand>,
}

enum VersionUpdaterCommand {
    SetShowBetaReleases(bool),
    GetVersionInfo(oneshot::Sender<mullvad_types::version::AppVersionInfo>),
}

impl VersionUpdaterHandle {
    /// Get the latest cached [AppVersionInfo].
    ///
    /// If the cache is stale or missing, this will immediately query the API for the latest
    /// version. This may take a few seconds.
    pub async fn get_version_info(&mut self) -> Result<mullvad_types::version::AppVersionInfo, Error> {
        let (done_tx, done_rx) = oneshot::channel();
        if self
            .tx
            .send(VersionUpdaterCommand::GetVersionInfo(done_tx))
            .await
            .is_err()
        {
            Err(Error::VersionUpdaterDown)
        } else {
            done_rx.await.map_err(|_| Error::UpdateAborted)
        }
    }
}

impl VersionUpdater {
    pub async fn spawn(
        beta_program: bool,
        mut api_handle: MullvadRestHandle,
        availability_handle: ApiAvailability,
        cache_dir: PathBuf,
        update_sender: DaemonEventSender<mullvad_types::version::AppVersionInfo>,
    ) -> VersionUpdaterHandle {
        // load the last known AppVersionInfo from cache
        let last_app_version_info = load_cache(&cache_dir).await;

        let (tx, rx) = mpsc::channel(1);

        api_handle.factory = api_handle.factory.default_timeout(DOWNLOAD_TIMEOUT);
        let version_proxy = AppVersionProxy::new(api_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        let platform_version = talpid_platform_metadata::short_version();

        tokio::spawn(
            VersionUpdaterInner {
                beta_program,
                last_app_version_info,
                get_version_info_responders: vec![],
            }
            .run(
                rx,
                UpdateContext {
                    cache_path,
                    update_sender,
                },
                ApiContext {
                    api_handle: availability_handle,
                    version_proxy,
                    platform_version,
                },
            ),
        );

        VersionUpdaterHandle { tx }
    }
}

impl VersionUpdaterInner {
    /// Get the last known [AppVersionInfo]. May be stale.
    pub fn last_app_version_info(&self) -> Option<&VersionCache> {
        self.last_app_version_info.as_ref().map(|(info, _)| info)
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
        self
            .last_app_version_info
            .as_ref()
            .map(|(_, last_update_time)| last_update_time)
            .and_then(|&last_update_time| now.duration_since(last_update_time).ok())
            .map(|time_since_last_update| UPDATE_INTERVAL.saturating_sub(time_since_last_update))
            // if there is no last_app_version_info, or if clocks are being weird,
            // assume that the version is stale
            .unwrap_or(Duration::ZERO)
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
        mut rx: mpsc::Receiver<VersionUpdaterCommand>,
        update: UpdateContext,
        api: ApiContext,
    ) {
        // If this is a dev build, there's no need to pester the API for version checks.
        if *IS_DEV_BUILD {
            log::warn!("Not checking for updates because this is a development build");
            while let Some(cmd) = rx.next().await {
                if let VersionUpdaterCommand::GetVersionInfo(done_tx) = cmd {
                    log::info!("Version check is disabled in dev builds");
                    let _ = done_tx.send(dev_version_cache());
                }
            }
            return;
        }

        let update = |info| Box::pin(update.update(info, self.beta_program)) as BoxFuture<'static, _>;
        let do_version_check = || do_version_check(api.clone());
        let do_version_check_in_background = || do_version_check_in_background(api.clone());

        self.run_inner(rx, update, do_version_check, do_version_check_in_background)
            .await
    }

    async fn run_inner(
        mut self,
        mut rx: mpsc::Receiver<VersionUpdaterCommand>,
        update: impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>>,
        do_version_check: impl Fn() -> BoxFuture<'static, Result<VersionCache, Error>>,
        do_version_check_in_background: impl Fn()
            -> BoxFuture<'static, Result<VersionCache, Error>>,
    ) {
        let mut version_is_stale = self.wait_until_version_is_stale();
        let mut version_check = futures::future::Fuse::terminated();

        loop {
            futures::select! {
                command = rx.next() => match command {
                    Some(VersionUpdaterCommand::SetShowBetaReleases(show_beta_releases)) => {
                        self.beta_program = show_beta_releases;

                        if let Some(last_version) = self.last_app_version_info() {
                            // FIXME: notify
                            let _ = self.send(last_version.suggested_upgrade(self.beta_program));
                        }
                    }

                    Some(VersionUpdaterCommand::GetVersionInfo(done_tx)) => {
                        match (self.version_is_stale(), self.last_app_version_info()) {
                            (false, Some(version_info)) => {
                                // if the version_info isn't stale, return it immediately.
                                let _ = done_tx.send(version_info.clone());
                            }
                            _ => {
                                // otherwise, start a foreground query to get the latest version_info.
                                if !self.is_running_version_check() {
                                    version_check = do_version_check().fuse();
                                }
                                self.get_version_info_responders.retain(|r| !r.is_canceled());
                                self.get_version_info_responders.push(done_tx);
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
                    version_check = do_version_check_in_background().fuse();
                },

                response = version_check => {
                    match response {
                        Ok(version_info) => {
                            // Respond to all pending GetVersionInfo commands
                            for done_tx in self.get_version_info_responders.drain(..) {
                                let _ = done_tx.send(version_info.clone());
                            }

                            self.update_version_info(&update, new_version_info).await;

                        }
                        Err(err) => {
                            log::error!("Failed to fetch version info: {err:#}");
                            self.get_version_info_responders.clear();
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
    update_sender: DaemonEventSender<mullvad_types::version::AppVersionInfo>,
}

impl UpdateContext {
    /// Write [VersionUpdaterInner::last_app_version_info], if any, to the cache file
    /// ([VERSION_INFO_FILENAME]). Also, notify `self.update_sender`
    fn update(
        &self,
        last_app_version: VersionCache,
        beta_program: bool,
    ) -> impl Future<Output = Result<(), Error>> + use<> {
        let _ = self.update_sender.send(last_app_version.suggested_upgrade(beta_program));
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
fn do_version_check(api: ApiContext) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let api_handle = api.api_handle.clone();

    let download_future_factory = move || {
        let api = api.clone();
        async move {
            let first = api
                .version_proxy
                .version_check(
                    mullvad_version::VERSION.to_owned(),
                    PLATFORM,
                    api.platform_version.clone(),
                )
                .map_err(Error::Download);
            let second = api
                .version_proxy
                .version_check_2(
                    PLATFORM,
                    // TODO: get current architecture (from talpid_platform_metadata)
                    mullvad_update::format::Architecture::X86,
                    // TODO: set reasonable rollout,
                    0.,
                    // TODO: set last known metadata version + 1
                    0,
                )
                .map_err(Error::Download);
            let (v1_response, v2_response) = tokio::try_join!(first, second)?;

            Ok(VersionCache {
                supported: v1_response.supported,
                latest_version: v2_response,
            })
        }
    };

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
) -> BoxFuture<'static, Result<VersionCache, Error>> {
    let download_future_factory = move || {
        let when_available = api.api_handle.wait_background();
        let request = api.version_proxy.version_check(
            mullvad_version::VERSION.to_owned(),
            PLATFORM,
            api.platform_version.clone(),
        );

        let first = api
            .version_proxy
            .version_check(
                mullvad_version::VERSION.to_owned(),
                PLATFORM,
                api.platform_version.clone(),
            )
            .map_err(Error::Download);
        let second = api
            .version_proxy
            .version_check_2(
                PLATFORM,
                // TODO: get current architecture (from talpid_platform_metadata)
                mullvad_update::format::Architecture::X86,
                // TODO: set reasonable rollout,
                0.,
                // TODO: set last known metadata version + 1
                0,
            )
            .map_err(Error::Download);

        async move {
            when_available.await.map_err(Error::ApiCheck)?;
            let (v1_response, v2_response) = tokio::try_join!(first, second)?;
            Ok(VersionCache {
                supported: v1_response.supported,
                latest_version: v2_response,
            })
        }
    };

    Box::pin(retry_future(
        download_future_factory,
        |result| result.is_err(),
        std::iter::repeat(UPDATE_INTERVAL_ERROR),
    ))
}

/// Read the app version cache from the provided directory.
///
/// Returns the [AppVersionInfo] along with the modification time of the cache file,
/// or `None` on any error.
async fn load_cache(cache_dir: &Path) -> Option<(VersionCache, SystemTime)> {
    try_load_cache(cache_dir)
        .await
        .inspect_err(|error| {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Unable to load cached version info")
            )
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

    let cache = serde_json::from_str(&content).map_err(Error::Deserialize)?;

    // TODO: discard cache if the latest (beta) version is older than the current version

    Ok((cache, mtime))
}

fn dev_version_cache() -> VersionCache {
    assert!(*IS_DEV_BUILD);

    VersionCache {
        supported: false,
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
    }
}

#[cfg(test)]
mod test {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use super::*;

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

        let (_tx, rx) = mpsc::channel(1);
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

        let (_tx, rx) = mpsc::channel(1);
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

        let (mut tx, rx) = mpsc::channel(1);
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

        // The next request should do nothing
        send_version_request(&mut tx).await.unwrap();
        talpid_time::sleep(Duration::from_secs(1)).await;
        assert!(!updated.load(Ordering::SeqCst), "expected cached version");
    }

    async fn send_version_request(
        tx: &mut mpsc::Sender<VersionUpdaterCommand>,
    ) -> Result<(), futures::channel::mpsc::SendError> {
        let (done_tx, _done_rx) = oneshot::channel();
        tx.send(VersionUpdaterCommand::GetVersionInfo(done_tx))
            .await
    }

    fn fake_updater(
        updated: Arc<AtomicBool>,
    ) -> impl Fn(VersionCache) -> BoxFuture<'static, Result<(), Error>> {
        move |_new_version| {
            updated.store(true, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }

    fn fake_version_check() -> BoxFuture<'static, Result<AppVersionResponse, Error>> {
        Box::pin(async { Ok(fake_version_response()) })
    }

    fn fake_version_check_err() -> BoxFuture<'static, Result<AppVersionResponse, Error>> {
        Box::pin(retry_future(
            || async { Err(Error::Download(mullvad_api::rest::Error::TimeoutError)) },
            |_| true,
            std::iter::repeat(UPDATE_INTERVAL_ERROR),
        ))
    }

    fn fake_version_response() -> AppVersionResponse {
        AppVersionResponse {
            supported: true,
            latest: "2024.1".to_owned(),
            latest_stable: None,
            latest_beta: "2024.1-beta1".to_owned(),
        }
    }

    #[test]
    fn test_version_upgrade_suggestions() {
        let latest_stable = Some("2020.4".to_string());
        let latest_beta = "2020.5-beta3";

        let older_stable = Version::from_str("2020.3").unwrap();
        let current_stable = Version::from_str("2020.4").unwrap();
        let newer_stable = Version::from_str("2021.5").unwrap();

        let older_beta = Version::from_str("2020.3-beta3").unwrap();
        let current_beta = Version::from_str("2020.5-beta3").unwrap();
        let newer_beta = Version::from_str("2021.5-beta3").unwrap();

        let older_alpha = Version::from_str("2020.3-alpha3").unwrap();
        let current_alpha = Version::from_str("2020.5-alpha3").unwrap();
        let newer_alpha = Version::from_str("2021.5-alpha3").unwrap();

        assert_eq!(
            suggested_upgrade(&older_stable, &latest_stable, latest_beta, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&older_stable, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&current_stable, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            suggested_upgrade(&current_stable, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&newer_stable, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            suggested_upgrade(&newer_stable, &latest_stable, latest_beta, true),
            None
        );

        assert_eq!(
            suggested_upgrade(&older_beta, &latest_stable, latest_beta, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&older_beta, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&current_beta, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            suggested_upgrade(&current_beta, &latest_stable, latest_beta, true),
            None
        );
        assert_eq!(
            suggested_upgrade(&newer_beta, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            suggested_upgrade(&newer_beta, &latest_stable, latest_beta, true),
            None
        );

        assert_eq!(
            suggested_upgrade(&older_alpha, &latest_stable, latest_beta, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&older_alpha, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&current_alpha, &latest_stable, latest_beta, false),
            None,
        );
        assert_eq!(
            suggested_upgrade(&current_alpha, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            suggested_upgrade(&newer_alpha, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            suggested_upgrade(&newer_alpha, &latest_stable, latest_beta, true),
            None
        );
    }
}
