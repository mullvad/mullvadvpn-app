pub mod grpc_service;

use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use mullvad_relay_selector::query::{Hops, RelayQuery, obfuscation_constraint_from_settings};
use mullvad_relay_selector::{EntrySpecificConstraints, Error, GetRelay, RelaySelector};
use mullvad_types::custom_list::CustomListsSettings;
use mullvad_types::relay_constraints::{ObfuscationSettings, SelectedObfuscation};
use mullvad_types::relay_list::{BridgeList, RelayList};
use mullvad_types::settings::Settings;
use mullvad_types::wireguard::TunnelOptions;
use talpid_types::net::obfuscation::ObfuscatorConfig;
use talpid_types::net::obfuscation::Obfuscators;
use talpid_types::net::wireguard::use_userspace_wg;

use crate::relay_list;

/// The kind of an obfuscation method.
///
/// Differs from [`SelectedObfuscation`], which contains `Off` and `Auto`.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ObfuscationMethodKind {
    Udp2Tcp,
    Shadowsocks,
    Quic,
    Lwo,
}

impl ObfuscationMethodKind {
    fn to_constraint(self) -> EntrySpecificConstraints {
        EntrySpecificConstraints {
            obfuscation: obfuscation_constraint_from_settings(ObfuscationSettings {
                selected_obfuscation: match self {
                    ObfuscationMethodKind::Udp2Tcp => SelectedObfuscation::Udp2Tcp,
                    ObfuscationMethodKind::Shadowsocks => SelectedObfuscation::Shadowsocks,
                    ObfuscationMethodKind::Quic => SelectedObfuscation::Quic,
                    ObfuscationMethodKind::Lwo => SelectedObfuscation::Lwo,
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<&ObfuscatorConfig> for ObfuscationMethodKind {
    fn from(config: &ObfuscatorConfig) -> Self {
        match config {
            ObfuscatorConfig::Udp2Tcp { .. } => ObfuscationMethodKind::Udp2Tcp,
            ObfuscatorConfig::Shadowsocks { .. } => ObfuscationMethodKind::Shadowsocks,
            ObfuscatorConfig::Quic { .. } => ObfuscationMethodKind::Quic,
            ObfuscatorConfig::Lwo { .. } => ObfuscationMethodKind::Lwo,
        }
    }
}

/// The obfuscation methods tried, in order, by the staggered obfuscation retry strategy.
///
/// Each connection attempt selects a relay supporting at least one of the still-pending
/// methods, then removes every method that relay offers from the pending set. Once the set
/// is empty, a new round starts. This guarantees that every method is attempted at least
/// once per round, in a number of retries bounded by the number of methods.
pub const RETRY_ORDER: &[ObfuscationMethodKind] = &[
    ObfuscationMethodKind::Lwo,
    ObfuscationMethodKind::Shadowsocks,
    ObfuscationMethodKind::Quic,
    ObfuscationMethodKind::Udp2Tcp,
];

/// The staggered obfuscation retry round, see [`RETRY_ORDER`].
#[derive(Debug, Default)]
pub struct ObfuscationRound {
    /// Obfuscation methods not yet attempted in this round.
    pending: Vec<(ObfuscationMethodKind, RelayQuery)>,
}

impl ObfuscationRound {
    pub fn new(query: RelayQuery) -> Self {
        Self {
            pending: RETRY_ORDER
                .iter()
                .filter_map(|obf| {
                    query
                        .clone()
                        .merge_retry(obf.to_constraint())
                        .map(|obf_query| (*obf, obf_query))
                })
                .collect(),
        }
    }

    /// Remove every method the selected relay's multiplexer offers from the pending set, since
    /// they have all been attempted.
    fn remove_offered(&mut self, offered: &std::collections::HashSet<ObfuscationMethodKind>) {
        self.pending.retain(|pending| !offered.contains(&pending.0));
    }
}

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

    /// Get a relay from the user's query, or return `None` if the multiplexed obfuscation retry
    /// strategy should be used instead.
    pub fn get_user_relay(&self, user_query: RelayQuery) -> Option<Result<GetRelay, Error>> {
        // Do not use the obfuscation multiplexer if the user has explicitly requested a single
        // anti-censorship method, or disabled anti-censorship.
        if user_query.entry_specific().obfuscation.is_only() {
            Some(self.get_relay_by_query(user_query))
        } else {
            None
        }
    }

    pub fn multiplexed_obfuscation_relay(
        &self,
        user_query: RelayQuery,
        obfuscation_strategy: &mut ObfuscationRound,
    ) -> Result<GetRelay, Error> {
        // Select a relay using the user's preferences and the first pending obfuscation method
        // that yields a relay.
        let relay = obfuscation_strategy
            .pending
            .iter()
            .filter_map(|(_, selection_query)| {
                // Select a relay that supports this pending method, but connect to it with the
                // user's query, whose obfuscation is on "auto" — so the relay's multiplexer
                // races every method it supports.
                self.get_relay_for_pending_obfuscation(selection_query.clone(), user_query.clone())
                    .inspect_err(|error| {
                        log::debug!("No relay for pending obfuscation method: {error}")
                    })
                    .ok()
            })
            .next();

        let Some(relay) = relay else {
            // No relay supports any of the pending obfuscation methods. Start a new round
            // and fall back to the user's query alone, which multiplexes every method the selected relay offers.
            *obfuscation_strategy = ObfuscationRound::new(user_query.clone());
            // No relay matches any pending obfuscation method. Fall back to the user's query
            // alone, which multiplexes every method the selected relay offers.
            return self.get_relay_by_query(user_query);
        };

        // Remove the methods the selected relay offers from the pending set. A relay selected
        // for one pending method multiplexes every method it supports, so they have all been
        // attempted and should not be retried within this round.
        if let Some(obfuscator) = &relay.obfuscator
            && let Obfuscators::Multiplexer {
                configs: (first, rest),
                ..
            } = obfuscator
        {
            let offered: std::collections::HashSet<ObfuscationMethodKind> = std::iter::once(first)
                .chain(rest.iter())
                .map(ObfuscationMethodKind::from)
                .collect();
            obfuscation_strategy.remove_offered(&offered);
        }

        Ok(relay)
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
