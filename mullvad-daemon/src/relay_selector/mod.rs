pub mod service;
// TODO: Skip re-exporting every item in service.
pub use service::*;

use std::io;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use mullvad_relay_selector::query::RelayQuery;
use mullvad_relay_selector::{
    CustomListProvider, EntrySpecificConstraints, Error, GetRelay, RETRY_ORDER,
    RelaySelector as Impl,
};
use mullvad_types::custom_list::CustomListsSettings;
use mullvad_types::relay_list::{BridgeList, RelayList};
use mullvad_types::settings::Settings;
use talpid_types::net::IpAvailability;

#[derive(Clone)]
pub struct RelaySelector(Impl<Config>);

impl Deref for RelaySelector {
    type Target = Impl<Config>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait RelaySelectorIO {
    /// Initialize relay selector state from disk, loading any cached relay list.
    fn load() -> io::Result<Box<Self>>;
}

impl RelaySelectorIO for Impl {
    #[cfg(not(test))]
    fn load() -> io::Result<Box<Self>> {
        use crate::relay_list::parsed_relays::parse_relays_from_file;

        let cache_dir = mullvad_paths::get_cache_dir()
            .map_err(|_err| io::Error::other("Missing cache directory"))?;
        let config_dir = mullvad_paths::get_resource_dir();
        // Initialize relay selector asap, since it's a pre-requisite for accepting incoming gRPC
        // connections *and* for the split-filter / multihop migration of 2026. More info on that
        // may be found in [`migrations::multihop`].
        let initial_relay_list = parse_relays_from_file(&cache_dir, &config_dir)
            .inspect_err(|err| log::error!("{err}"))
            .ok();
        let relay_selector = {
            let (initial_relay_list, initial_bridge_list) = initial_relay_list
                .clone()
                .map(mullvad_api::CachedRelayList::into_internal_repr)
                .unwrap_or_default();
            Impl::new(initial_relay_list.clone(), initial_bridge_list.clone())
        };
        Ok(Box::new(relay_selector))
    }
    #[cfg(test)]
    fn load() -> io::Result<Box<Self>> {
        let relay_selector = {
            let (initial_relay_list, initial_bridge_list): (RelayList, BridgeList) =
                Default::default();
            Impl::new(initial_relay_list.clone(), initial_bridge_list.clone())
        };
        Ok(Box::new(relay_selector))
    }
}

/// Relay selector configuration. This datastructure keeps the relay selector in sync with
/// mullvad-daemon.
///
/// Carries the pre-computed [`RelayQuery`] derived from the user's settings together with the
/// custom lists needed for location filtering. When the user has configured a custom tunnel
/// endpoint the relay selector is never queried, so a dormant default config is used.
#[derive(Debug, Clone, Default)]
pub struct Config {
    query: Arc<Mutex<RelayQuery>>,
    custom_lists: Arc<Mutex<CustomListsSettings>>,
}

impl From<Settings> for Config {
    fn from(settings: Settings) -> Self {
        let custom_lists = Arc::new(Mutex::new(settings.custom_lists.clone()));
        let query = Arc::new(Mutex::new(RelayQuery::from(settings)));
        Self {
            query,
            custom_lists,
        }
    }
}

impl From<RelayQuery> for Config {
    fn from(query: RelayQuery) -> Self {
        let query = Arc::new(Mutex::new(query));
        let custom_lists = Arc::new(Mutex::new(CustomListsSettings::default()));
        Config {
            query,
            custom_lists,
        }
    }
}

impl CustomListProvider for Config {
    fn custom_lists(&self) -> CustomListsSettings {
        self.custom_lists.lock().unwrap().clone()
    }
}

impl RelaySelector {
    pub fn create() -> Option<Self> {
        todo!("implement")
    }

    pub fn from_settings(
        settings: Settings,
        relays: RelayList,
        bridges: BridgeList,
    ) -> RelaySelector {
        let config = Config::from(settings);
        let relay_selector = Impl::new(relays, bridges).with_config(config);
        RelaySelector(relay_selector)
    }

    /// Update the relay selector config.
    pub fn set_config(&self, settings: Settings) {
        let config = &self.0.config;
        *config.custom_lists.lock().unwrap() = settings.custom_lists.clone();
        *config.query.lock().unwrap() = RelayQuery::from(settings);
    }

    /// Update only the custom list settings used for location filtering.
    pub fn set_custom_lists(&self, custom_lists: CustomListsSettings) {
        let config = &self.0.config;
        *config.custom_lists.lock().unwrap() = custom_lists;
    }

    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in one of the retry orders while considering the [`Config`].
    pub fn get_relay(
        &self,
        retry_attempt: usize,
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        self.get_relay_with_custom_params(retry_attempt, &RETRY_ORDER, runtime_ip_availability)
    }

    /// Returns a random relay and relay endpoint matching the current constraints defined by
    /// `retry_order` corresponding to `retry_attempt`.
    pub fn get_relay_with_custom_params(
        &self,
        retry_attempt: usize,
        retry_order: &[EntrySpecificConstraints],
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        let mut user_query = self.config.query.lock().unwrap().clone();
        // Runtime parameters may shrink the set of usable IP versions — apply that *before*
        // merging with retry_order so an IPv6-only retry attempt is correctly rejected when only
        // IPv4 is available.
        user_query.apply_ip_availability(runtime_ip_availability)?;
        log::trace!("Merging user preferences {user_query:?} with default retry strategy");

        // Select a relay using the user's preferences merged with the nth compatible retry entry,
        // looping back to the start if necessary.
        let maybe_relay = retry_order
            .iter()
            .filter_map(|retry| user_query.clone().merge_retry(retry.clone()))
            .filter_map(|query| self.get_relay_by_query(query).ok())
            .cycle()
            .nth(retry_attempt);

        match maybe_relay {
            Some(v) => Ok(v),
            // If no retry merged with `user_query` yields a relay, fall back to the user's
            // preferences alone.
            None => self.get_relay_by_query(user_query),
        }
    }
}
