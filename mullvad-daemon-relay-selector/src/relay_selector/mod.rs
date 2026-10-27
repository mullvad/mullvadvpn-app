pub mod grpc_service;

use std::{
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};

use mullvad_relay_selector::{GetRelay, RelayQueue, RelaySelector, query::RelayQuery};
use mullvad_types::{
    custom_list::CustomListsSettings,
    relay_list::{BridgeList, RelayList},
    settings::Settings,
};
use talpid_types::net::IpAvailability;
use talpid_types::net::obfuscation::TransportConfig;
use talpid_types::net::obfuscation::{ObfuscatorConfig, Obfuscators};

use crate::relay_list;

/// A [RelaySelector] instance backed by a relay list on-disk.
///
/// The queries run against this relay selector is automatically derived from the mullvad-daemon
/// settings. See [Config] for details.
#[derive(Clone)]
pub struct RelaySelectorIO {
    inner: RelaySelector,
    config: Config,
    relay_queue: Arc<Mutex<RelayQueue>>,
}

impl Deref for RelaySelectorIO {
    type Target = RelaySelector;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RelaySelectorIO {
    /// Create a new [RelaySelectorIO] with an empty relay list.
    pub fn new(custom_lists: CustomListsSettings) -> Self {
        let inner = {
            let (initial_relay_list, initial_bridge_list): (RelayList, BridgeList) =
                Default::default();
            RelaySelector::new(initial_relay_list.clone(), initial_bridge_list.clone())
        };
        RelaySelectorIO {
            inner,
            config: Config::from(custom_lists),
            relay_queue: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Try to initialize [RelaySelectorIO] from cached relay list.
    pub fn load(
        custom_lists: CustomListsSettings,
        cache_dir: impl AsRef<Path>,
        resource_dir: impl AsRef<Path>,
    ) -> Result<Self, relay_list::error::Error> {
        use crate::relay_list::parsed_relays::parse_relays_from_file;
        let initial_relay_list = parse_relays_from_file(cache_dir, resource_dir)
            .inspect_err(|err| log::error!("{err}"))?;
        let inner = {
            let (initial_relay_list, initial_bridge_list) = initial_relay_list.into_internal_repr();
            RelaySelector::new(initial_relay_list.clone(), initial_bridge_list.clone())
        };
        Ok(RelaySelectorIO {
            inner,
            config: Config::from(custom_lists),
            relay_queue: Arc::new(Mutex::new(Default::default())),
        })
    }

    /// Try to initialize [RelaySelectorIO] from cached relay list. If that fails, fall back to
    /// initializing a [RelaySelectorIO] with an empty relay list.
    pub fn load_with_default(
        custom_lists: CustomListsSettings,
        cache_dir: impl AsRef<Path>,
        resource_dir: impl AsRef<Path>,
    ) -> Self {
        Self::load(custom_lists.clone(), cache_dir, resource_dir)
            .unwrap_or_else(|_| Self::new(custom_lists))
    }

    pub fn from_settings(
        settings: Settings,
        relays: RelayList,
        bridges: BridgeList,
    ) -> RelaySelectorIO {
        RelaySelectorIO {
            inner: RelaySelector::new(relays, bridges),
            config: Config::from(settings),
            relay_queue: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Update the available IP versions. If an IP version is lost, reset the relay queue.
    ///
    /// NOTE: Because IP availability is derived differently on Android during disconnected
    /// and connected states, the IP availability sometimes changes while consuming the
    /// relay queue. If we were to reset the queue when it changed from `Ipv4` to
    /// `Ipv4AndIpv6` it would produce two "direct" connections in a row. The drawback is
    /// that we would not use IPv6 endpoints for obfuscation until the next round.
    pub fn set_ip_availability(&self, ip_availability: IpAvailability) {
        let mut lock = self.config.ip_availability.lock().unwrap();
        let lost_ipv4 = lock.has_ipv4() && !ip_availability.has_ipv4();
        let lost_ipv6 = lock.has_ipv6() && !ip_availability.has_ipv6();
        *lock = ip_availability;

        if lost_ipv4 || lost_ipv6 {
            self.clear_relay_queue();
        }
    }

    /// Update the relay selector config.
    pub fn set_config(&self, settings: Settings) {
        let config = &self.config;
        *config.custom_lists.lock().unwrap() = settings.custom_lists.clone();
        *config.query.lock().unwrap() = RelayQuery::from(settings);
        self.clear_relay_queue();
    }

    pub fn query(&self) -> impl Deref<Target = RelayQuery> {
        self.config.query.lock().unwrap()
    }

    /// Update only the custom list settings used for location filtering.
    pub fn set_custom_lists(&self, custom_lists: CustomListsSettings) {
        let config = &self.config;
        *config.custom_lists.lock().unwrap() = custom_lists;
        self.clear_relay_queue();
    }

    pub fn clear_relay_queue(&self) {
        *self.relay_queue.lock().unwrap() = Default::default();
    }

    /// Get the next relay to use for a connection attempt, based on the user settings.
    ///
    /// The first relay will have a WireGuard-only connection if available, followed by
    /// relays that use the obfuscation methods in the user query, see
    /// [`RelaySelector::get_relays_with_automatic_anti_censorship`].
    ///
    /// The internal queue is automatically reset when exhausted, when the user settings
    /// or the IP availability changes, or when the user calls [RelaySelectorIO::clear_relay_queue].
    pub fn next_relay(&self) -> Result<GetRelay, mullvad_relay_selector::Error> {
        if let Some(relay) = self.relay_queue.lock().unwrap().next() {
            return Ok(relay);
        }
        // The relay queue is empty, so we need to refill it.
        let mut query = self.query().clone();
        query.apply_ip_availability(*self.config.ip_availability.lock().unwrap())?;
        let (next_relay, relay_queue) = self.get_relays_with_automatic_anti_censorship(query)?;
        *self.relay_queue.lock().unwrap() = relay_queue;
        Ok(next_relay)
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
    ip_availability: Arc<Mutex<IpAvailability>>,
}

impl Config {
    fn custom_lists(&self) -> CustomListsSettings {
        self.custom_lists.lock().unwrap().clone()
    }
}

impl From<CustomListsSettings> for Config {
    fn from(custom_lists: CustomListsSettings) -> Self {
        Self {
            query: Default::default(),
            custom_lists: Arc::new(Mutex::new(custom_lists)),
            ip_availability: Arc::new(Mutex::new(IpAvailability::default())),
        }
    }
}

impl From<Settings> for Config {
    fn from(settings: Settings) -> Self {
        let custom_lists = Arc::new(Mutex::new(settings.custom_lists.clone()));
        let query = Arc::new(Mutex::new(RelayQuery::from(settings)));
        Self {
            query,
            custom_lists,
            ip_availability: Arc::new(Mutex::new(IpAvailability::default())),
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
            ip_availability: Arc::new(Mutex::new(IpAvailability::default())),
        }
    }
}
