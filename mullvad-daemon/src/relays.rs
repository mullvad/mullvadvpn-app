//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use chrono::{DateTime, Local};
use futures::{
    channel::mpsc,
    future::{Fuse, FusedFuture},
    FutureExt, SinkExt, StreamExt,
};
use ipnetwork::IpNetwork;
use mullvad_rpc::{availability::ApiAvailabilityHandle, rest::MullvadRestHandle, RelayListProxy};
use mullvad_types::{
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    location::Location,
    relay_constraints::{
        BridgeState, Constraint, InternalBridgeConstraints, LocationConstraint, Match,
        OpenVpnConstraints, Providers, RelayConstraints, Set, TransportPort, WireguardConstraints,
    },
    relay_list::{Relay, RelayList, RelayTunnels, WireguardEndpointData},
};
use parking_lot::Mutex;
use rand::{self, seq::SliceRandom, Rng};
use std::{
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::{self, Duration, Instant, SystemTime},
};
use talpid_core::future_retry::{retry_future, ExponentialBackoff, Jittered};
use talpid_types::{
    net::{
        all_of_the_internet, openvpn::ProxySettings, wireguard, IpVersion, TransportProtocol,
        TunnelType,
    },
    ErrorExt,
};
use tokio::fs::File;

const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";
const RELAYS_FILENAME: &str = "relays.json";
/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 15);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);

