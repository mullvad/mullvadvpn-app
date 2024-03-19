//! The implementation of the relay selector.

pub mod detailer;
mod helpers;
mod matcher;
mod parsed_relays;
pub mod query;

use chrono::{DateTime, Local};
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use matcher::{BridgeMatcher, RelayMatcher};
use mullvad_types::{
    constraints::Constraint,
    custom_list::CustomListsSettings,
    endpoint::MullvadWireguardEndpoint,
    location::{Coordinates, Location},
    relay_constraints::{
        BridgeSettings, BridgeState, InternalBridgeConstraints, ObfuscationSettings,
        OpenVpnConstraints, RelayConstraints, RelayOverride, RelaySettings, ResolvedBridgeSettings,
        SelectedObfuscation, WireguardConstraints,
    },
    relay_list::{Relay, RelayList},
    settings::Settings,
    CustomTunnelEndpoint,
};
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig, proxy::CustomProxy, Endpoint, TransportProtocol, TunnelType,
    },
    ErrorExt,
};

use crate::error::{EndpointErrorDetails, Error};

use self::{
    detailer::{openvpn_endpoint, wireguard_endpoint},
    matcher::AnyTunnelMatcher,
    parsed_relays::ParsedRelays,
    query::{BridgeQuery, Intersection, OpenVpnRelayQuery, RelayQuery, WireguardRelayQuery},
};

/// [`RETRY_ORDER`] defines an ordered set of relay parameters which the relay selector should prioritize on
/// successive connection attempts. Note that these will *never* override user preferences.
/// See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.ms`
pub static RETRY_ORDER: Lazy<Vec<RelayQuery>> = Lazy::new(|| {
    use query::builder::{IpVersion, RelayQueryBuilder};
    vec![
        // 1
        // Note: This query can be unified with all possible user preferences.
        // If the user has tunnel protocol set to 'Auto', the relay selector will
        // default to picking a Wireguard relay.
        RelayQueryBuilder::new().build(),
        // 2
        RelayQueryBuilder::new().wireguard().port(443).build(),
        // 3
        RelayQueryBuilder::new()
            .wireguard()
            .ip_version(IpVersion::V6)
            .build(),
        // 4
        RelayQueryBuilder::new()
            .openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .port(443)
            .build(),
        // 5
        RelayQueryBuilder::new().wireguard().udp2tcp().build(),
        // 6
        RelayQueryBuilder::new()
            .wireguard()
            .udp2tcp()
            .ip_version(IpVersion::V6)
            .build(),
        // 7
        RelayQueryBuilder::new()
            .openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .bridge()
            .build(),
    ]
});

#[derive(Clone)]
pub struct RelaySelector {
    config: Arc<Mutex<SelectorConfig>>,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
}

#[derive(Clone)]
pub struct SelectorConfig {
    // Normal relay settings
    pub relay_settings: RelaySettings,
    pub custom_lists: CustomListsSettings,
    pub relay_overrides: Vec<RelayOverride>,
    // Wireguard specific data
    pub obfuscation_settings: ObfuscationSettings,
    // OpenVPN specific data
    pub bridge_state: BridgeState,
    pub bridge_settings: BridgeSettings,
}

/// This enum exists to separate the two types of [`SelectorConfig`] that exists.
///
/// The first one is a "regular" config, where [`SelectorConfig::relay_settings`] is [`RelaySettings::Normal`].
/// This is the most common variant, and there exists a mapping from this variant to [`RelayQueryBuilder`].
/// Being able to implement `From<NormalSelectorConfig> for RelayQueryBuilder` was the main
/// motivator for introducing these seemingly useless derivates of [`SelectorConfig`].
///
/// The second one is a custom config, where [`SelectorConfig::relay_settings`] is [`RelaySettings::Custom`].
/// For this variant, the  an endpoint where the client should connect to is already specified inside of the variant,
/// so in practice relay selector's function is superflous. Also, there exists no mapping to [`RelayQueryBuilder`].
#[derive(Debug, Clone)]
enum SpecializedSelectorConfig<'a> {
    // This variant implements `From<NormalSelectorConfig> for RelayQuery`
    Normal(NormalSelectorConfig<'a>),
    // This variant does not
    Custom(&'a CustomTunnelEndpoint),
}

/// A special-cased variant of [`SelectorConfig`].
///
/// For context, see [`SpecializedSelectorConfig`].
#[derive(Debug, Clone)]
struct NormalSelectorConfig<'a> {
    user_preferences: &'a RelayConstraints,
    custom_lists: &'a CustomListsSettings,
    // Wireguard specific data
    obfuscation_settings: &'a ObfuscationSettings,
    // OpenVPN specific data
    bridge_state: &'a BridgeState,
    bridge_settings: &'a BridgeSettings,
}

