//! Relay list updater

use futures::{
    channel::mpsc,
    future::{Fuse, FusedFuture},
    Future, FutureExt, SinkExt, StreamExt,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::fs::File;

use mullvad_api::{availability::ApiAvailability, rest::MullvadRestHandle, RelayListProxy};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::relay_list::RelayList;
use talpid_future::retry::{retry_future, ExponentialBackoff, Jittered};
use talpid_types::ErrorExt;

/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 15);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);

const DOWNLOAD_RETRY_STRATEGY: Jittered<ExponentialBackoff> = Jittered::jitter(
    ExponentialBackoff::new(Duration::from_secs(16), 8)
        .max_delay(Some(Duration::from_secs(2 * 60 * 60))),
);

/// Where the relay list is cached on disk.
pub(crate) const RELAYS_FILENAME: &str = "relays.json";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Downloader already shut down")]
    DownloaderShutdown,

    #[error("Mullvad relay selector error")]
    RelaySelector(#[from] mullvad_relay_selector::Error),
}

#[derive(Clone)]
pub struct RelayListUpdaterHandle {
    tx: mpsc::Sender<()>,
}

impl RelayListUpdaterHandle {
    pub async fn update(&mut self) {
        if let Err(error) = self
            .tx
            .send(())
            .await
            .map_err(|_| Error::DownloaderShutdown)
        {
            log::error!(
                "{}",
                error.display_chain_with_msg("Unable to send update command to relay list updater")
            );
        }
    }
}

pub struct RelayListUpdater {
    api_client: RelayListProxy,
    cache_path: PathBuf,
    relay_selector: RelaySelector,
    on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
    last_check: SystemTime,
    api_availability: ApiAvailability,
}

impl RelayListUpdater {
    pub fn spawn(
        selector: RelaySelector,
        api_handle: MullvadRestHandle,
        cache_dir: &Path,
        on_update: impl Fn(&RelayList) + Send + 'static,
    ) -> RelayListUpdaterHandle {
        let (tx, cmd_rx) = mpsc::channel(1);
        let api_availability = api_handle.availability.clone();
        let api_client = RelayListProxy::new(api_handle);
        let updater = RelayListUpdater {
            api_client,
            cache_path: cache_dir.join(RELAYS_FILENAME),
            relay_selector: selector,
            on_update: Box::new(on_update),
            last_check: UNIX_EPOCH,
            api_availability,
        };

        tokio::spawn(updater.run(cmd_rx));

        RelayListUpdaterHandle { tx }
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<()>) {
        let mut download_future = Box::pin(Fuse::terminated());
        loop {
            let next_check = tokio::time::sleep(UPDATE_CHECK_INTERVAL).fuse();
            tokio::pin!(next_check);

            futures::select! {
                _check_update = next_check => {
                    if download_future.is_terminated() && self.should_update() {
                        let tag = self.relay_selector.etag();
                        download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(), self.api_client.clone(), tag).fuse());
                        self.last_check = SystemTime::now();
                    }
                },

                new_relay_list = download_future => {
                    self.consume_new_relay_list(new_relay_list).await;
                },

                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(()) => {
                            let tag = self.relay_selector.etag();
                            download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(), self.api_client.clone(), tag).fuse());
                            self.last_check = SystemTime::now();
                        },
                        None => {
                            log::trace!("Relay list updater shutting down");
                            return;
                        }
                    }
                }

            };
        }
    }

    async fn consume_new_relay_list(
        &mut self,
        result: Result<Option<RelayList>, mullvad_api::Error>,
    ) {
        match result {
            Ok(Some(relay_list)) => {
                if let Err(err) = self.update_cache(relay_list).await {
                    log::error!("Failed to update relay list cache: {}", err);
                }
            }
            Ok(None) => log::debug!("Relay list is up-to-date"),
            Err(error) => log::error!(
                "{}",
                error.display_chain_with_msg("Failed to fetch new relay list")
            ),
        }
    }

    /// Returns true if the current parsed_relays is older than UPDATE_INTERVAL
    fn should_update(&mut self) -> bool {
        let last_check = std::cmp::max(self.relay_selector.last_updated(), self.last_check);
        match SystemTime::now().duration_since(last_check) {
            Ok(duration) => duration >= UPDATE_INTERVAL,
            // If the clock is skewed we have no idea by how much or when the last update
            // actually was, better download again to get in sync and get a `last_updated`
            // timestamp corresponding to the new time.
            Err(_) => true,
        }
    }

    fn download_relay_list(
        api_handle: ApiAvailability,
        proxy: RelayListProxy,
        tag: Option<String>,
    ) -> impl Future<Output = Result<Option<RelayList>, mullvad_api::Error>> + use<> {
        async fn download_future(
            api_handle: ApiAvailability,
            proxy: RelayListProxy,
            tag: Option<String>,
        ) -> Result<Option<RelayList>, mullvad_api::Error> {
            let available = api_handle.wait_background();
            let req = proxy.relay_list(tag);
            available.await?;
            req.await.map_err(mullvad_api::Error::from)
        }

        let download_futures =
            move || download_future(api_handle.clone(), proxy.clone(), tag.clone());

        retry_future(
            download_futures,
            |result| result.is_err(),
            DOWNLOAD_RETRY_STRATEGY,
        )
    }

    async fn update_cache(&mut self, new_relay_list: RelayList) -> Result<(), Error> {
        if let Err(error) = Self::cache_relays(&self.cache_path, &new_relay_list).await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay cache on disk")
            );
        }

        self.relay_selector.set_relays(new_relay_list.clone());
        (self.on_update)(&new_relay_list);
        Ok(())
    }

    /// Write a `RelayList` to the cache file.
    async fn cache_relays(cache_path: &Path, relays: &RelayList) -> Result<(), Error> {
        log::debug!("Writing relays cache to {}", cache_path.display());
        let mut file = File::create(cache_path)
            .await
            .map_err(mullvad_relay_selector::Error::OpenRelayCache)?;
        let bytes =
            serde_json::to_vec_pretty(relays).map_err(mullvad_relay_selector::Error::Serialize)?;
        let mut slice: &[u8] = bytes.as_slice();
        let _ = tokio::io::copy(&mut slice, &mut file)
            .await
            .map_err(mullvad_relay_selector::Error::WriteRelayCache)?;
        Ok(())
    }
}