const EXPONENTIAL_BACKOFF_INITIAL: Duration = Duration::from_secs(16);
const EXPONENTIAL_BACKOFF_FACTOR: u32 = 8;
const DEFAULT_WIREGUARD_PORT: u16 = 51820;
const WIREGUARD_EXIT_CONSTRAINTS: WireguardMatcher = WireguardMatcher {
    peer: None,
    port: Constraint::Only(TransportPort {
        protocol: TransportProtocol::Udp,
        port: Constraint::Only(DEFAULT_WIREGUARD_PORT),
    }),
    ip_version: Constraint::Only(IpVersion::V4),
};
const WIREGUARD_TCP_PORTS: [(u16, u16); 3] = [(80, 80), (443, 443), (5001, 5001)];

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open relay cache file")]
    OpenRelayCache(#[error(source)] io::Error),

    #[error(display = "Failed to write relay cache file to disk")]
    WriteRelayCache(#[error(source)] io::Error),

    #[error(display = "No relays matching current constraints")]
    NoRelay,

    #[error(display = "Failure in serialization of the relay list")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Downloader already shut down")]
    DownloaderShutDown,
}

struct ParsedRelays {
    last_updated: SystemTime,
    locations: RelayList,
    relays: Vec<Relay>,
}

impl ParsedRelays {
    pub fn empty() -> Self {
        ParsedRelays {
            last_updated: time::UNIX_EPOCH,
            locations: RelayList::empty(),
            relays: Vec::new(),
        }
    }

    pub fn from_relay_list(relay_list: RelayList, last_updated: SystemTime) -> Self {
        let mut relays = Vec::new();
        for country in &relay_list.countries {
            let country_name = country.name.clone();
            let country_code = country.code.clone();
            for city in &country.cities {
                let city_name = city.name.clone();
                let city_code = city.code.clone();
                let latitude = city.latitude;
                let longitude = city.longitude;
                for relay in &city.relays {
                    let mut relay_with_location = relay.clone();
                    relay_with_location.location = Some(Location {
                        country: country_name.clone(),
                        country_code: country_code.clone(),
                        city: city_name.clone(),
                        city_code: city_code.clone(),
                        latitude,
                        longitude,
                    });

                    for wg_tunnel in &relay.tunnels.wireguard {
                        relay_with_location
                            .tunnels
                            .wireguard
                            .push(WireguardEndpointData {
                                protocol: TransportProtocol::Tcp,
                                port_ranges: WIREGUARD_TCP_PORTS.to_vec(),
                                ..wg_tunnel.clone()
                            });
                    }

                    relays.push(relay_with_location);
                }
            }
        }
        ParsedRelays {
            last_updated,
            locations: relay_list,
            relays,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        log::debug!("Reading relays from {}", path.as_ref().display());
        let (last_modified, file) =
            Self::open_file(path.as_ref()).map_err(Error::OpenRelayCache)?;
        let relay_list =
            serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)?;

        Ok(Self::from_relay_list(relay_list, last_modified))
    }

    fn open_file(path: &Path) -> io::Result<(SystemTime, std::fs::File)> {
        let file = std::fs::File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn locations(&self) -> &RelayList {
        &self.locations
    }

    pub fn relays(&self) -> &Vec<Relay> {
        &self.relays
    }

    pub fn tag(&self) -> Option<&str> {
        self.locations.etag.as_deref()
    }
}

pub struct RelaySelector {
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    updater: Option<RelayListUpdaterHandle>,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk. Use the `update` method
    /// to refresh the relay list from the internet.
    pub fn new(
        rpc_handle: MullvadRestHandle,
        on_update: impl Fn(&RelayList) + Send + 'static,
        resource_dir: &Path,
        cache_dir: &Path,
        api_availability: ApiAvailabilityHandle,
    ) -> Self {
        let cache_path = cache_dir.join(RELAYS_FILENAME);
        let resource_path = resource_dir.join(RELAYS_FILENAME);
        let unsynchronized_parsed_relays = Self::read_relays_from_disk(&cache_path, &resource_path)
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Unable to load cached relays")
                );
                ParsedRelays::empty()
            });
        log::info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().len(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );
        let parsed_relays = Arc::new(Mutex::new(unsynchronized_parsed_relays));

        let updater = RelayListUpdater::new(
            rpc_handle,
            cache_path,
            parsed_relays.clone(),
            Box::new(on_update),
            api_availability,
        );

        RelaySelector {
            parsed_relays,
            updater: Some(updater),
        }
    }

    /// Download the newest relay list.
    pub async fn update(&self) {
        if let Some(mut updater) = self.updater.clone() {
            let _ = updater.update_relay_list().await;
        }
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_locations(&mut self) -> RelayList {
        self.parsed_relays.lock().locations().clone()
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    pub fn get_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
    ) -> Result<RelaySelectorResult, Error> {
        match relay_constraints.tunnel_protocol {
            Constraint::Only(TunnelType::OpenVpn) => self.get_openvpn_endpoint(
                &relay_constraints.location,
                &relay_constraints.providers,
                relay_constraints.openvpn_constraints.clone(),
                bridge_state,
                retry_attempt,
            ),

            Constraint::Only(TunnelType::Wireguard) => self.get_wireguard_endpoint(
                &relay_constraints.location,
                &relay_constraints.providers,
                &relay_constraints.wireguard_constraints,
                retry_attempt,
            ),
            Constraint::Any => self.get_any_tunnel_endpoint(
                relay_constraints,
                bridge_state,
                retry_attempt,
                wg_key_exists,
            ),
        }
    }

    /// Returns an OpenVpn endpoint, should only ever be used when the user has specified the tunnel
    /// protocol as only OpenVPN.
    fn get_openvpn_endpoint(
        &self,
        location: &Constraint<LocationConstraint>,
        providers: &Constraint<Providers>,
        openvpn_constraints: OpenVpnConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        let mut relay_matcher = RelayMatcher {
            location: location.clone(),
            providers: providers.clone(),
            tunnel: openvpn_constraints,
        };

        if relay_matcher.tunnel.port.is_any() && bridge_state == BridgeState::On {
            relay_matcher.tunnel.port = Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            });

            return self.get_tunnel_endpoint_internal(&relay_matcher);
        }

        let mut preferred_relay_matcher = relay_matcher.clone();

        let (preferred_port, preferred_protocol) =
            Self::preferred_openvpn_constraints(retry_attempt);
        let should_try_preferred = match &mut preferred_relay_matcher.tunnel.port {
            any @ Constraint::Any => {
                *any = Constraint::Only(TransportPort {
                    protocol: preferred_protocol,
                    port: preferred_port,
                });
                true
            }
            Constraint::Only(ref mut port_constraints)
                if port_constraints.protocol == preferred_protocol
                    && port_constraints.port.is_any() =>
            {
                port_constraints.port = preferred_port;
                true
            }
            _ => false,
        };

        if should_try_preferred {
            self.get_tunnel_endpoint_internal(&preferred_relay_matcher)
                .or_else(|_| self.get_tunnel_endpoint_internal(&relay_matcher))
        } else {
            self.get_tunnel_endpoint_internal(&relay_matcher)
        }
    }

    fn get_wireguard_multi_hop_endpoint(
        &self,
        mut entry_matcher: RelayMatcher<WireguardMatcher>,
        exit_location: Constraint<LocationConstraint>,
    ) -> Result<RelaySelectorResult, Error> {
        let mut exit_matcher = RelayMatcher {
            location: exit_location,
            tunnel: WIREGUARD_EXIT_CONSTRAINTS.clone().into(),
            ..entry_matcher.clone()
        };

        let (exit_relay, entry_relay, exit_endpoint, mut entry_endpoint) =
            if entry_matcher.location.is_subset(&exit_matcher.location) {
                let (entry_relay, entry_endpoint) = self.get_entry_endpoint(&entry_matcher)?;
                exit_matcher.set_peer(entry_relay.clone());
                let exit_result = self.get_tunnel_endpoint_internal(&exit_matcher)?;
                (
                    exit_result.exit_relay,
                    entry_relay,
                    exit_result.endpoint,
                    entry_endpoint,
                )
            } else {
                let exit_result = self.get_tunnel_endpoint_internal(&exit_matcher)?;

                entry_matcher.set_peer(exit_result.exit_relay.clone());
                let (entry_relay, entry_endpoint) = self.get_entry_endpoint(&entry_matcher)?;
                (
                    exit_result.exit_relay,
                    entry_relay,
                    exit_result.endpoint,
                    entry_endpoint,
                )
            };

        Self::set_entry_peers(
            exit_endpoint
                .get_peer_config()
                .expect("Failed to get peer config from WireGuard endpoint"),
            &mut entry_endpoint,
        );

        log::info!(
            "Selected entry relay {} at {} going through {} at {}",
            entry_relay.hostname,
            entry_endpoint.peer.endpoint.ip(),
            exit_relay.hostname,
            exit_endpoint.to_endpoint().address.ip(),
        );
        let result = RelaySelectorResult::wireguard_endpoint_with_entry(
            exit_relay,
            entry_endpoint,
            entry_relay,
        );
        return Ok(result);
    }

    /// Returns a WireGuard endpoint, should only ever be used when the user has specified the
    /// tunnel protocol as only WireGuard.
    fn get_wireguard_endpoint(
        &self,
        location: &Constraint<LocationConstraint>,
        providers: &Constraint<Providers>,
        wireguard_constraints: &WireguardConstraints,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        let mut entry_relay_matcher = RelayMatcher {
            location: location.clone(),
            providers: providers.clone(),
            tunnel: wireguard_constraints.clone().into(),
        };

        let mut preferred_matcher: RelayMatcher<WireguardMatcher> = entry_relay_matcher.clone();
        preferred_matcher.tunnel.port = preferred_matcher
            .tunnel
            .port
            .or(Self::preferred_wireguard_port(retry_attempt));

        if !wireguard_constraints.use_multihop {
            return self
                .get_tunnel_endpoint_internal(&preferred_matcher)
                .or_else(|_| self.get_tunnel_endpoint_internal(&entry_relay_matcher));
        }

        entry_relay_matcher.location = wireguard_constraints.entry_location.clone();
        self.get_wireguard_multi_hop_endpoint(entry_relay_matcher, location.clone())
    }

    /// Returns a tunnel endpoint of any type, should only be used when the user hasn't specified a
    /// tunnel protocol.
    fn get_any_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
    ) -> Result<RelaySelectorResult, Error> {
        let preferred_constraints = self.preferred_constraints(
            &relay_constraints,
            bridge_state,
            retry_attempt,
            wg_key_exists,
        );
        let original_matcher: RelayMatcher<_> = relay_constraints.clone().into();

        let preferred_tunnel_protocol = preferred_constraints.tunnel_protocol;
        let preferred_matcher: RelayMatcher<_> = preferred_constraints.into();

        match preferred_tunnel_protocol {
            Constraint::Only(TunnelType::Wireguard)
                if relay_constraints.wireguard_constraints.use_multihop =>
            {
                let exit_location = relay_constraints.location.clone();
                let mut preferred_entry_matcher = preferred_matcher.to_wireguard_matcher();
                preferred_entry_matcher.location = relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .clone();
                let mut original_entry_matcher = original_matcher.to_wireguard_matcher();
                original_entry_matcher.location = relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .clone();
                self.get_wireguard_multi_hop_endpoint(
                    preferred_entry_matcher,
                    exit_location.clone(),
                )
                .or_else(|_| {
                    self.get_wireguard_multi_hop_endpoint(original_entry_matcher, exit_location)
                })
            }

            _ => {
                if let Ok(result) = self.get_tunnel_endpoint_internal(&preferred_matcher) {
                    log::debug!(
                        "Relay matched on highest preference for retry attempt {}",
                        retry_attempt
                    );
                    Ok(result)
                } else if let Ok(result) = self.get_tunnel_endpoint_internal(&original_matcher) {
                    log::debug!(
                        "Relay matched on second preference for retry attempt {}",
                        retry_attempt
                    );
                    Ok(result)
                } else {
                    log::warn!("No relays matching {}", &relay_constraints);
                    Err(Error::NoRelay)
                }
            }
        }
    }

    // This function ignores the tunnel type constraint on purpose.
    fn preferred_constraints(
        &self,
        original_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
    ) -> RelayConstraints {
        let (preferred_port, preferred_protocol, preferred_tunnel) = self
            .preferred_tunnel_constraints(
                retry_attempt,
                &original_constraints.location,
                &original_constraints.providers,
                wg_key_exists,
            );

        let mut relay_constraints = original_constraints.clone();
        relay_constraints.openvpn_constraints = Default::default();

        // Highest priority preference. Where we prefer OpenVPN using UDP. But without changing
        // any constraints that are explicitly specified.
        match original_constraints.tunnel_protocol {
            // If no tunnel protocol is selected, use preferred constraints
            Constraint::Any => {
                if bridge_state == BridgeState::On {
                    relay_constraints.openvpn_constraints = OpenVpnConstraints {
                        port: Constraint::Only(TransportPort {
                            protocol: TransportProtocol::Tcp,
                            port: Constraint::Any,
                        }),
                    };
                } else if original_constraints.openvpn_constraints.port.is_any() {
                    relay_constraints.openvpn_constraints = OpenVpnConstraints {
                        port: Constraint::Only(TransportPort {
                            protocol: preferred_protocol,
                            port: preferred_port,
                        }),
                    };
                } else {
                    relay_constraints.openvpn_constraints =
                        original_constraints.openvpn_constraints;
                }

                if relay_constraints.wireguard_constraints.port.is_any() {
                    relay_constraints.wireguard_constraints.port =
                        Constraint::Only(TransportPort {
                            protocol: preferred_protocol,
                            port: preferred_port,
                        });
                }

                relay_constraints.tunnel_protocol = Constraint::Only(preferred_tunnel);
            }
            Constraint::Only(TunnelType::OpenVpn) => {
                let openvpn_constraints = &mut relay_constraints.openvpn_constraints;
                *openvpn_constraints = original_constraints.openvpn_constraints;
                if bridge_state == BridgeState::On && openvpn_constraints.port.is_any() {
                    openvpn_constraints.port = Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Tcp,
                        port: Constraint::Any,
                    });
                } else if openvpn_constraints.port.is_any() {
                    let (preferred_port, preferred_protocol) =
                        Self::preferred_openvpn_constraints(retry_attempt);
                    openvpn_constraints.port = Constraint::Only(TransportPort {
                        protocol: preferred_protocol,
                        port: preferred_port,
                    });
                }
            }
            Constraint::Only(TunnelType::Wireguard) => {
                relay_constraints.wireguard_constraints =
                    original_constraints.wireguard_constraints.clone();
                // This ensures that if after the first 2 failed attempts the daemon does not
                // connect, then afterwards 2 of each 4 successive attempts will try to connect
                // on port 53.
                if retry_attempt % 4 > 1 && relay_constraints.wireguard_constraints.port.is_any() {
                    relay_constraints.wireguard_constraints.port =
                        Constraint::Only(TransportPort {
                            protocol: TransportProtocol::Udp,
                            port: Constraint::Only(53),
                        });
                }
            }
        };

        if relay_constraints.wireguard_constraints.port.is_any() {
            relay_constraints.wireguard_constraints.port = Constraint::Only(TransportPort {
                port: preferred_port,
                protocol: TransportProtocol::Udp,
            });
        }

        relay_constraints.tunnel_protocol = Constraint::Only(preferred_tunnel);

        relay_constraints
    }

    fn get_entry_endpoint(
        &self,
        matcher: &RelayMatcher<WireguardMatcher>,
    ) -> Result<(Relay, MullvadWireguardEndpoint), Error> {
        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| matcher.matching_relay(relay))
            .collect();

        let relay = self
            .pick_random_relay(&matching_relays)
            .map(|relay| relay.clone())
            .ok_or(Error::NoRelay)?;
        let endpoint = matcher.mullvad_endpoint(&relay).ok_or(Error::NoRelay)?;
        match endpoint {
            MullvadEndpoint::Wireguard(endpoint) => Ok((relay, endpoint)),
            _ => {
                unreachable!(
                    "Entry endpoints should only ever be WireGuard endpoints, instead got a {:?}",
                    endpoint
                );
            }
        }
    }

    fn set_entry_peers(
        exit_peer: &wireguard::PeerConfig,
        entry_endpoint: &mut MullvadWireguardEndpoint,
    ) {
        entry_endpoint.peer.allowed_ips = vec![IpNetwork::from(exit_peer.endpoint.ip())];
        entry_endpoint.exit_peer = Some(exit_peer.clone());
    }

    pub fn get_auto_proxy_settings(
        &mut self,
        bridge_constraints: &InternalBridgeConstraints,
        location: &Location,
        retry_attempt: u32,
    ) -> Option<(ProxySettings, Relay)> {
        if !self.should_use_bridge(retry_attempt) {
            return None;
        }

        // For now, only TCP tunnels are supported.
        if let Constraint::Only(TransportProtocol::Udp) = bridge_constraints.transport_protocol {
            return None;
        }

        self.get_proxy_settings(bridge_constraints, location)
    }

    pub fn should_use_bridge(&self, retry_attempt: u32) -> bool {
        // shouldn't use a bridge for the first 3 times
        retry_attempt > 3 &&
            // i.e. 4th and 5th with bridge, 6th & 7th without
            // The test is to see whether the current _couple of connections_ is even or not.
            // | retry_attempt                | 4 | 5 | 6 | 7 | 8 | 9 |
            // | (retry_attempt % 4) < 2      | t | t | f | f | t | t |
            (retry_attempt % 4) < 2
    }

    pub fn get_proxy_settings(
        &mut self,
        constraints: &InternalBridgeConstraints,
        location: &Location,
    ) -> Option<(ProxySettings, Relay)> {
        let mut matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| Self::matching_bridge_relay(relay, constraints))
            .collect();

        if matching_relays.is_empty() {
            return None;
        }

        matching_relays.sort_by_cached_key(|relay| {
            (relay.location.as_ref().unwrap().distance_from(&location) * 1000.0) as i64
        });
        matching_relays.get(0).and_then(|relay| {
            self.pick_random_bridge(&relay)
                .map(|bridge| (bridge, relay.clone()))
        })
    }

    /// Returns preferred constraints
    #[allow(unused_variables)]
    fn preferred_tunnel_constraints(
        &self,
        retry_attempt: u32,
        location_constraint: &Constraint<LocationConstraint>,
        providers_constraint: &Constraint<Providers>,
        wg_key_exists: bool,
    ) -> (Constraint<u16>, TransportProtocol, TunnelType) {
        #[cfg(target_os = "windows")]
        {
            let location_supports_openvpn =
                self.parsed_relays.lock().relays().iter().any(|relay| {
                    relay.active
                        && !relay.tunnels.openvpn.is_empty()
                        && location_constraint.matches(relay)
                        && providers_constraint.matches(relay)
                });
            if location_supports_openvpn {
                let (preferred_port, preferred_protocol) =
                    Self::preferred_openvpn_constraints(retry_attempt);
                return (preferred_port, preferred_protocol, TunnelType::OpenVpn);
            }
        }

        let location_supports_wireguard = self.parsed_relays.lock().relays().iter().any(|relay| {
            relay.active
                && !relay.tunnels.wireguard.is_empty()
                && location_constraint.matches(relay)
                && providers_constraint.matches(relay)
        });
        // If location does not support WireGuard, defer to preferred OpenVPN tunnel
        // constraints
        if !location_supports_wireguard || !wg_key_exists {
            let (preferred_port, preferred_protocol) =
                Self::preferred_openvpn_constraints(retry_attempt);
            return (preferred_port, preferred_protocol, TunnelType::OpenVpn);
        }

        // Try out WireGuard in the first two connection attempts, first with any port,
        // afterwards on port 53. Afterwards, connect through OpenVPN alternating between UDP
        // on any port twice and TCP on port 443 once.
        match retry_attempt {
            0 => (
                Constraint::Any,
                TransportProtocol::Udp,
                TunnelType::Wireguard,
            ),
            1 => (
                Constraint::Only(53),
                TransportProtocol::Udp,
                TunnelType::Wireguard,
            ),
            _ => {
                let (preferred_port, preferred_protocol) =
                    Self::preferred_openvpn_constraints(retry_attempt - 2);
                (preferred_port, preferred_protocol, TunnelType::OpenVpn)
            }
        }
    }

    fn preferred_wireguard_port(retry_attempt: u32) -> Constraint<TransportPort> {
        let port = match retry_attempt % 4 {
            0 | 1 => Constraint::Any,
            _ => Constraint::Only(53),
        };
        Constraint::Only(TransportPort {
            port,
            protocol: TransportProtocol::Udp,
        })
    }

    fn preferred_openvpn_constraints(retry_attempt: u32) -> (Constraint<u16>, TransportProtocol) {
        // Prefer UDP by default. But if that has failed a couple of times, then try TCP port
        // 443, which works for many with UDP problems. After that, just alternate
        // between protocols.
        // If the tunnel type constraint is set OpenVpn, from the 4th attempt onwards, every two
        // retry attempts OpenVpn constraints should be set to TCP as a bridge will be used,
        // and to UDP for the next two attempts. If the tunnel type is specified to be _Any_
        // and on not-Windows, the first two tries are used for WireGuard and don't
        // affect counting here.
        match retry_attempt {
            0 | 1 => (Constraint::Any, TransportProtocol::Udp),
            2 | 3 => (Constraint::Only(443), TransportProtocol::Tcp),
            attempt if attempt % 2 == 0 => (Constraint::Any, TransportProtocol::Udp),
            _ => (Constraint::Any, TransportProtocol::Tcp),
        }
    }

    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal<T: TunnelMatcher>(
        &self,
        matcher: &RelayMatcher<T>,
    ) -> Result<RelaySelectorResult, Error> {
        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| matcher.matching_relay(relay))
            .collect();

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                let endpoint = matcher.mullvad_endpoint(&selected_relay);
                let addr_in = endpoint
                    .as_ref()
                    .map(|endpoint| endpoint.to_endpoint().address.ip())
                    .unwrap_or(IpAddr::from(selected_relay.ipv4_addr_in));
                log::info!("Selected relay {} at {}", selected_relay.hostname, addr_in);
                endpoint.map(|endpoint| RelaySelectorResult::new(endpoint, selected_relay.clone()))
            })
            .ok_or(Error::NoRelay)
    }

    fn matching_bridge_relay(
        relay: &Relay,
        constraints: &InternalBridgeConstraints,
    ) -> Option<Relay> {
        if !constraints.location.matches(relay) {
            return None;
        }
        if !constraints.providers.matches(relay) {
            return None;
        }

        let mut filtered_relay = relay.clone();
        filtered_relay
            .bridges
            .shadowsocks
            .retain(|bridge| constraints.transport_protocol.matches_eq(&bridge.protocol));
        if filtered_relay.bridges.shadowsocks.is_empty() {
            return None;
        }

        Some(filtered_relay)
    }

    /// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
    /// If all of the relays have a weight of 0, one will be picked at random without bias,
    /// otherwise roulette wheel selection will be used to pick only relays with non-zero
    /// weights.
    fn pick_random_relay<'a>(&self, relays: &'a [Relay]) -> Option<&'a Relay> {
        let total_weight: u64 = relays.iter().map(|relay| relay.weight).sum();
        let mut rng = rand::thread_rng();
        if total_weight == 0 {
            relays.choose(&mut rng)
        } else {
            // Pick a random number in the range 1 - total_weight. This choses the relay with a
            // non-zero weight.
            let mut i: u64 = rng.gen_range(1, total_weight + 1);
            Some(
                relays
                    .iter()
                    .find(|relay| {
                        i = i.saturating_sub(relay.weight);
                        i == 0
                    })
                    .expect("At least one relay must've had a weight above 0"),
            )
        }
    }

    /// Picks a random bridge from a relay.
    fn pick_random_bridge(&self, relay: &Relay) -> Option<ProxySettings> {
        relay
            .bridges
            .shadowsocks
            .choose(&mut rand::thread_rng())
            .map(|shadowsocks_endpoint| {
                log::info!(
                    "Selected Shadowsocks bridge {} at {}:{}/{}",
                    relay.hostname,
                    relay.ipv4_addr_in,
                    shadowsocks_endpoint.port,
                    shadowsocks_endpoint.protocol
                );
                shadowsocks_endpoint
                    .clone()
                    .to_proxy_settings(relay.ipv4_addr_in.into())
            })
    }

    /// Try to read the relays from disk, preferring the newer ones.
    fn read_relays_from_disk(
        cache_path: &Path,
        resource_path: &Path,
    ) -> Result<ParsedRelays, Error> {
        // prefer the resource path's relay list if the cached one doesn't exist or was modified
        // before the resource one was created.
        let cached_relays = ParsedRelays::from_file(cache_path);
        let bundled_relays = match ParsedRelays::from_file(resource_path) {
            Ok(bundled_relays) => bundled_relays,
            Err(e) => {
                log::error!("Failed to load bundled relays: {}", e);
                return cached_relays;
            }
        };

        if cached_relays
            .as_ref()
            .map(|cached| cached.last_updated > bundled_relays.last_updated)
            .unwrap_or(false)
        {
            cached_relays
        } else {
            Ok(bundled_relays)
        }
    }
}

