use crate::{
    version::{is_beta_version, PRODUCT_VERSION},
    DaemonEventSender,
};
use futures::{channel::mpsc, stream::FusedStream, FutureExt, SinkExt, StreamExt, TryFutureExt};
use mullvad_rpc::{rest::MullvadRestHandle, AppVersionProxy};
use mullvad_types::version::{AppVersionInfo, ParsedAppVersion};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    future::Future,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;
use tokio::fs::File;

const VERSION_INFO_FILENAME: &str = "version-info.json";

lazy_static::lazy_static! {
    static ref APP_VERSION: ParsedAppVersion = ParsedAppVersion::from_str(PRODUCT_VERSION).unwrap();
    static ref IS_DEV_BUILD: bool = APP_VERSION.is_dev();
}

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);
/// How often the updater should wake up to check the in-memory cache.
/// This exist to prevent problems around sleeping. If you set it to sleep
/// for `UPDATE_INTERVAL` directly and the computer is suspended, that clock
/// won't tick, and the next update will be after 24 hours of the computer being *on*.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 5);
/// Wait this long until next check after a successful check
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);
/// Wait this long until next try if an update failed
const UPDATE_INTERVAL_ERROR: Duration = Duration::from_secs(60 * 60 * 6);

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(target_os = "android")]
const PLATFORM: &str = "android";


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct CachedAppVersionInfo {
    #[serde(flatten)]
    pub version_info: AppVersionInfo,
    pub cached_from_version: String,
}

impl From<AppVersionInfo> for CachedAppVersionInfo {
    fn from(version_info: AppVersionInfo) -> CachedAppVersionInfo {
        CachedAppVersionInfo {
            version_info,
            cached_from_version: PRODUCT_VERSION.to_owned(),
        }
    }
}

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open app version cache file for reading")]
    ReadVersionCache(#[error(source)] io::Error),

    #[error(display = "Failed to open app version cache file for writing")]
    WriteVersionCache(#[error(source)] io::Error),

    #[error(display = "Failure in serialization of the version info")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Failed to check the latest app version")]
    Download(#[error(source)] mullvad_rpc::rest::Error),

    #[error(display = "Clearing version check cache due to a version mismatch")]
    CacheVersionMismatch,
}


pub(crate) struct VersionUpdater {
    version_proxy: AppVersionProxy,
    cache_path: PathBuf,
    update_sender: DaemonEventSender<AppVersionInfo>,
    last_app_version_info: AppVersionInfo,
    platform_version: String,
    next_update_time: Instant,
    show_beta_releases: bool,
    rx: Option<mpsc::Receiver<bool>>,
}

#[derive(Clone)]
pub(crate) struct VersionUpdaterHandle {
    tx: mpsc::Sender<bool>,
}

impl VersionUpdaterHandle {
    pub async fn set_show_beta_releases(&mut self, show_beta_releases: bool) {
        if self.tx.send(show_beta_releases).await.is_err() {
            log::error!("Version updater already down, can't send new `show_beta_releases` state");
        }
    }
}

impl VersionUpdater {
    pub fn new(
        mut rpc_handle: MullvadRestHandle,
        cache_dir: PathBuf,
        update_sender: DaemonEventSender<AppVersionInfo>,
        last_app_version_info: AppVersionInfo,
        show_beta_releases: bool,
    ) -> (Self, VersionUpdaterHandle) {
        rpc_handle.factory.timeout = DOWNLOAD_TIMEOUT;
        let version_proxy = AppVersionProxy::new(rpc_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        let (tx, rx) = mpsc::channel(1);
        let platform_version = mullvad_platform_metadata::version();

        (
            Self {
                version_proxy,
                cache_path,
                update_sender,
                last_app_version_info,
                platform_version,
                next_update_time: Instant::now(),
                show_beta_releases,
                rx: Some(rx),
            },
            VersionUpdaterHandle { tx },
        )
    }

    fn create_update_future(
        &self,
    ) -> impl Future<Output = Result<mullvad_rpc::AppVersionResponse, Error>> + Send + 'static {
        let version_proxy = self.version_proxy.clone();
        let platform_version = self.platform_version.clone();
        let download_future_factory = move || {
            let response = version_proxy.version_check(
                PRODUCT_VERSION.to_owned(),
                PLATFORM,
                platform_version.clone(),
            );
            response.map_err(Error::Download)
        };

        let should_retry = |result: &Result<_, _>| -> bool { result.is_err() };

        Box::pin(talpid_core::future_retry::retry_future_with_backoff(
            download_future_factory,
            should_retry,
            std::iter::repeat(UPDATE_INTERVAL_ERROR),
        ))
    }

    async fn write_cache(&self) -> Result<(), Error> {
        log::debug!(
            "Writing version check cache to {}",
            self.cache_path.display()
        );
        let mut file = File::create(&self.cache_path)
            .await
            .map_err(Error::WriteVersionCache)?;
        let cached_app_version = CachedAppVersionInfo::from(self.last_app_version_info.clone());
        let mut buf = serde_json::to_vec_pretty(&cached_app_version).map_err(Error::Serialize)?;
        let mut read_buf: &[u8] = buf.as_mut();

        let _ = tokio::io::copy(&mut read_buf, &mut file)
            .await
            .map_err(Error::WriteVersionCache)?;
        Ok(())
    }

    fn response_to_version_info(
        &mut self,
        response: mullvad_rpc::AppVersionResponse,
    ) -> AppVersionInfo {
        let suggested_upgrade = if !*IS_DEV_BUILD {
            Self::suggested_upgrade(
                &*APP_VERSION,
                &response,
                self.show_beta_releases || is_beta_version(),
            )
        } else {
            None
        };

        AppVersionInfo {
            supported: response.supported,
            latest_stable: response.latest_stable.unwrap_or_else(|| "".to_owned()),
            latest_beta: response.latest_beta,
            suggested_upgrade,
        }
    }

    fn suggested_upgrade(
        current_version: &ParsedAppVersion,
        response: &mullvad_rpc::AppVersionResponse,
        show_beta: bool,
    ) -> Option<String> {
        let stable_version = response
            .latest_stable
            .as_ref()
            .and_then(|stable| ParsedAppVersion::from_str(stable));

        let beta_version = if show_beta {
            ParsedAppVersion::from_str(&response.latest_beta)
        } else {
            None
        };

        let latest_version = stable_version.iter().chain(beta_version.iter()).max()?;

        if current_version < latest_version {
            Some(latest_version.to_string())
        } else {
            None
        }
    }

    pub async fn run(mut self) {
        let mut rx = self.rx.take().unwrap().fuse();
        let next_delay = || tokio::time::delay_for(UPDATE_CHECK_INTERVAL).fuse();
        let mut check_delay = next_delay();
        let mut version_check = futures::future::Fuse::terminated();

        // If this is a dev build ,there's no need to pester the API for version checks.
        if *IS_DEV_BUILD {
            log::warn!("Not checking for updates because this is a development build");
            while let Some(_) = rx.next().await {}
            return;
        }

        loop {
            futures::select! {
                show_beta_releases = rx.next() => {
                    match show_beta_releases {
                        Some(show_beta_releases ) => {
                            self.show_beta_releases = show_beta_releases;
                        },
                        // time to shut down
                        None => {
                            return;
                        },
                    }
                },

                _sleep = check_delay => {
                    if rx.is_terminated() || self.update_sender.is_closed() {
                        return;
                    }

                    if Instant::now() > self.next_update_time {
                        let download_future = self.create_update_future().fuse();
                        version_check = download_future;
                    } else {
                        check_delay = next_delay();
                    }

                },

                response = version_check => {
                    if rx.is_terminated() || self.update_sender.is_closed() {
                        return;
                    }
                    self.next_update_time = Instant::now() + UPDATE_INTERVAL;

                    match response {
                        Ok(version_info_response) => {
                            let new_version_info = self.response_to_version_info(version_info_response);
                            // if daemon can't be reached, return immediately
                            if self.update_sender.send(new_version_info.clone()).is_err() {
                                return;
                            }

                            self.last_app_version_info = new_version_info;
                            if let Err(err) = self.write_cache().await {
                                log::error!("Failed to save version cache to disk: {}", err);

                            }
                        },
                        Err(err) => {
                            log::error!("Failed to get fetch version info - {}", err);
                        },
                    }

                    check_delay = next_delay();
                },
            }
        }
    }
}

fn try_load_cache(cache_dir: &Path) -> Result<AppVersionInfo, Error> {
    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());
    let file = fs::File::open(&path).map_err(Error::ReadVersionCache)?;
    let version_info: CachedAppVersionInfo =
        serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)?;

    if version_info.cached_from_version == PRODUCT_VERSION {
        Ok(version_info.version_info)
    } else {
        Err(Error::CacheVersionMismatch)
    }
}

