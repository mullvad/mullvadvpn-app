use super::{Error, ParsedRelays};
use futures::{
    channel::mpsc,
    future::{Fuse, FusedFuture},
    Future, FutureExt, SinkExt, StreamExt,
};
use mullvad_rpc::{availability::ApiAvailabilityHandle, rest::MullvadRestHandle, RelayListProxy};
use mullvad_types::relay_list::RelayList;
use parking_lot::Mutex;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use talpid_core::future_retry::{retry_future, ExponentialBackoff, Jittered};
use talpid_types::ErrorExt;
use tokio::fs::File;

/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 15);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);

const EXPONENTIAL_BACKOFF_INITIAL: Duration = Duration::from_secs(16);
const EXPONENTIAL_BACKOFF_FACTOR: u32 = 8;

#[derive(Clone)]
pub struct RelayListUpdaterHandle {
    tx: mpsc::Sender<()>,
}

impl RelayListUpdaterHandle {
    pub async fn update_relay_list(&mut self) -> Result<(), Error> {
        self.tx
            .send(())
            .await
            .map_err(|_| Error::DownloaderShutDown)
    }
}

pub struct RelayListUpdater {
    rpc_client: RelayListProxy,
    cache_path: PathBuf,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
    earliest_next_try: Instant,
    api_availability: ApiAvailabilityHandle,
}

impl RelayListUpdater {
    pub(super) fn new(
        rpc_handle: MullvadRestHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
        on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
        api_availability: ApiAvailabilityHandle,
    ) -> RelayListUpdaterHandle {
        let (tx, cmd_rx) = mpsc::channel(1);
        let rpc_client = RelayListProxy::new(rpc_handle);
        let updater = RelayListUpdater {
            rpc_client,
            cache_path,
            parsed_relays,
            on_update,
            earliest_next_try: Instant::now() + UPDATE_INTERVAL,
            api_availability,
        };

        tokio::spawn(updater.run(cmd_rx));

        RelayListUpdaterHandle { tx }
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<()>) {
        let mut check_interval = tokio::time::interval_at(
            (Instant::now() + UPDATE_CHECK_INTERVAL).into(),
            UPDATE_CHECK_INTERVAL,
        );
        check_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut ticker = tokio_stream::wrappers::IntervalStream::new(check_interval).fuse();
        let mut download_future = Box::pin(Fuse::terminated());
        loop {
            futures::select! {
                _check_update = ticker.select_next_some() => {
                    if download_future.is_terminated() && self.should_update() {
                        let tag = self.parsed_relays.lock().tag().map(|tag| tag.to_string());
                        download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(), self.rpc_client.clone(), tag).fuse());
                        self.earliest_next_try = Instant::now() + UPDATE_INTERVAL;
                    }
                },

                new_relay_list = download_future => {
                    self.consume_new_relay_list(new_relay_list).await;
                },

                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(()) => {
                            let tag = self.parsed_relays.lock().tag().map(|tag| tag.to_string());
                            download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(), self.rpc_client.clone(), tag).fuse());
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
        result: Result<Option<RelayList>, mullvad_rpc::Error>,
    ) {
        match result {
            Ok(Some(relay_list)) => {
                if let Err(err) = self.update_cache(relay_list).await {
                    log::error!("Failed to update relay list cache: {}", err);
                }
            }
            Ok(None) => log::debug!("Relay list is up-to-date"),
            Err(err) => {
                log::error!(
                    "Failed to fetch new relay list: {}. Will retry in {} minutes",
                    err,
                    self.earliest_next_try
                        .saturating_duration_since(Instant::now())
                        .as_secs()
                        / 60
                );
            }
        }
    }

    /// Returns true if the current parsed_relays is older than UPDATE_INTERVAL
    fn should_update(&mut self) -> bool {
        match SystemTime::now().duration_since(self.parsed_relays.lock().last_updated()) {
            Ok(duration) => duration > UPDATE_INTERVAL && self.earliest_next_try <= Instant::now(),
            // If the clock is skewed we have no idea by how much or when the last update
            // actually was, better download again to get in sync and get a `last_updated`
            // timestamp corresponding to the new time.
            Err(_) => true,
        }
    }

    fn download_relay_list(
        api_handle: ApiAvailabilityHandle,
        rpc_handle: RelayListProxy,
        tag: Option<String>,
    ) -> impl Future<Output = Result<Option<RelayList>, mullvad_rpc::Error>> + 'static {
        let download_futures = move || {
            let available = api_handle.wait_background();
            let req = rpc_handle.relay_list(tag.clone());
            async move {
                available.await?;
                req.await.map_err(mullvad_rpc::Error::from)
            }
        };

        let exponential_backoff =
            ExponentialBackoff::new(EXPONENTIAL_BACKOFF_INITIAL, EXPONENTIAL_BACKOFF_FACTOR)
                .max_delay(UPDATE_INTERVAL * 2);

        let download_future = retry_future(
            download_futures,
            |result| result.is_err(),
            Jittered::jitter(exponential_backoff),
        );
        download_future
    }

    async fn update_cache(&mut self, new_relay_list: RelayList) -> Result<(), Error> {
        if let Err(error) = Self::cache_relays(&self.cache_path, &new_relay_list).await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay cache on disk")
            );
        }

        let new_parsed_relays = ParsedRelays::from_relay_list(new_relay_list, SystemTime::now());
        log::info!(
            "Downloaded relay inventory has {} relays",
            new_parsed_relays.relays().len()
        );

        let mut parsed_relays = self.parsed_relays.lock();
        *parsed_relays = new_parsed_relays;
        (self.on_update)(parsed_relays.locations());
        Ok(())
    }

    /// Write a `RelayList` to the cache file.
    async fn cache_relays(cache_path: &Path, relays: &RelayList) -> Result<(), Error> {
        log::debug!("Writing relays cache to {}", cache_path.display());
        let mut file = File::create(cache_path)
            .await
            .map_err(Error::OpenRelayCache)?;
        let bytes = serde_json::to_vec_pretty(relays).map_err(Error::Serialize)?;
        let mut slice: &[u8] = bytes.as_slice();
        let _ = tokio::io::copy(&mut slice, &mut file)
            .await
            .map_err(Error::WriteRelayCache)?;
        Ok(())
    }
}