#[derive(Debug)]
pub struct RelaySelectorResult {
    pub exit_relay: Relay,
    pub endpoint: MullvadEndpoint,
    pub entry_relay: Option<Relay>,
}

impl RelaySelectorResult {
    fn new(endpoint: MullvadEndpoint, exit_relay: Relay) -> Self {
        Self {
            exit_relay,
            endpoint,
            entry_relay: None,
        }
    }

    fn wireguard_endpoint_with_entry(
        exit_relay: Relay,
        endpoint: MullvadWireguardEndpoint,
        entry: Relay,
    ) -> Self {
        Self {
            exit_relay,
            endpoint: MullvadEndpoint::Wireguard(endpoint),
            entry_relay: Some(entry),
        }
    }
}

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

struct RelayListUpdater {
    rpc_client: RelayListProxy,
    cache_path: PathBuf,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
    earliest_next_try: Instant,
    api_availability: ApiAvailabilityHandle,
}

impl RelayListUpdater {
    pub fn new(
        rpc_handle: MullvadRestHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
        on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
        api_availability: ApiAvailabilityHandle,
    ) -> RelayListUpdaterHandle {
        let (tx, cmd_rx) = mpsc::channel(1);
        let service = rpc_handle.service();
        let rpc_client = RelayListProxy::new(rpc_handle);
        let updater = RelayListUpdater {
            rpc_client,
            cache_path,
            parsed_relays,
            on_update,
            earliest_next_try: Instant::now() + UPDATE_INTERVAL,
            api_availability,
        };

        service.spawn(updater.run(cmd_rx));

        RelayListUpdaterHandle { tx }
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<()>) {
        let mut check_interval =
            tokio_stream::wrappers::IntervalStream::new(tokio::time::interval_at(
                (Instant::now() + UPDATE_CHECK_INTERVAL).into(),
                UPDATE_CHECK_INTERVAL,
            ))
            .fuse();
        let mut download_future = Box::pin(Fuse::terminated());
        loop {
            futures::select! {
                _check_update = check_interval.next() => {
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

#[derive(Clone)]
struct RelayMatcher<T: TunnelMatcher> {
    location: Constraint<LocationConstraint>,
    providers: Constraint<Providers>,
    tunnel: T,
}

impl From<RelayConstraints> for RelayMatcher<AnyTunnelMatcher> {
    fn from(constraints: RelayConstraints) -> Self {
        Self {
            location: constraints.location,
            providers: constraints.providers,
            tunnel: AnyTunnelMatcher {
                wireguard: constraints.wireguard_constraints.into(),
                openvpn: constraints.openvpn_constraints,
                tunnel_type: constraints.tunnel_protocol,
            },
        }
    }
}

impl RelayMatcher<AnyTunnelMatcher> {
    fn to_wireguard_matcher(self) -> RelayMatcher<WireguardMatcher> {
        let Self {
            location,
            providers,
            tunnel,
        } = self;

        let tunnel = tunnel.wireguard;
        RelayMatcher {
            tunnel,
            location,
            providers,
        }
    }
}

impl RelayMatcher<WireguardMatcher> {
    pub fn set_peer(&mut self, peer: Relay) {
        self.tunnel.peer = Some(peer);
    }
}

impl<T: TunnelMatcher> RelayMatcher<T> {
    fn matching_relay(&self, relay: &Relay) -> Option<Relay> {
        if !self.location.matches(relay) || !self.providers.matches(relay) {
            return None;
        }

        self.tunnel.matching_relay(relay)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        self.tunnel.mullvad_endpoint(relay)
    }
}
/// TunnelMatcher allows to abstract over different tunnel-specific constraints,
/// as to not have false dependencies on OpenVpn specific constraints when
/// selecting only WireGuard tunnels.
trait TunnelMatcher: Clone {
    /// Check if given relay matches tunnel-specific constraints.
    fn matching_relay(&self, relay: &Relay) -> Option<Relay>;
    /// Constructs a MullvadEndpoint for a given Relay using extra data from the relay matcher
    /// itself.
    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint>;
}

impl TunnelMatcher for OpenVpnConstraints {
    fn matching_relay(&self, relay: &Relay) -> Option<Relay> {
        let tunnels = relay
            .tunnels
            .openvpn
            .iter()
            .filter(|endpoint| self.matches(endpoint))
            .cloned()
            .collect::<Vec<_>>();
        if tunnels.is_empty() {
            return None;
        }
        let mut relay = relay.clone();
        relay.tunnels = RelayTunnels {
            openvpn: tunnels,
            wireguard: vec![],
        };
        Some(relay)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        relay
            .tunnels
            .openvpn
            .choose(&mut rand::thread_rng())
            .cloned()
            .map(|endpoint| endpoint.into_mullvad_endpoint(relay.ipv4_addr_in.into()))
    }
}

#[derive(Clone)]
struct AnyTunnelMatcher {
    wireguard: WireguardMatcher,
    openvpn: OpenVpnConstraints,
    /// in the case that a user hasn't specified a tunnel protocol, the relay
    /// selector might still construct preferred constraints that do select a
    /// specific tunnel protocol, which is why the tunnel type may be specified
    /// in the `AnyTunnelMatcher`.
    tunnel_type: Constraint<TunnelType>,
}

impl TunnelMatcher for AnyTunnelMatcher {
    fn matching_relay(&self, relay: &Relay) -> Option<Relay> {
        match self.tunnel_type {
            Constraint::Any => {
                let wireguard_relay = self.wireguard.matching_relay(relay);
                let openvpn_relay = self.openvpn.matching_relay(relay);

                match (wireguard_relay, openvpn_relay) {
                    (Some(mut matched_relay), Some(openvpn_relay)) => {
                        matched_relay.tunnels.openvpn = openvpn_relay.tunnels.openvpn;
                        Some(matched_relay)
                    }
                    (Some(relay), None) | (None, Some(relay)) => Some(relay),
                    _ => None,
                }
            }
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.matching_relay(relay),
            Constraint::Only(TunnelType::Wireguard) => self.wireguard.matching_relay(relay),
        }
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        #[cfg(not(target_os = "android"))]
        match self.tunnel_type {
            Constraint::Any => vec![
                self.openvpn.mullvad_endpoint(relay),
                self.wireguard.mullvad_endpoint(relay),
            ]
            .into_iter()
            .filter_map(|relay| relay)
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .cloned(),
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.mullvad_endpoint(relay),
            Constraint::Only(TunnelType::Wireguard) => self.wireguard.mullvad_endpoint(relay),
        }

        #[cfg(target_os = "android")]
        self.wireguard_matcher.mullvad_endpoint(relay)
    }
}

#[derive(Clone)]
struct WireguardMatcher {
    peer: Option<Relay>,
    port: Constraint<TransportPort>,
    ip_version: Constraint<IpVersion>,
}

impl WireguardMatcher {
    fn wg_data_to_endpoint(
        &self,
        relay: &Relay,
        data: WireguardEndpointData,
    ) -> Option<MullvadEndpoint> {
        let host = self.get_address_for_wireguard_relay(relay)?;
        let port = self.get_port_for_wireguard_relay(&data)?;
        let peer_config = wireguard::PeerConfig {
            public_key: data.public_key,
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
            protocol: self
                .port
                .map(|port| port.protocol)
                .unwrap_or(TransportProtocol::Udp),
        };
        Some(MullvadEndpoint::Wireguard(MullvadWireguardEndpoint {
            peer: peer_config,
            exit_peer: None,
            ipv4_gateway: data.ipv4_gateway,
            ipv6_gateway: data.ipv6_gateway,
        }))
    }

    fn get_address_for_wireguard_relay(&self, relay: &Relay) -> Option<IpAddr> {
        match self.ip_version {
            Constraint::Any | Constraint::Only(IpVersion::V4) => Some(relay.ipv4_addr_in.into()),
            Constraint::Only(IpVersion::V6) => relay.ipv6_addr_in.map(|addr| addr.into()),
        }
    }

    fn get_port_for_wireguard_relay(&self, data: &WireguardEndpointData) -> Option<u16> {
        match self
            .port
            .as_ref()
            .map(|port| port.port)
            .unwrap_or(Constraint::Any)
        {
            Constraint::Any => {
                let get_port_amount =
                    |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
                let port_amount: u64 = data.port_ranges.iter().map(get_port_amount).sum();

                if port_amount < 1 {
                    return None;
                }

                let mut port_index = rand::thread_rng().gen_range(0, port_amount);

                for range in data.port_ranges.iter() {
                    let ports_in_range = get_port_amount(range);
                    if port_index < ports_in_range {
                        return Some(port_index as u16 + range.0);
                    }
                    port_index -= ports_in_range;
                }
                log::error!("Port selection algorithm is broken!");
                None
            }
            Constraint::Only(port) => {
                if data
                    .port_ranges
                    .iter()
                    .any(|range| (range.0 <= port && port <= range.1))
                {
                    Some(port)
                } else {
                    None
                }
            }
        }
    }
}

impl From<WireguardConstraints> for WireguardMatcher {
    fn from(constraints: WireguardConstraints) -> Self {
        Self {
            peer: None,
            port: constraints.port,
            ip_version: constraints.ip_version,
        }
    }
}

impl Match<WireguardEndpointData> for WireguardMatcher {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        match self
            .port
            .as_ref()
            .map(|port| port.port)
            .unwrap_or(Constraint::Any)
        {
            Constraint::Any => true,
            Constraint::Only(port) => endpoint
                .port_ranges
                .iter()
                .any(|range| (port >= range.0 && port <= range.1)),
        }
    }
}

impl TunnelMatcher for WireguardMatcher {
    fn matching_relay(&self, relay: &Relay) -> Option<Relay> {
        if self
            .peer
            .as_ref()
            .map(|peer_relay| peer_relay.hostname == relay.hostname)
            .unwrap_or(false)
        {
            return None;
        }

        let tunnels = relay
            .tunnels
            .wireguard
            .iter()
            .filter(|endpoint| self.matches(*endpoint))
            .cloned()
            .collect::<Vec<_>>();
        if tunnels.is_empty() {
            return None;
        }
        let mut relay = relay.clone();
        relay.tunnels = RelayTunnels {
            wireguard: tunnels,
            openvpn: vec![],
        };
        Some(relay)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        relay
            .tunnels
            .wireguard
            .choose(&mut rand::thread_rng())
            .cloned()
            .and_then(|wg_tunnel| self.wg_data_to_endpoint(relay, wg_tunnel))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mullvad_types::{
        relay_constraints::RelayConstraints,
        relay_list::{
            OpenVpnEndpointData, Relay, RelayBridges, RelayListCity, RelayListCountry,
            RelayTunnels, WireguardEndpointData,
        },
    };
    use talpid_types::net::wireguard::PublicKey;

    lazy_static::lazy_static! {
        static ref RELAYS: RelayList = RelayList {
            etag: None,
            countries: vec![
                RelayListCountry {
                    name: "Sweden".to_string(),
                    code: "se".to_string(),
                    cities: vec![
                        RelayListCity {
                            name: "Gothenburg".to_string(),
                            code: "got".to_string(),
                            latitude: 57.70887,
                            longitude: 11.97456,
                            relays: vec![
                                Relay {
                                    hostname: "se9-wireguard".to_string(),
                                    ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                                    ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![],
                                        wireguard: vec![
                                            WireguardEndpointData {
                                                port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
                                                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                                                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                                                public_key: PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap(),
                                                protocol: TransportProtocol::Udp,
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se10-wireguard".to_string(),
                                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                                    ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![],
                                        wireguard: vec![
                                            WireguardEndpointData {
                                                port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
                                                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                                                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                                                public_key: PublicKey::from_base64("veGD6/aEY6sMfN3Ls7YWPmNgu3AheO7nQqsFT47YSws=").unwrap(),
                                                protocol: TransportProtocol::Udp,
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se-got-001".to_string(),
                                    ipv4_addr_in: "185.213.154.131".parse().unwrap(),
                                    ipv6_addr_in: None,
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![
                                            OpenVpnEndpointData {
                                                port: 1194,
                                                protocol: TransportProtocol::Udp,
                                            },
                                            OpenVpnEndpointData {
                                                port: 443,
                                                protocol: TransportProtocol::Tcp,
                                            },
                                            OpenVpnEndpointData {
                                                port: 80,
                                                protocol: TransportProtocol::Tcp,
                                            },
                                        ],
                                        wireguard: vec![],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                            ],
                        },
                    ],
                }
            ],
        };
    }

    fn new_relay_selector() -> RelaySelector {
        RelaySelector {
            parsed_relays: Arc::new(Mutex::new(ParsedRelays::from_relay_list(
                RELAYS.clone(),
                SystemTime::now(),
            ))),
            updater: None,
        }
    }

    #[test]
    fn test_preferred_tunnel_protocol() {
        let relay_selector = new_relay_selector();

        // Prefer WG if the location only supports it
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(&relay_constraints, BridgeState::Off, attempt, true)
                .is_ok());
        }

        // Prefer OpenVPN if the location only supports it
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(&relay_constraints, BridgeState::Off, attempt, true)
                .is_ok());
        }

        // Prefer OpenVPN on Windows when possible
        #[cfg(windows)]
        {
            let relay_constraints = RelayConstraints::default();
            for attempt in 0..10 {
                let preferred = relay_selector.preferred_constraints(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    true,
                );
                assert_eq!(
                    preferred.tunnel_protocol,
                    Constraint::Only(TunnelType::OpenVpn)
                );
                match relay_selector.get_any_tunnel_endpoint(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    true,
                ) {
                    Ok(result) if matches!(result.endpoint, MullvadEndpoint::OpenVpn(_)) => (),
                    _ => panic!("OpenVPN endpoint was not selected"),
                }
            }
        }
    }

    #[test]
    fn test_wg_entry_hostname_collision() {
        let relay_selector = new_relay_selector();

        let location1 = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let location2 = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        );

        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location1.clone()),
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        relay_constraints.wireguard_constraints.use_multihop = true;
        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location1);

        // The same host cannot be used for entry and exit
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .is_err());

        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location2);

        // If the entry and exit differ, this should succeed
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .is_ok());
    }