/// The return type of [`RelaySelector::get_relay`].
#[derive(Clone, Debug)]
pub enum GetRelay {
    Wireguard {
        endpoint: MullvadWireguardEndpoint,
        obfuscator: Option<SelectedObfuscator>,
        inner: WireguardConfig,
    },
    #[cfg(not(target_os = "android"))]
    OpenVpn {
        endpoint: Endpoint,
        exit: Relay,
        bridge: Option<SelectedBridge>,
    },
    Custom(CustomTunnelEndpoint),
}

/// This struct defines the different Wireguard relays the the relay selector can end up selecting
/// for an arbitrary Wireguard [`query`].
///
/// - [`WireguardConfig::Singlehop`]; A normal wireguard relay where VPN traffic enters and exits
/// through this sole relay.
/// - [`WireguardConfig::Multihop`]; Two wireguard relays to be used in a multihop circuit. VPN
/// traffic will enter through `entry` and eventually come out from `exit` before the traffic
/// will actually be routed to the broader internet.
#[derive(Clone, Debug)]
pub enum WireguardConfig {
    Singlehop { exit: Relay },
    Multihop { exit: Relay, entry: Relay },
}

#[derive(Clone, Debug)]
pub enum SelectedBridge {
    Normal { settings: CustomProxy, relay: Relay },
    Custom(CustomProxy),
}

impl SelectedBridge {
    /// Get the bridge settings.
    pub fn settings(&self) -> &CustomProxy {
        match self {
            SelectedBridge::Normal { settings, .. } => settings,
            SelectedBridge::Custom(settings) => settings,
        }
    }

