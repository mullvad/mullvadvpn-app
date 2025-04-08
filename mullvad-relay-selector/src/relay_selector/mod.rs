//! The implementation of the relay selector.

pub mod detailer;
mod helpers;
pub mod matcher;
mod parsed_relays;
pub mod query;
pub mod relays;

use matcher::{filter_matching_bridges, filter_matching_relay_list};
use parsed_relays::ParsedRelays;
use relays::{Multihop, Singlehop, WireguardConfig};

use crate::{
    detailer::{openvpn_endpoint, wireguard_endpoint},
    error::{EndpointErrorDetails, Error},
    query::{
        BridgeQuery, ObfuscationQuery, OpenVpnRelayQuery, RelayQuery, RelayQueryExt,
        WireguardRelayQuery,
    },
};

use chrono::{DateTime, Local};
use itertools::Itertools;
use mullvad_types::{
    constraints::Constraint,
    custom_list::CustomListsSettings,
    endpoint::MullvadWireguardEndpoint,
    location::{Coordinates, Location},
    relay_constraints::{
        BridgeSettings, BridgeState, InternalBridgeConstraints, ObfuscationSettings,
        OpenVpnConstraints, RelayConstraints, RelayOverride, RelaySettings, ResolvedBridgeSettings,
        WireguardConstraints,
    },
    relay_list::{Relay, RelayEndpointData, RelayList},
    settings::Settings,
    wireguard::QuantumResistantState,
    CustomTunnelEndpoint, Intersection,
};
use std::{
    path::Path,
    sync::{Arc, LazyLock, Mutex},
    time::SystemTime,
};
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig,
        proxy::{CustomProxy, Shadowsocks},
        Endpoint, IpAvailability, IpVersion, TransportProtocol, TunnelType,
    },
    ErrorExt,
};

/// [`WIREGUARD_RETRY_ORDER`] defines an ordered set of relay parameters which the relay selector
/// should prioritize on successive connection attempts. Note that these will *never* override user
/// preferences. See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.md`
pub static WIREGUARD_RETRY_ORDER: LazyLock<Vec<RelayQuery>> = LazyLock::new(|| {
    use query::builder::{IpVersion, RelayQueryBuilder};
    vec![
        // 1 This works with any wireguard relay
        RelayQueryBuilder::wireguard().build(),
        // 2
        RelayQueryBuilder::wireguard().port(443).build(),
        // 3
        RelayQueryBuilder::wireguard()
            .ip_version(IpVersion::V6)
            .build(),
        // 4
        RelayQueryBuilder::wireguard().shadowsocks().build(),
        // 5
        RelayQueryBuilder::wireguard().udp2tcp().build(),
        // 6
        RelayQueryBuilder::wireguard()
            .udp2tcp()
            .ip_version(IpVersion::V6)
            .build(),
    ]
});

/// [`OPENVPN_RETRY_ORDER`] defines an ordered set of relay parameters which the relay selector
/// should prioritize on successive connection attempts. Note that these will *never* override user
/// preferences. See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.md`
pub static OPENVPN_RETRY_ORDER: LazyLock<Vec<RelayQuery>> = LazyLock::new(|| {
    use query::builder::RelayQueryBuilder;
    vec![
        // 1 (openvpn) This works with any openvpn relay
        RelayQueryBuilder::openvpn().build(),
        // 2
        RelayQueryBuilder::openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .port(443)
            .build(),
        // 3
        RelayQueryBuilder::openvpn()
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
    pub additional_constraints: AdditionalRelayConstraints,
    pub custom_lists: CustomListsSettings,
    pub relay_overrides: Vec<RelayOverride>,
    // Wireguard specific data
    pub obfuscation_settings: ObfuscationSettings,
    // OpenVPN specific data
    pub bridge_state: BridgeState,
    pub bridge_settings: BridgeSettings,
}

impl SelectorConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let additional_constraints = AdditionalRelayConstraints {
            wireguard: AdditionalWireguardConstraints {
                #[cfg(daita)]
                daita: settings.tunnel_options.wireguard.daita.enabled,
                #[cfg(daita)]
                daita_use_multihop_if_necessary: settings
                    .tunnel_options
                    .wireguard
                    .daita
                    .use_multihop_if_necessary,

                #[cfg(not(daita))]
                daita: false,
                #[cfg(not(daita))]
                daita_use_multihop_if_necessary: false,

                quantum_resistant: settings.tunnel_options.wireguard.quantum_resistant,
            },
        };

        Self {
            relay_settings: settings.relay_settings.clone(),
            additional_constraints,
            bridge_state: settings.bridge_state,
            bridge_settings: settings.bridge_settings.clone(),
            obfuscation_settings: settings.obfuscation_settings.clone(),
            custom_lists: settings.custom_lists.clone(),
            relay_overrides: settings.relay_overrides.clone(),
        }
    }
}

