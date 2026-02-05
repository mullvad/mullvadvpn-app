//! Relay list updater

pub mod error;
pub(crate) mod parsed_relays;

use error::Error;
use mullvad_types::relay_constraints::RelayOverride;

use chrono::{DateTime, Utc};
use futures::channel::mpsc;
use futures::future::{Fuse, FusedFuture};
use futures::{Future, FutureExt, SinkExt, StreamExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs::File;

use crate::sigsum;
use mullvad_api::rest;
use mullvad_api::{
    CachedRelayList, RelayListDigest, RelayListProxy, availability::ApiAvailability,
    rest::MullvadRestHandle,
};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::relay_list::{BridgeList, RelayList};
use talpid_future::retry::{ExponentialBackoff, Jittered, retry_future};
use talpid_types::ErrorExt;

/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_mins(15);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_hours(1);

const DOWNLOAD_RETRY_STRATEGY: Jittered<ExponentialBackoff> = Jittered::jitter(
    ExponentialBackoff::new(Duration::from_secs(16), 8).max_delay(Some(Duration::from_hours(2))),
);

/// Where the relay list is cached on disk.
const RELAYS_FILENAME: &str = "relays.json";

#[derive(Clone)]
pub struct RelayListUpdaterHandle {
    tx: mpsc::Sender<Event>,
}

/// Possible events that occur in the [RelayListUpdater] life cycle.
#[derive(Debug)]
enum Event {
    /// Trigger a relay list refresh.
    Update,
    /// Register new relay IP overrides.
    Override(Vec<RelayOverride>),
}

impl RelayListUpdaterHandle {
    pub async fn update(&mut self) {
        if let Err(error) = self
            .tx
            .send(Event::Update)
            .await
            .map_err(|_| Error::DownloaderShutdown)
        {
            log::error!(
                "{}",
                error.display_chain_with_msg("Unable to send update command to relay list updater")
            );
        }
    }

    /// Update relay overrides.
    pub async fn update_overrides(&mut self, overrides: Vec<RelayOverride>) {
        if let Err(_err) = self.tx.send(Event::Override(overrides)).await {
            log::error!("Failed to apply new relay overrides");
        };
    }
}

pub(crate) struct RelayListUpdater {
    api_client: RelayListProxy,
    cache_path: PathBuf,
    on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
    last_check: SystemTime,
    api_availability: ApiAvailability,
    digest: RelayListDigest,
    latest_timestamp: DateTime<Utc>,
    // Keep tabs on the up-to-date relay list.
    // Use [RelayListUpdater::get_final_relay_list] when exposing the relay list to other parts of
    // the app.
    relay_list: RelayList,
    bridge_list: BridgeList,
    overrides: Vec<RelayOverride>,
    // The relay selector will only ever see the relay list with IP overrides applied.
    relay_selector: RelaySelector,
}

impl RelayListUpdater {
    pub fn spawn(
        selector: RelaySelector,
        api_handle: MullvadRestHandle,
        cache_dir: &Path,
        overrides: Vec<RelayOverride>,
        on_update: impl Fn(&RelayList) + Send + 'static,
        cached_relay_list: Option<CachedRelayList>,
    ) -> RelayListUpdaterHandle {
        let (tx, cmd_rx) = mpsc::channel(1);
        let api_availability = api_handle.availability.clone();
        let api_client = RelayListProxy::new(api_handle);

        let (relay_list, bridge_list, digest, latest_timestamp) = cached_relay_list
            .map(|cached_relay_list| {
                let digest = cached_relay_list.digest().clone();
                let timestamp = cached_relay_list.timestamp();
                let (relay_list, bridge_list) = cached_relay_list.into_internal_repr();
                (relay_list, bridge_list, digest, timestamp)
            })
            .unwrap_or_default();
        let updater = RelayListUpdater {
            api_client,
            cache_path: cache_dir.join(RELAYS_FILENAME),
            relay_selector: selector,
            on_update: Box::new(on_update),
            last_check: UNIX_EPOCH,
            digest,
            latest_timestamp,
            overrides,
            api_availability,
            relay_list,
            bridge_list,
        };

        tokio::spawn(updater.run(cmd_rx));

        RelayListUpdaterHandle { tx }
    }

    async fn run(mut self, mut internal_events: mpsc::Receiver<Event>) {
        let mut download_future = Box::pin(Fuse::terminated());
        loop {
            let next_check = tokio::time::sleep(UPDATE_CHECK_INTERVAL).fuse();
            tokio::pin!(next_check);

            let digest = self.digest.clone();
            let timestamp = self.latest_timestamp;

            futures::select! {
                _check_update = next_check => {
                    log::trace!("Received `next_check` event");
                    if download_future.is_terminated() && self.should_update() {
                        download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(),
                            self.api_client.clone(),
                            digest,
                            timestamp).fuse());

                        self.last_check = SystemTime::now();
                    }
                },

                new_relay_list = download_future => {
                    log::trace!("Finished downloading a new relay list");
                    self.consume_new_relay_list(new_relay_list).await;
                },

                cmd = internal_events.next() => {
                    log::trace!("Received {cmd:#?}");
                    let Some(event) = cmd else {
                            log::trace!("Relay list updater shutting down");
                            return;
                    };
                    match event {
                        Event::Update => {
                            download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(),
                                    self.api_client.clone(),
                                    digest,
                                    timestamp).fuse());

                            self.last_check = SystemTime::now();
                        },
                        // Only update the relay list with new overrides if they are actually new.
                        Event::Override(overrides) if self.overrides != overrides => {
                            self.overrides = overrides;
                            self.update_relay_selector();
                        }
                        Event::Override(overrides) => {
                            log::trace!("New overrides match the old overrides.");
                            log::trace!("{overrides:#?}");
                        }
                    }
                }

            };
        }
    }

    async fn consume_new_relay_list(
        &mut self,
        result: Result<Option<CachedRelayList>, mullvad_api::Error>,
    ) {
        match result {
            Ok(Some(relay_list)) => {
                log::trace!("Updating relay list cache");
                self.update_cache(relay_list).await
            }
            Ok(None) => log::debug!("Relay list is up-to-date"),
            Err(error) => log::error!(
                "{}",
                error.display_chain_with_msg("Failed to fetch new relay list")
            ),
        }
    }

    /// Returns true if the current relay list is older than [`UPDATE_INTERVAL`].
    fn should_update(&mut self) -> bool {
        match SystemTime::now().duration_since(self.last_check) {
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
        latest_digest: RelayListDigest,
        latest_timestamp: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<CachedRelayList>, mullvad_api::Error>> + use<> {
        let download_futures = move || {
            RelayListUpdater::download_and_verify_relay_list_future(
                api_handle.clone(),
                proxy.clone(),
                latest_digest.clone(),
                latest_timestamp,
            )
        };

        retry_future(
            download_futures,
            |result| result.is_err(),
            DOWNLOAD_RETRY_STRATEGY,
        )
    }

    /// Downloads and verifies the transparency logged relay list.
    /// If the verification fails the error is only logged, and a new relay list will still be
    /// fetched and used as long as we are able to parse the digest (which is needed to fetch
    /// the relay list).
    async fn download_and_verify_relay_list_future(
        api_handle: ApiAvailability,
        proxy: RelayListProxy,
        latest_digest: RelayListDigest,
        latest_timestamp: DateTime<Utc>,
    ) -> Result<Option<CachedRelayList>, mullvad_api::Error> {
        api_handle.wait_background().await?;

        // Fetch relay list latest sigsum signature.
        let relay_list_sig = proxy.relay_list_latest_signature().await?;

        // Parse the timestamp from the signature.
        let timestamp = match sigsum::validate_signature(&relay_list_sig) {
            Ok(timestamp) => {
                log::debug!("SIGSUM: Relay list sigsum signature validation succeeded");
                timestamp
            }
            Err(e) => {
                log::error!(
                    "SIGSUM: Relay list sigsum signature validation failed: {}",
                    e.source
                );
                log::debug!("SIGSUM: Attempting to parse unverified timestamp");

                e.timestamp_parser
                    .parse_without_verification()
                    .inspect_err(|_| {
                        log::error!(
                                "SIGSUM: Failed to parse unverified timestamp; aborting relay list update"
                            );
                    })
                    .inspect(|_| log::debug!("SIGSUM: Successfully parsed unverified timestamp"))
                    .map_err(rest::Error::from)?
            }
        };

        // Verify that the timestamp is not too old.
        let new_timestamp = timestamp.timestamp;
        if new_timestamp < (Utc::now() - Duration::from_hours(24)) {
            log::error!("SIGSUM: Relay list timestamp is older than 24 hours: {new_timestamp}",);
        }
        if new_timestamp < latest_timestamp {
            log::error!(
                "SIGSUM: Relay list timestamp is older than current timestamp\n\
                current {latest_timestamp}, new: {new_timestamp}",
            );
        }

        // If the digest has not changed we do not need to fetch the relay list.
        if latest_digest == timestamp.digest {
            log::debug!("SIGSUM: timestamp digest hasn't changed; will not fetch new relay list");
            return Ok(None);
        }

        // Fetch the actual relay list given the timestamp digest.
        let response = proxy
            .relay_list(&timestamp.digest, timestamp.timestamp)
            .await?;

        // Validate that the sigsum digest matches the relay list hash.
        match sigsum::validate_data(&timestamp, response.digest()) {
            Ok(_) => log::debug!("SIGSUM: Relay list sigsum data validation succeeded"),
            Err(e) => log::error!("SIGSUM: Relay list sigsum data validation failed: {}", e),
        }

        Ok(Some(response))
    }

    async fn update_cache(&mut self, new_relay_list: CachedRelayList) {
        // Save the new relay list to the cache file
        if let Err(error) = Self::cache_relays(&self.cache_path, &new_relay_list).await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay cache on disk")
            );
        }
        // Cache the digest so that we can check it before sending next request
        self.digest = new_relay_list.digest().clone();
        // Propagate the new relay list to the relay selector
        let (relay_list, bridge_list) = new_relay_list.into_internal_repr();
        self.relay_list = relay_list;
        self.bridge_list = bridge_list;
        self.update_relay_selector();
    }

    /// Update the relay selector state, applying IP overrides.
    fn update_relay_selector(&self) {
        let relay_list = self.get_final_relay_list();
        let bridge_list = self.bridge_list.clone();
        // Announce new relay list
        self.relay_selector.set_relays(relay_list.clone());
        self.relay_selector.set_bridges(bridge_list);
        // Note: It is important that dependants are updated after relay selector state has been
        // updated, since they might depend on the relay selector's state ..
        (self.on_update)(&relay_list);
    }

    /// Write a [`CachedRelayList`] to the file at `cache_path`.
    async fn cache_relays(cache_path: &Path, relays: &CachedRelayList) -> Result<(), Error> {
        log::debug!("Writing relays cache to {}", cache_path.display());
        let mut file = File::create(cache_path)
            .await
            .map_err(Error::OpenRelayCache)?;
        let bytes = serde_json::to_vec_pretty(relays)?;
        let mut slice: &[u8] = bytes.as_slice();
        let _ = tokio::io::copy(&mut slice, &mut file)
            .await
            .map_err(Error::WriteRelayCache)?;
        Ok(())
    }

    /// Return a version of the [`RelayList`] where [`RelayOverride`]s have been applied.
    fn get_final_relay_list(&self) -> RelayList {
        self.relay_list
            .clone()
            .apply_overrides(self.overrides.clone())
    }
}
