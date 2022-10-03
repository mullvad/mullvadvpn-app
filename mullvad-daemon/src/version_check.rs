use crate::{version::is_beta_version, DaemonEventSender};
use futures::{
    channel::{mpsc, oneshot},
    stream::FusedStream,
    FutureExt, SinkExt, StreamExt, TryFutureExt,
};
use mullvad_api::{availability::ApiAvailabilityHandle, rest::MullvadRestHandle, AppVersionProxy};
use mullvad_types::version::{AppVersionInfo, ParsedAppVersion};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    io,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;
use tokio::fs::{self, File};

const VERSION_INFO_FILENAME: &str = "version-info.json";

lazy_static::lazy_static! {
    static ref APP_VERSION: ParsedAppVersion = ParsedAppVersion::from_str(mullvad_version::VERSION).unwrap();
    static ref IS_DEV_BUILD: bool = APP_VERSION.is_dev();
}

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// Wait this long until next check after a successful check
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);
/// Wait this long until next try if an update failed
const UPDATE_INTERVAL_ERROR: Duration = Duration::from_secs(60 * 60 * 6);
/// Retry interval for `RunVersionCheck`.
const IMMEDIATE_UPDATE_INTERVAL_ERROR: Duration = Duration::ZERO;
const IMMEDIATE_UPDATE_MAX_RETRIES: usize = 2;

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(target_os = "android")]
const PLATFORM: &str = "android";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CachedAppVersionInfo {
    #[serde(flatten)]
    pub version_info: AppVersionInfo,
    pub cached_from_version: String,
}