/// Extra relay constraints not specified in `relay_settings`.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct AdditionalRelayConstraints {
    pub wireguard: AdditionalWireguardConstraints,
}

/// Constraints to use when selecting WireGuard servers
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct AdditionalWireguardConstraints {
    /// If true, select WireGuard relays that support DAITA. If false, select any
    /// server.
    pub daita: bool,

    /// If true and multihop is disabled, will set up multihop with an automatic entry relay if
    /// DAITA is enabled.
    pub daita_use_multihop_if_necessary: bool,

    /// If enabled, select relays that support PQ.
    pub quantum_resistant: QuantumResistantState,
}

/// This enum exists to separate the two types of [`SelectorConfig`] that exists.
///
/// The first one is a "regular" config, where [`SelectorConfig::relay_settings`] is
/// [`RelaySettings::Normal`]. This is the most common variant, and there exists a
/// mapping from this variant to [`RelayQueryBuilder`]. Being able to implement
/// `From<NormalSelectorConfig> for RelayQueryBuilder` was the main motivator for introducing these
/// seemingly useless derivates of [`SelectorConfig`].
///
/// The second one is a custom config, where [`SelectorConfig::relay_settings`] is
/// [`RelaySettings::CustomTunnelEndpoint`]. For this variant, the endpoint where the client should
/// connect to is already specified inside of the variant, so in practice the relay selector becomes
/// superfluous. Also, there exists no mapping to [`RelayQueryBuilder`].
///
/// [`RelayQueryBuilder`]: query::builder::RelayQueryBuilder
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
    additional_preferences: &'a AdditionalRelayConstraints,
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

#[derive(Clone, Debug)]
pub enum SelectedBridge {
    Normal {
        // Mullvad operated bridges will always be Shadowsocks proxies.
        settings: Shadowsocks,
        relay: Relay,
    },
    Custom(CustomProxy),
}

impl SelectedBridge {
    /// Get the bridge settings.
    pub fn to_proxy(self) -> CustomProxy {
        match self {
            SelectedBridge::Normal { settings, .. } => CustomProxy::Shadowsocks(settings),
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
            additional_constraints: AdditionalRelayConstraints::default(),
            bridge_settings: default_settings.bridge_settings,
            obfuscation_settings: default_settings.obfuscation_settings,
            bridge_state: default_settings.bridge_state,
            custom_lists: default_settings.custom_lists,
            relay_overrides: default_settings.relay_overrides,
        }
    }
}

impl TryFrom<Settings> for RelayQuery {
    type Error = crate::Error;

    fn try_from(value: Settings) -> Result<Self, Self::Error> {
        let selector_config = SelectorConfig::from_settings(&value);
        let specilized_selector_config = SpecializedSelectorConfig::from(&selector_config);
        let SpecializedSelectorConfig::Normal(normal_selector_config) = specilized_selector_config
        else {
            return Err(Error::InvalidConstraints);
        };

        RelayQuery::try_from(normal_selector_config)
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
                    additional_preferences: &value.additional_constraints,
                    obfuscation_settings: &value.obfuscation_settings,
                    bridge_state: &value.bridge_state,
                    bridge_settings: &value.bridge_settings,
                    custom_lists: &value.custom_lists,
                })
            }
        }
    }
}

impl<'a> TryFrom<NormalSelectorConfig<'a>> for RelayQuery {
    type Error = crate::Error;