    /// Get the relay acting as a bridge.
    /// This is not applicable if `self` is a [custom bridge][`SelectedBridge::Custom`].
    pub fn relay(&self) -> Option<&Relay> {
        match self {
            SelectedBridge::Normal { relay, .. } => Some(relay),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectedObfuscator {
    pub config: ObfuscatorConfig,
    pub relay: Relay,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        let default_settings = Settings::default();
        SelectorConfig {
            relay_settings: default_settings.relay_settings,
            bridge_settings: default_settings.bridge_settings,
            obfuscation_settings: default_settings.obfuscation_settings,
            bridge_state: default_settings.bridge_state,
            custom_lists: default_settings.custom_lists,
            relay_overrides: default_settings.relay_overrides,
        }
    }
}

impl<'a> From<&'a SelectorConfig> for SpecializedSelectorConfig<'a> {
    fn from(value: &'a SelectorConfig) -> SpecializedSelectorConfig<'a> {
        match &value.relay_settings {
            RelaySettings::CustomTunnelEndpoint(custom_tunnel_endpoint) => {
                SpecializedSelectorConfig::Custom(custom_tunnel_endpoint)
            }
            RelaySettings::Normal(user_preferences) => {
                SpecializedSelectorConfig::Normal(NormalSelectorConfig {
                    user_preferences,
                    obfuscation_settings: &value.obfuscation_settings,
                    bridge_state: &value.bridge_state,
                    bridge_settings: &value.bridge_settings,
                    custom_lists: &value.custom_lists,
                })
            }
        }
    }
}

impl<'a> From<NormalSelectorConfig<'a>> for RelayQuery {
    /// Map user settings to [`RelayQuery`].
    fn from(value: NormalSelectorConfig<'a>) -> Self {
        /// Map the Wireguard-specific bits of `value` to [`WireguradRelayQuery`]
        fn wireguard_constraints(
            wireguard_constraints: WireguardConstraints,
            obfuscation_settings: ObfuscationSettings,
        ) -> WireguardRelayQuery {
            let WireguardConstraints {
                port,
                ip_version,
                use_multihop,
                entry_location,
            } = wireguard_constraints;
            WireguardRelayQuery {
                port,
                ip_version,
                use_multihop: Constraint::Only(use_multihop),
                entry_location,
                obfuscation: obfuscation_settings.selected_obfuscation,
                udp2tcp_port: Constraint::Only(obfuscation_settings.udp2tcp.clone()),
            }
        }

        /// Map the OpenVPN-specific bits of `value` to [`OpenVpnRelayQuery`]
        fn openvpn_constraints(
            openvpn_constraints: OpenVpnConstraints,
            bridge_state: BridgeState,
            bridge_settings: BridgeSettings,
        ) -> OpenVpnRelayQuery {
            OpenVpnRelayQuery {
                port: openvpn_constraints.port,
                bridge_settings: match bridge_state {
                    BridgeState::On => match bridge_settings.bridge_type {
                        mullvad_types::relay_constraints::BridgeType::Normal => {
                            Constraint::Only(BridgeQuery::Normal(bridge_settings.normal.clone()))
                        }
                        mullvad_types::relay_constraints::BridgeType::Custom => {
                            Constraint::Only(BridgeQuery::Custom(bridge_settings.custom.clone()))
                        }
                    },
                    BridgeState::Auto => Constraint::Only(BridgeQuery::Auto),
                    BridgeState::Off => Constraint::Only(BridgeQuery::Off),
                },
            }
        }

        let wireguard_constraints = wireguard_constraints(
            value.user_preferences.wireguard_constraints.clone(),
            value.obfuscation_settings.clone(),
        );
        let openvpn_constraints = openvpn_constraints(
            value.user_preferences.openvpn_constraints,
            *value.bridge_state,
            value.bridge_settings.clone(),
        );
        RelayQuery {
            location: value.user_preferences.location.clone(),
            providers: value.user_preferences.providers.clone(),
            ownership: value.user_preferences.ownership,
            tunnel_protocol: value.user_preferences.tunnel_protocol,
            wireguard_constraints,
            openvpn_constraints,
        }
    }
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk.
    pub fn new(
        config: SelectorConfig,
        resource_path: impl AsRef<Path>,
        cache_path: impl AsRef<Path>,
    ) -> Self {
        const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";
        let unsynchronized_parsed_relays =
            ParsedRelays::from_file(&cache_path, &resource_path, &config.relay_overrides)
                .unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Unable to load cached and bundled relays")
                    );
                    ParsedRelays::empty()
                });
        log::info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().count(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );

        RelaySelector {
            config: Arc::new(Mutex::new(config)),
            parsed_relays: Arc::new(Mutex::new(unsynchronized_parsed_relays)),
        }
    }

    pub fn from_list(config: SelectorConfig, relay_list: RelayList) -> Self {
        RelaySelector {
            parsed_relays: Arc::new(Mutex::new(ParsedRelays::from_relay_list(
                relay_list,
                SystemTime::now(),
                &config.relay_overrides,
            ))),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn set_config(&mut self, config: SelectorConfig) {
        self.set_overrides(&config.relay_overrides);
        let mut config_mutex = self.config.lock().unwrap();
        *config_mutex = config;
    }

    pub fn set_relays(&self, relays: RelayList) {
        let mut parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.update(relays);
    }

    fn set_overrides(&mut self, relay_overrides: &[RelayOverride]) {
        let mut parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.set_overrides(relay_overrides);
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_relays(&mut self) -> RelayList {
        let parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.original_list().clone()
    }

    pub fn etag(&self) -> Option<String> {
        self.parsed_relays.lock().unwrap().etag()
    }

    pub fn last_updated(&self) -> SystemTime {
        self.parsed_relays.lock().unwrap().last_updated()
    }

    /// Returns a non-custom bridge based on the relay and bridge constraints, ignoring the bridge
    /// state.
    pub fn get_bridge_forced(&self) -> Option<CustomProxy> {
        let parsed_relays = &self.parsed_relays.lock().unwrap();
        let config = self.config.lock().unwrap();
        let specialized_config = SpecializedSelectorConfig::from(&*config);

        let near_location = match specialized_config {
            SpecializedSelectorConfig::Normal(pure) => {
                let user_preferences = RelayQuery::from(pure.clone());
                Self::get_relay_midpoint(parsed_relays, &user_preferences, &pure)
            }
            SpecializedSelectorConfig::Custom(_) => None,
        };

        let bridge_settings = &config.bridge_settings;
        let constraints = match bridge_settings.resolve() {
            Ok(ResolvedBridgeSettings::Normal(settings)) => InternalBridgeConstraints {
                location: settings.location.clone(),
                providers: settings.providers.clone(),
                ownership: settings.ownership,
                transport_protocol: Constraint::Only(TransportProtocol::Tcp),
            },
            _ => InternalBridgeConstraints {
                location: Constraint::Any,
                providers: Constraint::Any,
                ownership: Constraint::Any,
                transport_protocol: Constraint::Only(TransportProtocol::Tcp),
            },
        };

        let custom_lists = &config.custom_lists;
        Self::get_proxy_settings(parsed_relays, &constraints, near_location, custom_lists)
            .map(|(settings, _relay)| settings)
    }

    /// Returns random relay and relay endpoint matching `query`.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(pure_config) => {
                let parsed_relays = &self.parsed_relays.lock().unwrap();
                Self::get_relay_inner(&query, parsed_relays, &pure_config)
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in [`RETRY_ORDER`].
    ///
    /// [`RETRY_ORDER`]: crate::RETRY_ORDER
    pub fn get_relay(&self, retry_attempt: usize) -> Result<GetRelay, Error> {
        self.get_relay_with_order(&RETRY_ORDER, retry_attempt)
    }

    /// Peek at which [`TunnelType`] that would be returned for a certain connection attempt for a given
    /// [`SelectorConfig`]. Returns [`Option::None`] if the given config would return a custom
    /// tunnel endpoint.
    ///
    /// # Note
    /// This function is only really useful for testing-purposes. It is exposed to ease testing of
    /// other mullvad crates which depend on the retry behaviour of [`RelaySelector`].
    pub fn would_return(connection_attempt: usize, config: &SelectorConfig) -> Option<TunnelType> {
        let config = SpecializedSelectorConfig::from(config);
        match config {
            // This case is not really interesting
            SpecializedSelectorConfig::Custom(_) => None,
            SpecializedSelectorConfig::Normal(config) => Some(
                Self::pick_and_merge_query(
                    &RETRY_ORDER,
                    RelayQuery::from(config),
                    connection_attempt,
                )
                .tunnel_protocol
                .unwrap_or(TunnelType::Wireguard),
            ),
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints defined by
    /// `retry_order` corresponsing to `retry_attempt`.
    pub fn get_relay_with_order(
        &self,
        retry_order: &[RelayQuery],
        retry_attempt: usize,
    ) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);

        // Short-circuit if a custom tunnel endpoint is to be used - don't have to involve the
        // relay selector further!
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(normal_config) => {
                let parsed_relays = &self.parsed_relays.lock().unwrap();
                // Merge user preferences with the relay selector's default preferences.
                let user_preferences = RelayQuery::from(normal_config.clone());
                let query =
                    Self::pick_and_merge_query(retry_order, user_preferences, retry_attempt);
                Self::get_relay_inner(&query, parsed_relays, &normal_config)
            }
        }
    }

    /// This function defines the merge between a set of pre-defined queries and `user_preferences` for the given
    /// `retry_attempt`.
    ///
    /// This algorithm will loop back to the start of `retry_order` if `retry_attempt < retry_order.len()`.
    /// If `user_preferences` is not compatible with any of the pre-defined queries in `retry_order`, `user_preferences`
    /// is returned.
    fn pick_and_merge_query(
        retry_order: &[RelayQuery],
        user_preferences: RelayQuery,
        retry_attempt: usize,
    ) -> RelayQuery {
        retry_order
            .iter()
            .cycle()
            .filter_map(|constraint| constraint.clone().intersection(user_preferences.clone()))
            .nth(retry_attempt)
            .unwrap_or(user_preferences)
    }

    /// "Execute" the given query, yielding a final set of relays and/or bridges which the VPN traffic shall be routed through.
    ///
    /// # Parameters
    /// - `query`: Constraints that filter the available relays, such as geographic location or tunnel protocol.
    /// - `config`: Configuration settings that influence relay selection, including bridge state and custom lists.
    /// - `parsed_relays`: The complete set of parsed relays available for selection.
    ///
    /// # Returns
    /// * A randomly selected relay that meets the specified constraints (and a random bridge/entry relay if applicable).
    /// See [`GetRelay`] for more details.
    /// * An `Err` if no suitable relay is found
    /// * An `Err` if no suitable bridge is found
    #[cfg(not(target_os = "android"))]
    fn get_relay_inner(
        query: &RelayQuery,
        parsed_relays: &ParsedRelays,
        config: &NormalSelectorConfig<'_>,
    ) -> Result<GetRelay, Error> {
        match query.tunnel_protocol {
            Constraint::Only(TunnelType::Wireguard) => {
                Self::get_wireguard_relay(query, config, parsed_relays)
            }
            Constraint::Only(TunnelType::OpenVpn) => {
                Self::get_openvpn_relay(query, config, parsed_relays)
            }
            Constraint::Any => {
                // Try Wireguard, then OpenVPN, then fail
                for tunnel_type in [TunnelType::Wireguard, TunnelType::OpenVpn] {
                    let mut new_query = query.clone();
                    new_query.tunnel_protocol = Constraint::Only(tunnel_type);
                    // If a suitable relay is found, short-circuit and return it
                    if let Ok(relay) = Self::get_relay_inner(&new_query, parsed_relays, config) {
                        return Ok(relay);
                    }
                }
                Err(Error::NoRelay)
            }
        }
    }

    #[cfg(target_os = "android")]
    fn get_relay_inner(
        query: &RelayQuery,
        parsed_relays: &ParsedRelays,
        config: &NormalSelectorConfig<'_>,
    ) -> Result<GetRelay, Error> {
        Self::get_wireguard_relay(query, config, parsed_relays)
    }

    /// Derive a valid Wireguard relay configuration from `query`.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen (if multihop is enabled on `query`)
    /// * an `Err` if no [`MullvadEndpoint`] can be derived from the selected relay(s).
    /// * `Ok(GetRelay::Wireguard)` otherwise
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_relay(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Result<GetRelay, Error> {
        let inner = if !query.wireguard_constraints.multihop() {
            Self::get_wireguard_singlehop_config(query, config, parsed_relays)?
        } else {
            Self::get_wireguard_multihop_config(query, config, parsed_relays)?
        };
        let endpoint = Self::get_wireguard_endpoint(query, parsed_relays, &inner)?;
        let obfuscator =
            Self::get_wireguard_obfuscator(query, inner.clone(), &endpoint, parsed_relays)?;

        Ok(GetRelay::Wireguard {
            endpoint,
            obfuscator,
            inner,
        })
    }

    /// Select a valid Wireguard exit relay.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * `Ok(WireguardInner::Singlehop)` otherwise
    fn get_wireguard_singlehop_config(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Result<WireguardConfig, Error> {
        Self::choose_relay(query, config, parsed_relays)
            .map(|exit| WireguardConfig::Singlehop { exit })
            .ok_or(Error::NoRelay)
    }

    /// This function selects a valid entry and exit relay to be used in a multihop configuration.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen
    /// * An `Err` if the chosen entry and exit relays are the same
    /// * `Ok(WireguardInner::Multihop)` otherwise
    fn get_wireguard_multihop_config(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Result<WireguardConfig, Error> {
        // Here, we modify the original query just a bit.
        // The actual query for an exit relay is identical as for an exit relay, with the
        // exception that the location is different. It is simply the location as dictated by
        // the query's multihop constraint.
        let mut entry_relay_query = query.clone();
        entry_relay_query.location = query.wireguard_constraints.entry_location.clone();
        // After we have our two queries (one for the exit relay & one for the entry relay),
        // we can query for all exit & entry candidates! All candidates are needed for the next step.
        let exit_candidates = Self::get_tunnel_endpoints(query, config, parsed_relays);
        let entry_candidates =
            Self::get_tunnel_endpoints(&entry_relay_query, config, parsed_relays);

        // This algorithm gracefully handles a particular edge case that arise when a constraint on
        // the exit relay is more specific than on the entry relay which forces the relay selector
        // to choose one specific relay. The relay selector could end up selecting that specific
        // relay as the entry relay, thus leaving no remaining exit relay candidates or vice versa.
        let (exit, entry) = match (exit_candidates.as_slice(), entry_candidates.as_slice()) {
            ([exit], [entry]) if exit == entry => None,
            (exits, [entry]) if exits.contains(entry) => {
                let exit = helpers::random(exits, entry).ok_or(Error::NoRelay)?;
                Some((exit, entry))
            }
            ([exit], entrys) if entrys.contains(exit) => {
                let entry = helpers::random(entrys, exit).ok_or(Error::NoRelay)?;
                Some((exit, entry))
            }
            (exits, entrys) => {
                let exit = helpers::pick_random_relay(exits).ok_or(Error::NoRelay)?;
                let entry = helpers::random(entrys, exit).ok_or(Error::NoRelay)?;
                Some((exit, entry))
            }
        }
        .ok_or(Error::NoRelay)?;

        Ok(WireguardConfig::Multihop {
            exit: exit.clone(),
            entry: entry.clone(),
        })
    }

    /// Constructs a [`MullvadEndpoint`] with details for how to connect to `relay`.
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_endpoint(
        query: &RelayQuery,
        parsed_relays: &ParsedRelays,
        relay: &WireguardConfig,
    ) -> Result<MullvadWireguardEndpoint, Error> {
        wireguard_endpoint(
            &query.wireguard_constraints,
            &parsed_relays.parsed_list().wireguard,
            relay,
        )
        .map_err(|internal| Error::NoEndpoint {
            internal,
            relay: EndpointErrorDetails::from_wireguard(relay.clone()),
        })
    }

    fn get_wireguard_obfuscator(
        query: &RelayQuery,
        relay: WireguardConfig,
        endpoint: &MullvadWireguardEndpoint,
        parsed_relays: &ParsedRelays,
    ) -> Result<Option<SelectedObfuscator>, Error> {
        match query.wireguard_constraints.obfuscation {
            SelectedObfuscation::Off | SelectedObfuscation::Auto => Ok(None),
            SelectedObfuscation::Udp2Tcp => {
                let obfuscator_relay = match relay {
                    WireguardConfig::Singlehop { exit } => exit,
                    WireguardConfig::Multihop { entry, .. } => entry,
                };
                let udp2tcp_ports = &parsed_relays.parsed_list().wireguard.udp2tcp_ports;

                helpers::get_udp2tcp_obfuscator(
                    &query.wireguard_constraints.udp2tcp_port,
                    udp2tcp_ports,
                    obfuscator_relay,
                    endpoint,
                )
                .map(Some)
                .ok_or(Error::NoObfuscator)
            }
        }
    }

    /// Derive a valid OpenVPN relay configuration from `query`.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry bridge can be chosen (if bridge mode is enabled on `query`)
    /// * an `Err` if no [`MullvadEndpoint`] can be derived from the selected relay
    /// * `Ok(GetRelay::OpenVpn)` otherwise
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    #[cfg(not(target_os = "android"))]
    fn get_openvpn_relay(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Result<GetRelay, Error> {
        let exit = Self::choose_relay(query, config, parsed_relays).ok_or(Error::NoRelay)?;
        let endpoint = Self::get_openvpn_endpoint(query, &exit, parsed_relays)?;
        let bridge =
            Self::get_openvpn_bridge(query, &exit, &endpoint.protocol, parsed_relays, config)?;

        Ok(GetRelay::OpenVpn {
            endpoint,
            exit,
            bridge,
        })
    }

    /// Constructs a [`MullvadEndpoint`] with details for how to connect to `relay`.
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    #[cfg(not(target_os = "android"))]
    fn get_openvpn_endpoint(
        query: &RelayQuery,
        relay: &Relay,
        parsed_relays: &ParsedRelays,
    ) -> Result<Endpoint, Error> {
        openvpn_endpoint(
            &query.openvpn_constraints,
            &parsed_relays.parsed_list().openvpn,
            relay,
        )
        .map_err(|internal| Error::NoEndpoint {
            internal,
            relay: EndpointErrorDetails::from_openvpn(relay.clone()),
        })
    }

    /// Selects a suitable bridge based on the specified settings, relay information, and transport protocol.
    ///
    /// # Parameters
    /// - `query`: The filter criteria for selecting a bridge.
    /// - `relay`: Information about the current relay, including its location.
    /// - `protocol`: The transport protocol (TCP or UDP) in use.
    /// - `parsed_relays`: A structured representation of all available relays.
    /// - `custom_lists`: User-defined or application-specific settings that may influence bridge selection.
    ///
    /// # Returns
    /// * On success, returns an `Option` containing the selected bridge, if one is found. Returns `None` if no suitable bridge meets the criteria or bridges should not be used.
    /// * `Error::NoBridge` if attempting to use OpenVPN bridges over UDP, as this is unsupported.
    /// * `Error::NoRelay` if `relay` does not have a location set.
    #[cfg(not(target_os = "android"))]
    fn get_openvpn_bridge(
        query: &RelayQuery,
        relay: &Relay,
        protocol: &TransportProtocol,
        parsed_relays: &ParsedRelays,
        config: &NormalSelectorConfig<'_>,
    ) -> Result<Option<SelectedBridge>, Error> {
        if !BridgeQuery::should_use_bridge(&query.openvpn_constraints.bridge_settings) {
            Ok(None)
        } else {
            let bridge_query = &query.openvpn_constraints.bridge_settings.clone().unwrap();
            let custom_lists = &config.custom_lists;
            match protocol {
                TransportProtocol::Udp => {
                    log::error!("Can not use OpenVPN bridges over UDP");
                    Err(Error::NoBridge)
                }
                TransportProtocol::Tcp => {
                    let location = relay.location.as_ref().ok_or(Error::NoRelay)?;
                    Ok(Self::get_bridge_for(
                        bridge_query,
                        location,
                        // FIXME: This is temporary while talpid-core only supports TCP proxies
                        TransportProtocol::Tcp,
                        parsed_relays,
                        custom_lists,
                    ))
                }
            }
        }
    }

    fn get_bridge_for(
        query: &BridgeQuery,
        location: &Location,
        transport_protocol: TransportProtocol,
        parsed_relays: &ParsedRelays,
        custom_lists: &CustomListsSettings,
    ) -> Option<SelectedBridge> {
        match query {
            BridgeQuery::Normal(settings) => {
                let bridge_constraints = InternalBridgeConstraints {
                    location: settings.location.clone(),
                    providers: settings.providers.clone(),
                    ownership: settings.ownership,
                    transport_protocol: Constraint::Only(transport_protocol),
                };

                Self::get_proxy_settings(
                    parsed_relays,
                    &bridge_constraints,
                    Some(location),
                    custom_lists,
                )
                .map(|(settings, relay)| SelectedBridge::Normal { settings, relay })
            }
            BridgeQuery::Custom(settings) => settings.clone().map(SelectedBridge::Custom),
            BridgeQuery::Off | BridgeQuery::Auto => None,
        }
    }

    /// Try to get a bridge that matches the given `constraints`.
    ///
    /// The connection details are returned alongside the relay hosting the bridge.
    fn get_proxy_settings<T: Into<Coordinates>>(
        parsed_relays: &ParsedRelays,
        constraints: &InternalBridgeConstraints,
        location: Option<T>,
        custom_lists: &CustomListsSettings,
    ) -> Option<(CustomProxy, Relay)> {
        let matcher = BridgeMatcher::new_matcher(constraints.clone(), custom_lists);
        let relays = matcher.filter_matching_relay_list(parsed_relays.relays());

        let relay = match location {
            Some(location) => Self::get_proximate_bridge(relays, location),
            None => helpers::pick_random_relay(&relays).cloned(),
        }?;

        let bridge = &parsed_relays.parsed_list().bridge;
        helpers::pick_random_bridge(bridge, &relay).map(|bridge| (bridge, relay.clone()))
    }

    /// Try to get a bridge which is close to `location`.
    fn get_proximate_bridge<T: Into<Coordinates>>(
        relays: Vec<Relay>,
        location: T,
    ) -> Option<Relay> {
        /// Minimum number of bridges to keep for selection when filtering by distance.
        const MIN_BRIDGE_COUNT: usize = 5;
        /// Max distance of bridges to consider for selection (km).
        const MAX_BRIDGE_DISTANCE: f64 = 1500f64;
        let location = location.into();

        #[derive(Clone)]
        struct RelayWithDistance {
            relay: Relay,
            distance: f64,
        }

        // Filter out all candidate bridges.
        let matching_relays: Vec<RelayWithDistance> = relays
            .into_iter()
            .map(|relay| RelayWithDistance {
                distance: relay.location.as_ref().unwrap().distance_from(&location),
                relay,
            })
            .sorted_unstable_by_key(|relay| relay.distance as usize)
            .take(MIN_BRIDGE_COUNT)
            .filter(|relay| relay.distance <= MAX_BRIDGE_DISTANCE)
            .collect();

        // Calculate the maximum distance from `location` among the candidates.
        let greatest_distance: f64 = matching_relays
            .iter()
            .map(|relay| relay.distance)
            .reduce(f64::max)?;
        // Define the weight function to prioritize bridges which are closer to `location`.
        let weight_fn = |relay: &RelayWithDistance| 1 + (greatest_distance - relay.distance) as u64;

        helpers::pick_random_relay_weighted(&matching_relays, weight_fn)
            .cloned()
            .map(|relay_with_distance| relay_with_distance.relay)
    }

    /// Returns the average location of relays that match the given constraints.
    /// This returns `None` if the location is [`Constraint::Any`] or if no
    /// relays match the constraints.
    fn get_relay_midpoint(
        parsed_relays: &ParsedRelays,
        constraints: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
    ) -> Option<Coordinates> {
        if constraints.location.is_any() {
            return None;
        }

        let matcher = RelayMatcher::new(
            constraints.clone(),
            &parsed_relays.parsed_list().openvpn,
            *config.bridge_state,
            &parsed_relays.parsed_list().wireguard,
            config.custom_lists,
        );

        Self::get_relay_midpoint_inner(parsed_relays, matcher)
    }

    fn get_relay_midpoint_inner(
        parsed_relays: &ParsedRelays,
        matcher: RelayMatcher<AnyTunnelMatcher<'_>>,
    ) -> Option<Coordinates> {
        use std::ops::Not;
        let matching_locations: Vec<Location> = matcher
            .filter_matching_relay_list(parsed_relays.relays())
            .into_iter()
            .filter_map(|relay| relay.location)
            .unique_by(|location| location.city.clone())
            .collect();

        matching_locations
            .is_empty()
            .not()
            .then(|| Coordinates::midpoint(&matching_locations))
    }

    /// Chooses a suitable relay from a set of parsed relays based on specified constraints and configuration.
    ///
    /// This function filters the available relays according to the given [query][`RelayQuery`] and [config][`NormalSelectorConfig`],
    /// then selects one relay at random from the filtered list.
    ///
    /// # Returns
    /// A randomly selected relay that meets the specified constraints, or `None` if no suitable relay is found.
    fn choose_relay(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Option<Relay> {
        // Filter among all valid relays
        let relays = Self::get_tunnel_endpoints(query, config, parsed_relays);
        // Pick one of the valid relays.
        helpers::pick_random_relay(&relays).cloned()
    }

    /// Returns all relays matching the given `query`.
    #[cfg(not(target_os = "android"))]
    fn get_tunnel_endpoints(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Vec<Relay> {
        let relays = parsed_relays.relays();
        let matcher = RelayMatcher::new(
            query.clone(),
            &parsed_relays.parsed_list().openvpn,
            *config.bridge_state,
            &parsed_relays.parsed_list().wireguard,
            config.custom_lists,
        );
        matcher.filter_matching_relay_list(relays)
    }

    /// Returns all relays matching the given `query`.
    #[cfg(target_os = "android")]
    fn get_tunnel_endpoints(
        query: &RelayQuery,
        config: &NormalSelectorConfig<'_>,
        parsed_relays: &ParsedRelays,
    ) -> Vec<Relay> {
        use self::matcher::WireguardMatcher;
        let relays = parsed_relays.relays();
        let endpoint_matcher = WireguardMatcher::new(
            query.wireguard_constraints.clone(),
            &parsed_relays.parsed_list().wireguard,
        );
        let matcher = RelayMatcher::using(query.clone(), config.custom_lists, endpoint_matcher);
        matcher.filter_matching_relay_list(relays)
    }
}