pub fn load_cache(cache_dir: &Path) -> AppVersionInfo {
    match try_load_cache(cache_dir) {
        Ok(app_version_info) => app_version_info,
        Err(error) => {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Unable to load cached version info")
            );
            // If we don't have a cache, start out with sane defaults.
            AppVersionInfo {
                supported: !*IS_DEV_BUILD,
                latest_stable: PRODUCT_VERSION.to_owned(),
                latest_beta: PRODUCT_VERSION.to_owned(),
                suggested_upgrade: None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_upgrade_suggestions() {
        let app_version_info = mullvad_rpc::AppVersionResponse {
            supported: true,
            latest: "2020.5-beta3".to_owned(),
            latest_stable: Some("2020.4".to_string()),
            latest_beta: "2020.5-beta3".to_string(),
        };

        let older_stable = ParsedAppVersion::from_str("2020.3").unwrap();
        let current_stable = ParsedAppVersion::from_str("2020.4").unwrap();
        let newer_stable = ParsedAppVersion::from_str("2021.5").unwrap();

        let older_beta = ParsedAppVersion::from_str("2020.3-beta3").unwrap();
        let current_beta = ParsedAppVersion::from_str("2020.5-beta3").unwrap();
        let newer_beta = ParsedAppVersion::from_str("2021.5-beta3").unwrap();

        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_stable, &app_version_info, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_stable, &app_version_info, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_stable, &app_version_info, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_stable, &app_version_info, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_stable, &app_version_info, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_stable, &app_version_info, true),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_beta, &app_version_info, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_beta, &app_version_info, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_beta, &app_version_info, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_beta, &app_version_info, true),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_beta, &app_version_info, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_beta, &app_version_info, true),
            None
        );
    }
}