    /// Map user settings to [`RelayQuery`].
    fn try_from(value: NormalSelectorConfig<'a>) -> Result<Self, Self::Error> {
        /// Map the Wireguard-specific bits of `value` to [`WireguradRelayQuery`]
        fn wireguard_constraints(
            wireguard_constraints: WireguardConstraints,
            additional_constraints: AdditionalWireguardConstraints,
            obfuscation_settings: ObfuscationSettings,
        ) -> WireguardRelayQuery {
            let WireguardConstraints {
                port,
                ip_version,
                use_multihop,
                entry_location,
            } = wireguard_constraints;
            let AdditionalWireguardConstraints {
                daita,
                daita_use_multihop_if_necessary,
                quantum_resistant,
            } = additional_constraints;
            WireguardRelayQuery {
                port,
                ip_version,
                use_multihop: Constraint::Only(use_multihop),
                entry_location,
                obfuscation: ObfuscationQuery::from(obfuscation_settings),
                daita: Constraint::Only(daita),
                daita_use_multihop_if_necessary: Constraint::Only(daita_use_multihop_if_necessary),
                quantum_resistant,
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
                            BridgeQuery::Normal(bridge_settings.normal.clone())
                        }
                        mullvad_types::relay_constraints::BridgeType::Custom => {
                            BridgeQuery::Custom(bridge_settings.custom.clone())
                        }
                    },
                    BridgeState::Auto => BridgeQuery::Auto,
                    BridgeState::Off => BridgeQuery::Off,
                },
            }
        }

        let wireguard_constraints = wireguard_constraints(
            value.user_preferences.wireguard_constraints.clone(),
            value.additional_preferences.wireguard.clone(),
            value.obfuscation_settings.clone(),
        );
        let openvpn_constraints = openvpn_constraints(
            value.user_preferences.openvpn_constraints,
            *value.bridge_state,
            value.bridge_settings.clone(),
        );
        RelayQuery::new(
            value.user_preferences.location.clone(),
            value.user_preferences.providers.clone(),
            value.user_preferences.ownership,
            value.user_preferences.tunnel_protocol,
            wireguard_constraints,
            openvpn_constraints,
        )
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
    pub fn get_bridge_forced(&self) -> Option<Shadowsocks> {
        let parsed_relays = &self.parsed_relays.lock().unwrap().parsed_list().clone();
        let config = self.config.lock().unwrap();
        let specialized_config = SpecializedSelectorConfig::from(&*config);

        let near_location = match specialized_config {
            SpecializedSelectorConfig::Normal(config) => RelayQuery::try_from(config.clone())
                .ok()
                .and_then(|user_preferences| {
                    Self::get_relay_midpoint(&user_preferences, parsed_relays, config.custom_lists)
                }),
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
            .inspect_err(|error| log::error!("Failed to get bridge: {error}"))
            .ok()
    }

    /// Returns random relay and relay endpoint matching `query`.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(normal_config) => {
                let relay_list = &self.parsed_relays.lock().unwrap().parsed_list().clone();
                Self::get_relay_inner(&query, relay_list, normal_config.custom_lists)
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in one of the retry orders while considering
    /// [runtime_params][`RuntimeParameters`].
    pub fn get_relay(
        &self,
        retry_attempt: usize,
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(normal_config) => {
                let tunnel_protocol = normal_config.user_preferences.tunnel_protocol;
                drop(config_guard);

                match tunnel_protocol {
                    TunnelType::Wireguard => self.get_relay_with_custom_params(
                        retry_attempt,
                        &WIREGUARD_RETRY_ORDER,
                        runtime_ip_availability,
                    ),
                    TunnelType::OpenVpn => self.get_relay_with_custom_params(
                        retry_attempt,
                        &OPENVPN_RETRY_ORDER,
                        runtime_ip_availability,
                    ),
                }
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints defined by
    /// `retry_order` corresponding to `retry_attempt`.
    pub fn get_relay_with_custom_params(
        &self,
        retry_attempt: usize,
        retry_order: &[RelayQuery],
        runtime_ip_availability: IpAvailability,
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
                let relay_list = self.parsed_relays.lock().unwrap().parsed_list().clone();
                // Merge user preferences with the relay selector's default preferences.
                let query = Self::pick_and_merge_query(
                    retry_attempt,
                    retry_order,
                    runtime_ip_availability,
                    &normal_config,
                    &relay_list,
                )?;
                Self::get_relay_inner(&query, &relay_list, normal_config.custom_lists)
            }
        }
    }

    /// This function defines the merge between a set of pre-defined queries and `user_preferences`
    /// for the given `retry_attempt`.
    ///
    /// This algorithm will loop back to the start of `retry_order` if `retry_attempt <
    /// retry_order.len()`. If `user_preferences` is not compatible with any of the pre-defined
    /// queries in `retry_order`, `user_preferences` is returned.
    ///
    /// Runtime parameters may affect which of the default queries that are considered. For example,
    /// queries which rely on IPv6 will not be considered if working IPv6 is not available at
    /// runtime.
    ///
    /// Returns an error iff the intersection between the user's preferences and every default retry
    /// attempt-query yields queries with no matching relays. I.e., no retry attempt could ever
    /// resolve to a relay.
    fn pick_and_merge_query(
        retry_attempt: usize,
        retry_order: &[RelayQuery],
        runtime_ip_availability: IpAvailability,
        user_config: &NormalSelectorConfig<'_>,
        parsed_relays: &RelayList,
    ) -> Result<RelayQuery, Error> {
        let mut user_query = RelayQuery::try_from(user_config.clone())?;
        apply_ip_availability(runtime_ip_availability, &mut user_query)?;
        log::trace!("Merging user preferences {user_query:?} with default retry strategy");
        retry_order
            .iter()
            .filter_map(|query| query.clone().intersection(user_query.clone()))
            .filter(|query| Self::get_relay_inner(query, parsed_relays, user_config.custom_lists).is_ok())
            .cycle() // If the above filters remove all relays, cycle will also return an empty iterator
            .nth(retry_attempt)
            .ok_or(Error::NoRelay)
    }

    /// "Execute" the given query, yielding a final set of relays and/or bridges which the VPN
    /// traffic shall be routed through.
    ///
    /// # Parameters
    /// - `query`: Constraints that filter the available relays, such as geographic location or
    ///   tunnel protocol.
    /// - `config`: Configuration settings that influence relay selection, including bridge state
    ///   and custom lists.
    /// - `parsed_relays`: The complete set of parsed relays available for selection.
    ///
    /// # Returns
    /// * A randomly selected relay that meets the specified constraints (and a random bridge/entry
    ///   relay if applicable). See [`GetRelay`] for more details.
    /// * An `Err` if no suitable relay is found
    /// * An `Err` if no suitable bridge is found
    #[cfg(not(target_os = "android"))]
    fn get_relay_inner(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Result<GetRelay, Error> {
        match query.tunnel_protocol() {
            TunnelType::Wireguard => {
                Self::get_wireguard_relay_inner(query, custom_lists, parsed_relays)
            }
            TunnelType::OpenVpn => Self::get_openvpn_relay(query, custom_lists, parsed_relays),
        }
    }

    #[cfg(target_os = "android")]
    fn get_relay_inner(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Result<GetRelay, Error> {
        // FIXME: A bit of defensive programming - calling `get_wireguard_relay_inner` with a query
        // that doesn't specify Wireguard as the desired tunnel type is not valid and will
        // lead to unwanted behavior. This should be seen as a workaround, and it would be
        // nicer to lift this invariant to be checked by the type system instead.
        let mut query = query.clone();
        query.set_tunnel_protocol(TunnelType::Wireguard)?;
        Self::get_wireguard_relay_inner(&query, custom_lists, parsed_relays)
    }

    /// Derive a valid relay configuration from `query`.
    ///
    /// # Note
    /// This function should *only* be called with a Wireguard query.
    /// It will panic if the tunnel type is not specified as Wireguard.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen (if multihop is enabled on `query`)
    /// * an `Err` if no [`MullvadEndpoint`] can be derived from the selected relay(s).
    /// * `Ok(GetRelay::Wireguard)` otherwise
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_relay_inner(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<GetRelay, Error> {
        assert_eq!(query.tunnel_protocol(), TunnelType::Wireguard);
        let inner = Self::get_wireguard_relay_config(query, custom_lists, parsed_relays)?;
        let endpoint = Self::get_wireguard_endpoint(query, parsed_relays, &inner)?;
        let obfuscator =
            Self::get_wireguard_obfuscator(query, inner.clone(), &endpoint, parsed_relays)?;

        Ok(GetRelay::Wireguard {
            endpoint,
            obfuscator,
            inner,
        })
    }

    /// Derive a valid Wireguard relay configuration from `query`.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen (if multihop is enabled on `query`)
    /// * `Ok(WireguardConfig)` otherwise
    fn get_wireguard_relay_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<WireguardConfig, Error> {
        let inner = if query.singlehop() {
            match Self::get_wireguard_singlehop_config(query, custom_lists, parsed_relays) {
                Some(exit) => WireguardConfig::from(exit),
                None => {
                    // If we found no matching relays because DAITA was enabled, and
                    // `use_multihop_if_necessary` is enabled, try enabling
                    // multihop and connecting using an automatically selected
                    // entry relay.
                    if query.using_daita() && query.use_multihop_if_necessary() {
                        let multihop = Self::get_wireguard_auto_multihop_config(
                            query,
                            custom_lists,
                            parsed_relays,
                        )?;
                        WireguardConfig::from(multihop)
                    } else {
                        return Err(Error::NoRelay);
                    }
                }
            }
        } else {
            // A DAITA compatible entry should be used even when the exit is DAITA compatible.
            // This only makes sense in context: The user is no longer able to explicitly choose an
            // entry relay with smarting routing enabled, even if multihop is turned on
            // Also implied: Multihop is enabled.
            let multihop = if query.using_daita() && query.use_multihop_if_necessary() {
                Self::get_wireguard_auto_multihop_config(query, custom_lists, parsed_relays)?
            } else {
                Self::get_wireguard_multihop_config(query, custom_lists, parsed_relays)?
            };
            WireguardConfig::from(multihop)
        };

        Ok(inner)
    }

    /// Select a valid Wireguard exit relay.
    ///
    /// # Returns
    /// * `Ok(Singlehop)` if an exit relay was selected
    /// * `None` otherwise
    fn get_wireguard_singlehop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Option<Singlehop> {
        let candidates = filter_matching_relay_list(query, parsed_relays, custom_lists);
        helpers::pick_random_relay(&candidates)
            .cloned()
            .map(Singlehop::new)
    }

    /// Select a valid Wireguard exit relay, together with with an automatically chosen entry relay.
    ///
    /// # Returns
    /// * An `Err` if no entry/exit relay can be chosen
    /// * `Ok(Multihop)` otherwise
    fn get_wireguard_auto_multihop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<Multihop, Error> {
        let mut exit_relay_query = query.clone();

        // DAITA should only be enabled for the entry relay
        let mut wireguard_constraints = exit_relay_query.wireguard_constraints().clone();
        wireguard_constraints.daita = Constraint::Only(false);
        exit_relay_query.set_wireguard_constraints(wireguard_constraints)?;

        let exit_candidates =
            filter_matching_relay_list(&exit_relay_query, parsed_relays, custom_lists);
        let exit = helpers::pick_random_relay(&exit_candidates).ok_or(Error::NoRelay)?;

        // generate a list of potential entry relays, disregarding any location constraint
        let mut entry_query = query.clone();
        entry_query.set_location(Constraint::Any)?;
        let mut entry_candidates =
            filter_matching_relay_list(&entry_query, parsed_relays, custom_lists)
                .into_iter()
                .map(|entry| RelayWithDistance::new_with_distance_from(entry, &exit.location))
                .collect_vec();

        // sort entry relay candidates by distance, and pick one from those that are closest
        entry_candidates.sort_unstable_by(|a, b| a.distance.total_cmp(&b.distance));
        let smallest_distance = entry_candidates.first().map(|relay| relay.distance);
        let smallest_distance = smallest_distance.unwrap_or_default();
        let entry_candidates = entry_candidates
            .into_iter()
            // only consider the relay(s) with the smallest distance. note that the list is sorted.
            // NOTE: we could relax this requirement, but since so few relays support DAITA
            // (and this function is only used for daita) we might end up picking relays that are
            // needlessly far away. Consider making this closure  configurable if needed.
            .take_while(|relay| relay.distance <= smallest_distance)
            .map(|relay_with_distance| relay_with_distance.relay)
            .collect_vec();
        let entry =
            helpers::pick_random_relay_excluding(&entry_candidates, exit).ok_or(Error::NoRelay)?;

        Ok(Multihop::new(entry.clone(), exit.clone()))
    }

    /// This function selects a valid entry and exit relay to be used in a multihop configuration.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen
    /// * An `Err` if the chosen entry and exit relays are the same
    /// * `Ok(WireguardConfig::Multihop)` otherwise
    fn get_wireguard_multihop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<Multihop, Error> {
        // Here, we modify the original query just a bit.
        // The actual query for an entry relay is identical as for an exit relay, with the
        // exception that the location is different. It is simply the location as dictated by
        // the query's multihop constraint.
        let mut entry_relay_query = query.clone();
        entry_relay_query.set_location(query.wireguard_constraints().entry_location.clone())?;
        // After we have our two queries (one for the exit relay & one for the entry relay),
        // we can query for all exit & entry candidates! All candidates are needed for the next
        // step.
        let mut exit_relay_query = query.clone();
        // DAITA should only be enabled for the entry relay

        let mut wg_constraints = exit_relay_query.wireguard_constraints().clone();
        wg_constraints.daita = Constraint::Only(false);
        exit_relay_query.set_wireguard_constraints(wg_constraints)?;

        let exit_candidates =
            filter_matching_relay_list(&exit_relay_query, parsed_relays, custom_lists);
        let entry_candidates =
            filter_matching_relay_list(&entry_relay_query, parsed_relays, custom_lists);

        // We avoid picking the same relay for entry and exit by choosing one and excluding it when
        // choosing the other.
        let (exit, entry) = match (exit_candidates.as_slice(), entry_candidates.as_slice()) {
            // In the case where there is only one entry to choose from, we have to pick it before
            // the exit
            (exits, [entry]) if exits.contains(entry) => {
                helpers::pick_random_relay_excluding(exits, entry).map(|exit| (exit, entry))
            }
            // Vice versa for the case of only one exit
            ([exit], entries) if entries.contains(exit) => {
                helpers::pick_random_relay_excluding(entries, exit).map(|entry| (exit, entry))
            }
            (exits, entries) => helpers::pick_random_relay(exits).and_then(|exit| {
                helpers::pick_random_relay_excluding(entries, exit).map(|entry| (exit, entry))
            }),
        }
        .ok_or(Error::NoRelay)?;

        Ok(Multihop::new(entry.clone(), exit.clone()))
    }

    /// Constructs a [`MullvadEndpoint`] with details for how to connect to `relay`.
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_endpoint(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        relay: &WireguardConfig,
    ) -> Result<MullvadWireguardEndpoint, Error> {
        wireguard_endpoint(
            query.wireguard_constraints(),
            &parsed_relays.wireguard,
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
        parsed_relays: &RelayList,
    ) -> Result<Option<SelectedObfuscator>, Error> {
        let obfuscator_relay = match relay {
            WireguardConfig::Singlehop { exit } => exit,
            WireguardConfig::Multihop { entry, .. } => entry,
        };
        let box_obfsucation_error = |error: helpers::Error| Error::NoObfuscator(Box::new(error));

        match &query.wireguard_constraints().obfuscation {
            ObfuscationQuery::Off | ObfuscationQuery::Auto => Ok(None),
            ObfuscationQuery::Udp2tcp(settings) => {
                let udp2tcp_ports = &parsed_relays.wireguard.udp2tcp_ports;

                helpers::get_udp2tcp_obfuscator(settings, udp2tcp_ports, obfuscator_relay, endpoint)
                    .map(Some)
                    .map_err(box_obfsucation_error)
            }
            ObfuscationQuery::Shadowsocks(settings) => {
                let port_ranges = &parsed_relays.wireguard.shadowsocks_port_ranges;
                let obfuscation = helpers::get_shadowsocks_obfuscator(
                    settings,
                    port_ranges,
                    obfuscator_relay,
                    endpoint,
                )
                .map_err(box_obfsucation_error)?;

                Ok(Some(obfuscation))
            }
        }
    }

    /// Derive a valid OpenVPN relay configuration from `query`.
    ///
    /// # Note
    /// This function should *only* be called with an OpenVPN query.
    /// It will panic if the tunnel type is not specified as OpenVPN.
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
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<GetRelay, Error> {
        assert_eq!(query.tunnel_protocol(), TunnelType::OpenVpn);
        let exit =
            Self::choose_openvpn_relay(query, custom_lists, parsed_relays).ok_or(Error::NoRelay)?;
        let endpoint = Self::get_openvpn_endpoint(query, &exit, parsed_relays)?;
        let bridge = Self::get_openvpn_bridge(
            query,
            &exit,
            &endpoint.protocol,
            parsed_relays,
            custom_lists,
        )?;

        // FIXME: This assert would be better to encode at the type level.
        assert!(matches!(exit.endpoint_data, RelayEndpointData::Openvpn));
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
        parsed_relays: &RelayList,
    ) -> Result<Endpoint, Error> {
        openvpn_endpoint(query.openvpn_constraints(), &parsed_relays.openvpn, relay).map_err(
            |internal| Error::NoEndpoint {
                internal,
                relay: EndpointErrorDetails::from_openvpn(relay.clone()),
            },
        )
    }

    /// Selects a suitable bridge based on the specified settings, relay information, and transport
    /// protocol.
    ///
    /// # Parameters
    /// - `query`: The filter criteria for selecting a bridge.
    /// - `relay`: Information about the current relay, including its location.
    /// - `protocol`: The transport protocol (TCP or UDP) in use.
    /// - `parsed_relays`: A structured representation of all available relays.
    /// - `custom_lists`: User-defined or application-specific settings that may influence bridge
    ///   selection.
    ///
    /// # Returns
    /// * On success, returns an `Option` containing the selected bridge, if one is found. Returns
    ///   `None` if no suitable bridge meets the criteria or bridges should not be used.
    /// * `Error::NoBridge` if attempting to use OpenVPN bridges over UDP, as this is unsupported.
    /// * `Error::NoRelay` if `relay` does not have a location set.
    #[cfg(not(target_os = "android"))]
    fn get_openvpn_bridge(
        query: &RelayQuery,
        relay: &Relay,
        protocol: &TransportProtocol,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Result<Option<SelectedBridge>, Error> {
        if !BridgeQuery::should_use_bridge(&query.openvpn_constraints().bridge_settings) {
            Ok(None)
        } else {
            let bridge_query = &query.openvpn_constraints().bridge_settings;
            let custom_lists = &custom_lists;
            match protocol {
                TransportProtocol::Udp => {
                    log::error!("Can not use OpenVPN bridges over UDP");
                    Err(Error::NoBridge)
                }
                TransportProtocol::Tcp => {
                    Self::get_bridge_for(
                        bridge_query,
                        &relay.location,
                        // FIXME: This is temporary while talpid-core only supports TCP proxies
                        TransportProtocol::Tcp,
                        parsed_relays,
                        custom_lists,
                    )
                }
            }
        }
    }

    fn get_bridge_for(
        query: &BridgeQuery,
        location: &Location,
        transport_protocol: TransportProtocol,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Result<Option<SelectedBridge>, Error> {
        match query {
            BridgeQuery::Normal(settings) => {
                let bridge_constraints = InternalBridgeConstraints {
                    location: settings.location.clone(),
                    providers: settings.providers.clone(),
                    ownership: settings.ownership,
                    transport_protocol: Constraint::Only(transport_protocol),
                };

                let (settings, relay) = Self::get_proxy_settings(
                    parsed_relays,
                    &bridge_constraints,
                    Some(location),
                    custom_lists,
                )?;
                Ok(Some(SelectedBridge::Normal { settings, relay }))
            }
            BridgeQuery::Custom(settings) => Ok(settings.clone().map(SelectedBridge::Custom)),
            BridgeQuery::Off | BridgeQuery::Auto => Ok(None),
        }
    }

    /// Try to get a bridge that matches the given `constraints`.
    ///
    /// The connection details are returned alongside the relay hosting the bridge.
    fn get_proxy_settings<T: Into<Coordinates>>(
        relay_list: &RelayList,
        constraints: &InternalBridgeConstraints,
        location: Option<T>,
        custom_lists: &CustomListsSettings,
    ) -> Result<(Shadowsocks, Relay), Error> {
        let bridges = filter_matching_bridges(constraints, relay_list.relays(), custom_lists);
        let bridge_data = &relay_list.bridge;
        let bridge = match location {
            Some(location) => Self::get_proximate_bridge(bridges, location),
            None => helpers::pick_random_relay(&bridges)
                .cloned()
                .ok_or(Error::NoRelay),
        }?;
        let endpoint = detailer::bridge_endpoint(bridge_data, &bridge).ok_or(Error::NoBridge)?;
        Ok((endpoint, bridge))
    }

    /// Try to get a bridge which is close to `location`.
    fn get_proximate_bridge<T: Into<Coordinates>>(
        relays: Vec<Relay>,
        location: T,
    ) -> Result<Relay, Error> {
        /// Number of bridges to keep for selection by distance.
        const MIN_BRIDGE_COUNT: usize = 5;
        let location = location.into();

        // Filter out all candidate bridges.
        let matching_bridges: Vec<RelayWithDistance> = relays
            .into_iter()
            .map(|relay| RelayWithDistance::new_with_distance_from(relay, location))
            .sorted_unstable_by_key(|relay| relay.distance as usize)
            .take(MIN_BRIDGE_COUNT)
            .collect();

        // Calculate the maximum distance from `location` among the candidates.
        let greatest_distance: f64 = matching_bridges
            .iter()
            .map(|relay| relay.distance)
            .reduce(f64::max)
            .ok_or(Error::NoBridge)?;
        // Define the weight function to prioritize bridges which are closer to `location`.
        let weight_fn = |relay: &RelayWithDistance| 1 + (greatest_distance - relay.distance) as u64;

        helpers::pick_random_relay_weighted(matching_bridges.iter(), weight_fn)
            .cloned()
            .map(|relay_with_distance| relay_with_distance.relay)
            .ok_or(Error::NoBridge)
    }

    /// Returns the average location of relays that match the given constraints.
    /// This returns `None` if the location is [`Constraint::Any`] or if no
    /// relays match the constraints.
    fn get_relay_midpoint(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Option<Coordinates> {
        use std::ops::Not;
        if query.location().is_any() {
            return None;
        }

        let matching_locations: Vec<Location> =
            filter_matching_relay_list(query, parsed_relays, custom_lists)
                .into_iter()
                .map(|relay| relay.location)
                .unique_by(|location| location.city.clone())
                .collect();

        matching_locations
            .is_empty()
            .not()
            .then(|| Coordinates::midpoint(&matching_locations))
    }

    /// # Returns
    /// A randomly selected relay that meets the specified constraints, or `None` if no suitable
    /// relay is found.
    #[cfg(not(target_os = "android"))]
    fn choose_openvpn_relay(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Option<Relay> {
        // Filter among all valid relays
        let candidates = filter_matching_relay_list(query, parsed_relays, custom_lists);
        // Pick one of the valid relays.
        helpers::pick_random_relay(&candidates).cloned()
    }
}

fn apply_ip_availability(
    runtime_ip_availability: IpAvailability,
    user_query: &mut RelayQuery,
) -> Result<(), Error> {
    let ip_version = match runtime_ip_availability {
        IpAvailability::Ipv4 => Constraint::Only(IpVersion::V4),
        IpAvailability::Ipv6 => Constraint::Only(IpVersion::V6),
        IpAvailability::Ipv4AndIpv6 => Constraint::Any,
    };
    let wireguard_constraints = user_query
        .wireguard_constraints()
        .to_owned()
        .intersection(WireguardRelayQuery {
            ip_version,
            ..Default::default()
        })
        .ok_or_else(|| {
            // It is safe to call `unwrap` on `ip_version` here because this error will only happen
            // when ip_version is Constraint::Only
            let family = user_query.wireguard_constraints().ip_version.unwrap();
            Error::IpVersionUnavailable { family }
        })?;
    user_query.set_wireguard_constraints(wireguard_constraints)?;
    Ok(())
}

#[derive(Clone)]
struct RelayWithDistance {
    distance: f64,
    relay: Relay,
}

impl RelayWithDistance {
    fn new_with_distance_from(relay: Relay, from: impl Into<Coordinates>) -> Self {
        let distance = relay.location.distance_from(from);
        RelayWithDistance { relay, distance }
    }
}
