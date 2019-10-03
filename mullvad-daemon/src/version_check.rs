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
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 5);
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(target_os = "android")]
const PLATFORM: &str = "android";

pub struct VersionUpdater<F: Fn(&AppVersionInfo) + Send + 'static> {
    version: String,
    version_proxy: AppVersionProxy<HttpHandle>,
    cache_dir: PathBuf,
    on_version_update: F,
    last_app_version_info: AppVersionInfo,
    next_update_time: Instant,
    state: Option<VersionUpdaterState>,
}

enum VersionUpdaterState {
    Sleeping(tokio_timer::Sleep),
    Updating(Box<dyn Future<Item = AppVersionInfo, Error = Error> + Send + 'static>),
}

impl<F: Fn(&AppVersionInfo) + Send + 'static> VersionUpdater<F> {
    pub fn new(
        version: String,
        rpc_handle: HttpHandle,
        cache_dir: PathBuf,
        on_version_update: F,
        last_app_version_info: AppVersionInfo,
    ) -> Self {
        let version_proxy = AppVersionProxy::new(rpc_handle);
        Self {
            version,
            version_proxy,
            cache_dir,
            on_version_update,
            last_app_version_info,
            next_update_time: Instant::now(),
            state: Some(VersionUpdaterState::Sleeping(Self::create_sleep_future())),
        }
    }

    fn poll_sleep(&mut self, timer: &mut tokio_timer::Sleep) -> Option<VersionUpdaterState> {
        let should_progress = match timer.poll() {
            Err(e) => {
                log::error!("Version check sleep error: {}", e);
                true
            }
            Ok(Async::Ready(())) => true,
            Ok(Async::NotReady) => false,
        };
        if should_progress {
            let now = Instant::now();
            Some(if now > self.next_update_time {
                self.next_update_time = now + UPDATE_INTERVAL;
                VersionUpdaterState::Updating(self.create_update_future())
            } else {
                VersionUpdaterState::Sleeping(Self::create_sleep_future())
            })
        } else {
            None
        }
    }

    fn poll_updater(
        &mut self,
        future: &mut Box<dyn Future<Item = AppVersionInfo, Error = Error> + Send + 'static>,
    ) -> Option<VersionUpdaterState> {
        let should_progress = match future.poll() {
            Err(error) => {
                log::error!("{}", error.display_chain_with_msg("Version check failed"));
                true
            }
            Ok(Async::Ready(app_version_info)) => {
                if app_version_info != self.last_app_version_info {
                    log::debug!("Got new version check: {:?}", app_version_info);
                    write_cache(&app_version_info, &self.cache_dir).unwrap();
                    (self.on_version_update)(&app_version_info);
                    self.last_app_version_info = app_version_info;
                }
                true
            }
            Ok(Async::NotReady) => false,
        };
        if should_progress {
            Some(VersionUpdaterState::Sleeping(Self::create_sleep_future()))
        } else {
            None
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
            .app_version_check(&self.version, PLATFORM)
            .map_err(Error::Download);
        let future = Timer::default().timeout(download_future, DOWNLOAD_TIMEOUT);
        Box::new(future)
    }
}

impl<F: Fn(&AppVersionInfo) + Send + 'static> Future for VersionUpdater<F> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut state = self.state.take().expect("No state in VersionUpdater");
        while let Some(new_state) = match &mut state {
            VersionUpdaterState::Sleeping(sleep) => self.poll_sleep(sleep),
            VersionUpdaterState::Updating(future) => self.poll_updater(future),
        } {
            state = new_state;
        }
        self.state = Some(state);
        Ok(Async::NotReady)
    }
}

pub fn load_cache(cache_dir: &Path) -> Result<AppVersionInfo, Error> {
    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());
    let file = File::open(path).map_err(Error::ReadCachedRelays)?;
    serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)
}

fn write_cache(app_version_info: &AppVersionInfo, cache_dir: &Path) -> Result<(), Error> {
    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Writing version check cache to {}", path.display());
    let file = File::create(path).map_err(Error::WriteRelayCache)?;
    serde_json::to_writer_pretty(io::BufWriter::new(file), app_version_info)
        .map_err(Error::Serialize)
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
