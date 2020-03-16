use crate::{version::PRODUCT_VERSION, DaemonEventSender};
use futures::{Async, Future, Poll};
use mullvad_rpc::{rest::MullvadRestHandle, AppVersionProxy};
use mullvad_types::version::AppVersionInfo;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;
use tokio_timer::{TimeoutError, Timer};

const VERSION_INFO_FILENAME: &str = "version-info.json";

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
    ReadCachedRelays(#[error(source)] io::Error),

    #[error(display = "Failed to open app version cache file for writing")]
    WriteRelayCache(#[error(source)] io::Error),

    #[error(display = "Failure in serialization of the version info")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Timed out when trying to check the latest app version")]
    DownloadTimeout,

    #[error(display = "Failed to check the latest app version")]
    Download(#[error(source)] mullvad_rpc::rest::Error),

    #[error(display = "Clearing version check cache due to a version mismatch")]
    CacheVersionMismatch,
}

impl<T> From<TimeoutError<T>> for Error {
    fn from(_: TimeoutError<T>) -> Error {
        Error::DownloadTimeout
    }
}


pub(crate) struct VersionUpdater {
    version_proxy: AppVersionProxy,
    cache_path: PathBuf,
    update_sender: DaemonEventSender<AppVersionInfo>,
    last_app_version_info: AppVersionInfo,
    next_update_time: Instant,
    state: VersionUpdaterState,
}

enum VersionUpdaterState {
    Sleeping(tokio_timer::Sleep),
    Updating(Box<dyn Future<Item = AppVersionInfo, Error = Error> + Send + 'static>),
}

impl VersionUpdater {
    pub fn new(
        rpc_handle: MullvadRestHandle,
        cache_dir: PathBuf,
        update_sender: DaemonEventSender<AppVersionInfo>,
        last_app_version_info: AppVersionInfo,
    ) -> Self {
        let version_proxy = AppVersionProxy::new(rpc_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        Self {
            version_proxy,
            cache_path,
            update_sender,
            last_app_version_info,
            next_update_time: Instant::now(),
            state: VersionUpdaterState::Sleeping(Self::create_sleep_future()),
        }
    }

    fn create_sleep_future() -> tokio_timer::Sleep {
        Timer::default().sleep(UPDATE_CHECK_INTERVAL)
    }

    fn create_update_future(
        &mut self,
    ) -> Box<dyn Future<Item = AppVersionInfo, Error = Error> + Send + 'static> {
        let download_future = self
            .version_proxy
            .version_check(PRODUCT_VERSION.to_owned(), PLATFORM)
            .map_err(Error::Download);
        let future = Timer::default().timeout(download_future, DOWNLOAD_TIMEOUT);
        Box::new(future)
    }

    fn write_cache(&self) -> Result<(), Error> {
        log::debug!(
            "Writing version check cache to {}",
            self.cache_path.display()
        );
        let file = File::create(&self.cache_path).map_err(Error::WriteRelayCache)?;
        let cached_app_version = CachedAppVersionInfo::from(self.last_app_version_info.clone());
        serde_json::to_writer_pretty(io::BufWriter::new(file), &cached_app_version)
            .map_err(Error::Serialize)
    }
}

impl Future for VersionUpdater {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if self.update_sender.is_closed() {
                log::warn!("Version update receiver is closed, stopping version updater");
                return Ok(Async::Ready(()));
            }
            let next_state = match &mut self.state {
                VersionUpdaterState::Sleeping(timer) => match timer.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(e) => {
                        log::error!("Version check sleep error: {}", e);
                        return Err(());
                    }
                    Ok(Async::Ready(())) => {
                        if Instant::now() > self.next_update_time {
                            VersionUpdaterState::Updating(self.create_update_future())
                        } else {
                            VersionUpdaterState::Sleeping(Self::create_sleep_future())
                        }
                    }
                },
                VersionUpdaterState::Updating(future) => match future.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(error) => {
                        log::error!("{}", error.display_chain_with_msg("Version check failed"));
                        self.next_update_time = Instant::now() + UPDATE_INTERVAL_ERROR;
                        VersionUpdaterState::Sleeping(Self::create_sleep_future())
                    }
                    Ok(Async::Ready(app_version_info)) => {
                        log::debug!("Got new version check: {:?}", app_version_info);
                        self.next_update_time = Instant::now() + UPDATE_INTERVAL;
                        if app_version_info != self.last_app_version_info {
                            if self.update_sender.send(app_version_info.clone()).is_err() {
                                log::warn!(
                                    "Version update receiver is closed, stopping version updater"
                                );
                                return Ok(Async::Ready(()));
                            }
                            self.last_app_version_info = app_version_info;
                            if let Err(e) = self.write_cache() {
                                log::error!(
                                    "{}",
                                    e.display_chain_with_msg(
                                        "Unable to cache version check response"
                                    )
                                );
                            }
                        }
                        VersionUpdaterState::Sleeping(Self::create_sleep_future())
                    }
                },
            };
            self.state = next_state;
        }
    }
}

fn try_load_cache(cache_dir: &Path) -> Result<AppVersionInfo, Error> {
    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());
    let file = File::open(&path).map_err(Error::ReadCachedRelays)?;
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
                supported: true,
                latest_stable: PRODUCT_VERSION.to_owned(),
                latest_beta: PRODUCT_VERSION.to_owned(),
                latest: PRODUCT_VERSION.to_owned(),
            }
        }
    }
}