impl From<AppVersionInfo> for CachedAppVersionInfo {
    fn from(version_info: AppVersionInfo) -> CachedAppVersionInfo {
        CachedAppVersionInfo {
            version_info,
            cached_from_version: mullvad_version::VERSION.to_owned(),
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

    #[error(display = "Failure in deserialization of the version info")]
    Deserialize(#[error(source)] serde_json::Error),

    #[error(display = "Failed to check the latest app version")]
    Download(#[error(source)] mullvad_api::rest::Error),

    #[error(display = "API availability check failed")]
    ApiCheck(#[error(source)] mullvad_api::availability::Error),

    #[error(display = "Clearing version check cache due to a version mismatch")]
    CacheVersionMismatch,

    #[error(display = "Version updater is down")]
    VersionUpdaterDown,

    #[error(display = "Version cache update was aborted")]
    UpdateAborted,
}

pub(crate) struct VersionUpdater {
    version_proxy: AppVersionProxy,
    cache_path: PathBuf,
    update_sender: DaemonEventSender<AppVersionInfo>,
    last_app_version_info: Option<AppVersionInfo>,
    platform_version: String,
    show_beta_releases: bool,
    rx: Option<mpsc::Receiver<VersionUpdaterCommand>>,
    availability_handle: ApiAvailabilityHandle,
    internal_done_tx: Option<oneshot::Sender<AppVersionInfo>>,
}

#[derive(Clone)]
pub(crate) struct VersionUpdaterHandle {
    tx: mpsc::Sender<VersionUpdaterCommand>,
}

enum VersionUpdaterCommand {
    SetShowBetaReleases(bool),
    RunVersionCheck(oneshot::Sender<AppVersionInfo>),
}

impl VersionUpdaterHandle {
    pub async fn set_show_beta_releases(&mut self, show_beta_releases: bool) {
        if self
            .tx
            .send(VersionUpdaterCommand::SetShowBetaReleases(
                show_beta_releases,
            ))
            .await
            .is_err()
        {
            log::error!("Version updater already down, can't send new `show_beta_releases` state");
        }
    }

    pub async fn run_version_check(&mut self) -> Result<AppVersionInfo, Error> {
        let (done_tx, done_rx) = oneshot::channel();
        if self
            .tx
            .send(VersionUpdaterCommand::RunVersionCheck(done_tx))
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
    pub fn new(
        mut api_handle: MullvadRestHandle,
        availability_handle: ApiAvailabilityHandle,
        cache_dir: PathBuf,
        update_sender: DaemonEventSender<AppVersionInfo>,
        last_app_version_info: Option<AppVersionInfo>,
        show_beta_releases: bool,
    ) -> (Self, VersionUpdaterHandle) {
        api_handle.factory.timeout = DOWNLOAD_TIMEOUT;
        let version_proxy = AppVersionProxy::new(api_handle);
        let cache_path = cache_dir.join(VERSION_INFO_FILENAME);
        let (tx, rx) = mpsc::channel(1);
        let platform_version = talpid_platform_metadata::short_version();

        (
            Self {
                version_proxy,
                cache_path,
                update_sender,
                last_app_version_info,
                platform_version,
                show_beta_releases,
                rx: Some(rx),
                availability_handle,
                internal_done_tx: None,
            },
            VersionUpdaterHandle { tx },
        )
    }

    fn create_update_future(
        &mut self,
        done_tx: oneshot::Sender<AppVersionInfo>,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<mullvad_api::AppVersionResponse, Error>> + Send + 'static>,
    > {
        self.internal_done_tx = Some(done_tx);

        let api_handle = self.availability_handle.clone();
        let version_proxy = self.version_proxy.clone();
        let platform_version = self.platform_version.clone();
        let download_future_factory = move || {
            version_proxy
                .version_check(
                    mullvad_version::VERSION.to_owned(),
                    PLATFORM,
                    platform_version.clone(),
                )
                .map_err(Error::Download)
        };

        Box::pin(talpid_core::future_retry::retry_future_n(
            download_future_factory,
            move |result| Self::should_retry_immediate(result, &api_handle),
            std::iter::repeat(IMMEDIATE_UPDATE_INTERVAL_ERROR),
            IMMEDIATE_UPDATE_MAX_RETRIES,
        ))
    }

    fn should_retry_immediate<T>(
        result: &Result<T, Error>,
        api_handle: &ApiAvailabilityHandle,
    ) -> bool {
        match result {
            Err(Error::Download(error)) if error.is_network_error() => {
                !api_handle.get_state().is_offline()
            }
            _ => false,
        }
    }

    fn create_update_background_future(
        &self,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<mullvad_api::AppVersionResponse, Error>> + Send + 'static>,
    > {
        let api_handle = self.availability_handle.clone();
        let version_proxy = self.version_proxy.clone();
        let platform_version = self.platform_version.clone();
        let download_future_factory = move || {
            let when_available = api_handle.wait_background();
            let request = version_proxy.version_check(
                mullvad_version::VERSION.to_owned(),
                PLATFORM,
                platform_version.clone(),
            );
            async move {
                when_available.await.map_err(Error::ApiCheck)?;
                request.await.map_err(Error::Download)
            }
        };

        Box::pin(talpid_core::future_retry::retry_future(
            download_future_factory,
            |result| result.is_err(),
            std::iter::repeat(UPDATE_INTERVAL_ERROR),
        ))
    }

    async fn write_cache(&self) -> Result<(), Error> {
        let last_app_version_info = match self.last_app_version_info.as_ref() {
            Some(version_info) => version_info,
            None => {
                log::debug!("The version cache is empty -- not writing");
                return Ok(());
            }
        };
        log::debug!(
            "Writing version check cache to {}",
            self.cache_path.display()
        );
        let mut file = File::create(&self.cache_path)
            .await
            .map_err(Error::WriteVersionCache)?;
        let cached_app_version = CachedAppVersionInfo::from(last_app_version_info.clone());
        let mut buf = serde_json::to_vec_pretty(&cached_app_version).map_err(Error::Serialize)?;
        let mut read_buf: &[u8] = buf.as_mut();

        let _ = tokio::io::copy(&mut read_buf, &mut file)
            .await
            .map_err(Error::WriteVersionCache)?;
        Ok(())
    }

    fn response_to_version_info(
        &mut self,
        response: mullvad_api::AppVersionResponse,
    ) -> AppVersionInfo {
        let suggested_upgrade = Self::suggested_upgrade(
            &APP_VERSION,
            &response.latest_stable,
            &response.latest_beta,
            self.show_beta_releases || is_beta_version(),
        );

        let wg_migration_threshold = if response.x_threshold_wg_default.is_nan() {
            // If the value should for some strange reason be NaN then safe default to 0.0
            0.0
        } else {
            // Make sure that the returned value is between 0% and 100%
            response.x_threshold_wg_default.clamp(0.0, 1.0)
        };

        AppVersionInfo {
            supported: response.supported,
            latest_stable: response.latest_stable.unwrap_or_else(|| "".to_owned()),
            latest_beta: response.latest_beta,
            suggested_upgrade,
            wg_migration_threshold,
        }
    }

    fn suggested_upgrade(
        current_version: &ParsedAppVersion,
        latest_stable: &Option<String>,
        latest_beta: &str,
        show_beta: bool,
    ) -> Option<String> {
        let stable_version = latest_stable
            .as_ref()
            .and_then(|stable| ParsedAppVersion::from_str(stable).ok());

        let beta_version = if show_beta {
            ParsedAppVersion::from_str(latest_beta).ok()
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

    async fn update_version_info(&mut self, new_version_info: AppVersionInfo) {
        if let Some(done_tx) = self.internal_done_tx.take() {
            let _ = done_tx.send(new_version_info.clone());
        }

        // if daemon can't be reached, return immediately
        if self.update_sender.send(new_version_info.clone()).is_err() {
            return;
        }

        self.last_app_version_info = Some(new_version_info);
        if let Err(err) = self.write_cache().await {
            log::error!("Failed to save version cache to disk: {}", err);
        }
    }

    pub async fn run(mut self) {
        let mut rx = self.rx.take().unwrap().fuse();
        let next_delay = || Box::pin(talpid_time::sleep(UPDATE_INTERVAL)).fuse();
        let mut check_delay = next_delay();
        let mut version_check = futures::future::Fuse::terminated();

        // If this is a dev build, there's no need to pester the API for version checks.
        if *IS_DEV_BUILD {
            log::warn!("Not checking for updates because this is a development build");
            while let Some(cmd) = rx.next().await {
                if let VersionUpdaterCommand::RunVersionCheck(done_tx) = cmd {
                    log::info!("Version check is disabled in dev builds");
                    let _ = done_tx.send(dev_version_cache());
                }
            }
            return;
        }

        loop {
            futures::select! {
                command = rx.next() => {
                    match command {
                        Some(VersionUpdaterCommand::SetShowBetaReleases(show_beta_releases)) => {
                            self.show_beta_releases = show_beta_releases;

                            if let Some(last_app_version_info) = self
                                .last_app_version_info
                                .clone()
                            {
                                let suggested_upgrade = Self::suggested_upgrade(
                                    &APP_VERSION,
                                    &Some(last_app_version_info.latest_stable.clone()),
                                    &last_app_version_info.latest_beta,
                                    self.show_beta_releases || is_beta_version(),
                                );

                                self.update_version_info(AppVersionInfo {
                                    supported: last_app_version_info.supported,
                                    latest_stable: last_app_version_info.latest_stable,
                                    latest_beta: last_app_version_info.latest_beta,
                                    suggested_upgrade,
                                    wg_migration_threshold: last_app_version_info.wg_migration_threshold,
                                }).await;
                            }
                        }
                        Some(VersionUpdaterCommand::RunVersionCheck(done_tx)) => {
                            if self.update_sender.is_closed() {
                                return;
                            }
                            let download_future = self.create_update_future(done_tx).fuse();
                            version_check = download_future;
                        }
                        // time to shut down
                        None => {
                            return;
                        }
                    }
                },

                _sleep = check_delay => {
                    if rx.is_terminated() || self.update_sender.is_closed() {
                        return;
                    }
                    if self.internal_done_tx.is_some() {
                        // Sync check in progress
                        continue;
                    }
                    version_check = self.create_update_background_future().fuse();
                },

                response = version_check => {
                    if rx.is_terminated() || self.update_sender.is_closed() {
                        return;
                    }

                    match response {
                        Ok(version_info_response) => {
                            let new_version_info =
                                self.response_to_version_info(version_info_response);
                            self.update_version_info(new_version_info).await;
                        },
                        Err(err) => {
                            log::error!("Failed to fetch version info: {}", err);
                            self.internal_done_tx = None;
                        },
                    }

                    check_delay = next_delay();
                },
            }
        }
    }
}

async fn try_load_cache(cache_dir: &Path) -> Result<AppVersionInfo, Error> {
    if *IS_DEV_BUILD {
        return Ok(dev_version_cache());
    }

    let path = cache_dir.join(VERSION_INFO_FILENAME);
    log::debug!("Loading version check cache from {}", path.display());
    let content = fs::read_to_string(&path)
        .map_err(Error::ReadVersionCache)
        .await?;
    let version_info: CachedAppVersionInfo =
        serde_json::from_str(&content).map_err(Error::Deserialize)?;

    if version_info.cached_from_version == mullvad_version::VERSION {
        Ok(version_info.version_info)
    } else {
        Err(Error::CacheVersionMismatch)
    }
}

pub async fn load_cache(cache_dir: &Path) -> Option<AppVersionInfo> {
    match try_load_cache(cache_dir).await {
        Ok(app_version_info) => Some(app_version_info),
        Err(error) => {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Unable to load cached version info")
            );
            None
        }
    }
}

fn dev_version_cache() -> AppVersionInfo {
    assert!(*IS_DEV_BUILD);

    AppVersionInfo {
        supported: false,
        latest_stable: mullvad_version::VERSION.to_owned(),
        latest_beta: mullvad_version::VERSION.to_owned(),
        suggested_upgrade: None,
        // Use WireGuard on 75% of dev builds. So we can manually modify
        // wg_migration_rand_num in the settings and verify that the migration
        // works as expected.
        wg_migration_threshold: 0.75,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_upgrade_suggestions() {
        let latest_stable = Some("2020.4".to_string());
        let latest_beta = "2020.5-beta3";

        let older_stable = ParsedAppVersion::from_str("2020.3").unwrap();
        let current_stable = ParsedAppVersion::from_str("2020.4").unwrap();
        let newer_stable = ParsedAppVersion::from_str("2021.5").unwrap();

        let older_beta = ParsedAppVersion::from_str("2020.3-beta3").unwrap();
        let current_beta = ParsedAppVersion::from_str("2020.5-beta3").unwrap();
        let newer_beta = ParsedAppVersion::from_str("2021.5-beta3").unwrap();

        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_stable, &latest_stable, latest_beta, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_stable, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_stable, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_stable, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_stable, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_stable, &latest_stable, latest_beta, true),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_beta, &latest_stable, latest_beta, false),
            Some("2020.4".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&older_beta, &latest_stable, latest_beta, true),
            Some("2020.5-beta3".to_owned())
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_beta, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&current_beta, &latest_stable, latest_beta, true),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_beta, &latest_stable, latest_beta, false),
            None
        );
        assert_eq!(
            VersionUpdater::suggested_upgrade(&newer_beta, &latest_stable, latest_beta, true),
            None
        );
    }
}