    #[test]
    fn test_wg_entry_filter() -> Result<(), String> {
        let relay_selector = new_relay_selector();

        let specific_hostname = "se10-wireguard";

        let location_general = LocationConstraint::City("se".to_string(), "got".to_string());
        let location_specific = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            specific_hostname.to_string(),
        );

        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location_general.clone()),
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        relay_constraints.wireguard_constraints.use_multihop = true;
        relay_constraints.wireguard_constraints.entry_location =
            Constraint::Only(location_specific.clone());

        // The exit must not equal the entry
        let exit_relay = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .map_err(|error| error.to_string())?
            .exit_relay;

        assert_ne!(exit_relay.hostname, specific_hostname);

        relay_constraints.location = Constraint::Only(location_specific);
        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location_general);

        // The entry must not equal the exit
        let RelaySelectorResult {
            exit_relay,
            endpoint,
            ..
        } = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .map_err(|error| error.to_string())?;

        assert_eq!(exit_relay.hostname, specific_hostname);
        match endpoint {
            MullvadEndpoint::OpenVpn { .. } => return Err("Expected WireGuard relay".to_string()),
            MullvadEndpoint::Wireguard(endpoint) => {
                assert_eq!(
                    exit_relay.ipv4_addr_in,
                    endpoint.exit_peer.unwrap().endpoint.ip()
                );
                assert_ne!(exit_relay.ipv4_addr_in, endpoint.peer.endpoint.ip());
            }
        }

        Ok(())
    }

    #[test]
    fn test_bridge_constraints() -> Result<(), String> {
        let relay_selector = new_relay_selector();

        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        );
        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        relay_constraints.openvpn_constraints.port = Constraint::Only(TransportPort {
            protocol: TransportProtocol::Udp,
            port: Constraint::Any,
        });

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );
        // NOTE: TCP is preferred for bridges
        assert_eq!(
            preferred.openvpn_constraints.port,
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            })
        );

        // Ignore bridge state where WireGuard is used
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        // Handle bridge setting when falling back on OpenVPN
        let mut relay_constraints = RelayConstraints {
            location: Constraint::Any,
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        relay_constraints.openvpn_constraints.port = Constraint::Only(TransportPort {
            protocol: TransportProtocol::Udp,
            port: Constraint::Any,
        });
        #[cfg(all(unix, not(target_os = "android")))]
        {
            let preferred =
                relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0, true);
            assert_eq!(
                preferred.tunnel_protocol,
                Constraint::Only(TunnelType::Wireguard)
            );
        }
        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 2, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );
        assert_eq!(
            preferred.openvpn_constraints.port,
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            })
        );

        Ok(())
    }

    #[test]
    fn test_wg_relay_with_no_key() {
        let mut relay_constraints = RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        let relay_selector = new_relay_selector();

        let result = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, false)
            .expect("Failed to get WireGuard relay when WireGuard relay was specified as the only tunnel protocol");

        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard(_)));

        relay_constraints.tunnel_protocol = Constraint::Any;
        let result = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, false)
            .expect("Failed to get OpenVPN relay with tunnel protocol constraint set to Any and without a WireGuard key");

        assert!(matches!(result.endpoint, MullvadEndpoint::OpenVpn(_)));

        let wireguard_specific_location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        relay_constraints.location = Constraint::Only(wireguard_specific_location);

        let result = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, false)
            .expect(
                "Failed to get a valid WireGuard relay when tunnel constraints are set to any
                tunnel protocol and with a wireguard specific location without a wireguard key",
            );

        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard(_)));

        let result = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .expect(
                "Failed to get a valid WireGuard relay when tunnel constraints are set to any
                tunnel protocol and with a wireguard specific location with a wireguard key",
            );

        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard(_)));
    }

    #[test]
    fn test_selecting_any_relay_will_consider_multihop() {
        let relay_constraints = RelayConstraints {
            wireguard_constraints: WireguardConstraints {
                use_multihop: true,
                ..WireguardConstraints::default()
            },
            // This has to be explicit otherwise Android will chose WireGuard when default
            // constructing.
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .expect("Failed to get relay when tunnel constraints are set to Any and retrying the selection");
        // Windows will ignore WireGuard until WireGuard is supported well enough
        // TODO: Remove this caveat once Windows defaults to using WireGuard
        #[cfg(target_os = "windows")]
        assert!(
            matches!(result.endpoint, MullvadEndpoint::OpenVpn(_)) && result.entry_relay.is_none()
        );

        #[cfg(not(target_os = "windows"))]
        assert!(
            matches!(result.endpoint, MullvadEndpoint::Wireguard(_))
                && result.entry_relay.is_some()
        );
    }

    #[test]
    fn test_selecting_wireguard_location_will_consider_multihop() {
        let wireguard_specific_location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );

        let relay_constraints = RelayConstraints {
            location: Constraint::Only(wireguard_specific_location),
            wireguard_constraints: WireguardConstraints {
                use_multihop: true,
                ..WireguardConstraints::default()
            },
            // This has to be explicit otherwise Android will chose WireGuard when default
            // constructing.
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .expect("Failed to get relay when tunnel constraints are set to Any and retrying the selection");
        assert!(
            matches!(result.endpoint, MullvadEndpoint::Wireguard(_))
                && result.entry_relay.is_some()
        );
    }
}
