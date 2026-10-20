pub mod grpc_service;

use std::iter;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use mullvad_relay_selector::query::RelayQuery;
use mullvad_relay_selector::{GetRelay, RelaySelector};
use mullvad_types::custom_list::CustomListsSettings;
use mullvad_types::relay_list::{BridgeList, RelayList};
use mullvad_types::settings::Settings;
use talpid_types::net::obfuscation::{ObfuscatorConfig, Obfuscators};

use crate::relay_list;

/// A [RelaySelector] instance backed by a relay list on-disk.
///
/// The queries run against this relay selector is automatically derived from the mullvad-daemon settings.
/// This underpins the [RETRY_ORDER] mechanism where different queries may be run for consecutive
/// connection attempts. See [Config] for details.
#[derive(Clone)]
pub struct RelaySelectorIO {
    inner: RelaySelector,
    config: Config,
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
        let config = Config::from(custom_lists);
        RelaySelectorIO { inner, config }
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
        let config = Config::from(custom_lists);
        Ok(RelaySelectorIO { inner, config })
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
        let config = Config::from(settings);
        let inner = RelaySelector::new(relays, bridges);
        RelaySelectorIO { inner, config }
    }

    /// Update the relay selector config.
    pub fn set_config(&self, settings: Settings) {
        let config = &self.config;
        *config.custom_lists.lock().unwrap() = settings.custom_lists.clone();
        *config.query.lock().unwrap() = RelayQuery::from(settings);
    }

    pub fn query(&self) -> impl Deref<Target = RelayQuery> {
        self.config.query.lock().unwrap()
    }

    /// Update only the custom list settings used for location filtering.
    pub fn set_custom_lists(&self, custom_lists: CustomListsSettings) {
        let config = &self.config;
        *config.custom_lists.lock().unwrap() = custom_lists;
    }

    /// Return a list of next relays to connect to, in order. The list has multiple entries if
    /// obfuscation is used, indicating the relays which to connect to in order of preference.
    /// The list is empty if no relays match the query.
    pub fn get_next_relays(&self, mut user_query: RelayQuery) -> Vec<GetRelay> {
        let mut relays = vec![];

        // First, we make a WireGuard-only attempt before using any obfuscations
        if let Some(wg_query) = user_query.clone().wireguard_only()
            && let Ok(wg_relay) = self.get_relay_by_query(&wg_query)
        {
            // TODO: We could remove `direct` from the user query here to be more consistent
            // There might be some benefit to including it in the obfuscation attempt, though.
            relays.push(wg_relay);
        }

        // Then, append relays until we have covered all obfuscation methods included in the user query
        while let Ok(relay) = self.get_relay_by_query(&user_query) {
            remove_supported_obfuscations(&mut user_query, &relay);
            relays.push(relay);
        }

        relays
    }
}

/// Remove the obfuscation methods that the selected relay offers from the query.
fn remove_supported_obfuscations(user_query: &mut RelayQuery, relay: &GetRelay) {
    let (direct, obf_configs) = match &relay.obfuscator {
        None => (true, vec![]),
        Some(Obfuscators::Multiplexer {
            direct,
            configs: (first, rest),
        }) => (
            direct.is_some(),
            iter::once(first).chain(rest.iter()).collect(),
        ),
        Some(Obfuscators::Single(conf)) => (false, vec![conf]),
    };
    let obfuscation = &mut user_query.entry_specific_mut().obfuscation;
    if direct {
        obfuscation.direct = None;
    }
    for config in obf_configs {
        match config {
            ObfuscatorConfig::Udp2Tcp { .. } => obfuscation.udp2tcp = None,
            ObfuscatorConfig::Shadowsocks { .. } => obfuscation.shadowsocks = None,
            ObfuscatorConfig::Quic { .. } => obfuscation.quic = false,
            ObfuscatorConfig::Lwo { .. } => obfuscation.lwo = None,
        }
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

// TODO: Add tests
// Test that `get_next_relays` interacts correctly with `when needed`:
// All obfuscations reachable with singlehop should be used before multihop is used
