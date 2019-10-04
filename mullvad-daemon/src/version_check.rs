use futures::{Async, Future, Poll};
use mullvad_rpc::{AppVersionProxy, HttpHandle};
use mullvad_types::version::AppVersionInfo;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
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

pub struct VersionUpdater<F: Fn(&AppVersionInfo) + Send + 'static> {
    version_proxy: AppVersionProxy<HttpHandle>,
    cache_path: PathBuf,
    on_version_update: F,
    last_app_version_info: AppVersionInfo,
    next_update_time: Instant,
    state: VersionUpdaterState,
}

enum VersionUpdaterState {
    Sleeping(tokio_timer::Sleep),
    Updating(Box<dyn Future<Item = AppVersionInfo, Error = Error> + Send + 'static>),
}

impl<F: Fn(&AppVersionInfo) + Send + 'static> VersionUpdater<F> {
    pub fn new(
        rpc_handle: HttpHandle,
        cache_dir: PathBuf,
        on_version_update: F,
        last_app_version_info: AppVersionInfo,
    ) -> Self {
        let version_proxy = AppVersionProxy::new(rpc_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        Self {
            version_proxy,
            cache_path,
            on_version_update,
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
            .app_version_check(&crate::version::PRODUCT_VERSION.to_owned(), PLATFORM)
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
        serde_json::to_writer_pretty(io::BufWriter::new(file), &self.last_app_version_info)
            .map_err(Error::Serialize)
    }
}

impl<F: Fn(&AppVersionInfo) + Send + 'static> Future for VersionUpdater<F> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(new_state) = match &mut self.state {
            VersionUpdaterState::Sleeping(timer) => match timer.poll() {
                Err(e) => {
                    log::error!("Version check sleep error: {}", e);
                    return Err(());
                }
                Ok(Async::NotReady) => None,
                Ok(Async::Ready(())) => Some(if Instant::now() > self.next_update_time {
                    VersionUpdaterState::Updating(self.create_update_future())
                } else {
                    VersionUpdaterState::Sleeping(Self::create_sleep_future())
                }),
            },
            VersionUpdaterState::Updating(future) => match future.poll() {
                Err(error) => {
                    log::error!("{}", error.display_chain_with_msg("Version check failed"));
                    self.next_update_time = Instant::now() + UPDATE_INTERVAL_ERROR;
                    Some(VersionUpdaterState::Sleeping(Self::create_sleep_future()))
                }
                Ok(Async::Ready(app_version_info)) => {
                    if app_version_info != self.last_app_version_info {
                        self.next_update_time = Instant::now() + UPDATE_INTERVAL;
                        log::debug!("Got new version check: {:?}", app_version_info);
                        (self.on_version_update)(&app_version_info);
                        self.last_app_version_info = app_version_info;
                        if let Err(e) = self.write_cache() {
                            log::error!(
                                "{}",
                                e.display_chain_with_msg("Unable to cache version check response")
                            );
                        }
                    }
                    Some(VersionUpdaterState::Sleeping(Self::create_sleep_future()))
                }
                Ok(Async::NotReady) => None,
            },
        } {
            self.state = new_state;
        }
        Ok(Async::NotReady)
    }
}

pub fn load_cache(cache_dir: &Path) -> Result<AppVersionInfo, Error> {
    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());
    let file = File::open(path).map_err(Error::ReadCachedRelays)?;
    serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to open app version cache file for reading")]
    ReadCachedRelays(#[error(cause)] io::Error),

    #[error(display = "Failed to open app version cache file for writing")]
    WriteRelayCache(#[error(cause)] io::Error),

    #[error(display = "Failure in serialization of the version info")]
    Serialize(#[error(cause)] serde_json::Error),

    #[error(display = "Timed out when trying to check the latest app version")]
    DownloadTimeout,

    #[error(display = "Failed to check the latest app version")]
    Download(#[error(cause)] mullvad_rpc::Error),
}

impl<F> From<TimeoutError<F>> for Error {
    fn from(_: TimeoutError<F>) -> Error {
        Error::DownloadTimeout
    }
}
