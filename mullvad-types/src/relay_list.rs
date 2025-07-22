use crate::location::{CityCode, CountryCode, Location};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::RangeInclusive,
};
use talpid_types::net::{TransportProtocol, proxy::Shadowsocks, wireguard};

/// Stores a list of relays for each country obtained from the API using
/// `mullvad_api::RelayListProxy`. This can also be passed to frontends.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub etag: Option<String>,
    pub countries: Vec<RelayListCountry>,
    #[serde(rename = "openvpn")]
    pub openvpn: OpenVpnEndpointData,
    pub bridge: BridgeEndpointData,
    pub wireguard: WireguardEndpointData,
}

impl RelayList {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn lookup_country(&self, country_code: CountryCode) -> Option<&RelayListCountry> {
        self.countries
            .iter()
            .find(|country| country.code == country_code)
    }

    /// Return a flat iterator of all [`Relay`]s
    pub fn relays(&self) -> impl Iterator<Item = &Relay> + Clone + '_ {
        self.countries
            .iter()
            .flat_map(|country| country.cities.iter())
            .flat_map(|city| city.relays.iter())
    }

    /// Return a consuming flat iterator of all [`Relay`]s
    pub fn into_relays(self) -> impl Iterator<Item = Relay> + Clone {
        self.countries
            .into_iter()
            .flat_map(|country| country.cities)
            .flat_map(|city| city.relays)
    }
}

/// A list of [`RelayListCity`]s within a country. Used by [`RelayList`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCountry {
    pub name: String,
    pub code: CountryCode,
    pub cities: Vec<RelayListCity>,
}

impl RelayListCountry {
    pub fn lookup_city(&self, city_code: CityCode) -> Option<&RelayListCity> {
        self.cities.iter().find(|city| city.code == city_code)
    }
}

/// A list of [`Relay`]s within a city. Used by [`RelayListCountry`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCity {
    pub name: String,
    pub code: CityCode,
    pub latitude: f64,
    pub longitude: f64,
    pub relays: Vec<Relay>,
}

/// Stores information for a relay returned by the API at `v1/relays` using
/// `mullvad_api::RelayListProxy`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub ipv6_addr_in: Option<Ipv6Addr>,
    // NOTE: Probably a better design choice would be to store the overridden IP addresses
    // instead of a boolean override flags. This would allow us to access the original IPs.
    pub overridden_ipv4: bool,
    pub overridden_ipv6: bool,
    pub include_in_country: bool,
    pub active: bool,
    pub owned: bool,
    pub provider: String,
    pub weight: u64,
    pub endpoint_data: RelayEndpointData,
    pub location: Location,
}

impl Relay {
    /// If self is a Wireguard relay, we sometimes want to peek on its extra data.
    pub fn wireguard(&self) -> Option<&WireguardRelayEndpointData> {
        match &self.endpoint_data {
            RelayEndpointData::Wireguard(wireguard_relay_endpoint_data) => {
                Some(wireguard_relay_endpoint_data)
            }
            RelayEndpointData::Openvpn | RelayEndpointData::Bridge => None,
        }
    }
}

/// Parameters for setting up a QUIC obfuscator (connecting to a masque-proxy running on a relay).
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Quic {
    /// In-addresses for the QUIC obfuscator.
    ///
    /// There may be 0, 1 or 2 in IPs, depending on how many masque-proxy daemons running on the
    /// relay. Hopefully the API will tell use the correct amount🤞.
    addr_in: HashSet<IpAddr>,
    /// Authorization token
    token: String,
    /// Hostname where masque proxy is hosted
    domain: String,
}

impl Quic {
    pub fn new(addr_in: impl IntoIterator<Item = IpAddr>, token: String, domain: String) -> Self {
        let addr_in = HashSet::from_iter(addr_in);
        Self {
            addr_in,
            token,
            domain,
        }
    }

    /// In address as an IPv4 address.
    ///
    /// Use this if you want to connect to the masque-proxy using IPv4.
    pub fn in_ipv4(&self) -> Option<Ipv4Addr> {
        let ipv4 = |ipaddr: &IpAddr| match ipaddr {
            IpAddr::V4(ipv4_addr) => Some(*ipv4_addr),
            IpAddr::V6(_) => None,
        };
        self.addr_in.iter().find_map(ipv4)
    }

    /// In address as an IPv6 address.
    ///
    /// Use this if you want to connect to the masque-proxy using IPv6.
    pub fn in_ipv6(&self) -> Option<Ipv6Addr> {
        let ipv6 = |ipaddr: &IpAddr| match ipaddr {
            IpAddr::V4(_) => None,
            IpAddr::V6(ipv6_addr) => Some(*ipv6_addr),
        };
        self.addr_in.iter().find_map(ipv6)
    }

    /// Port of the masque-proxy daemon.
    pub const fn port(&self) -> u16 {
        // The point of the masque-proxy is to look like a regular web server serving http traffic.
        443
    }

    pub fn hostname(&self) -> &str {
        &self.domain
    }

    pub fn auth_token(&self) -> &str {
        &self.token
    }

    pub fn in_addr(&self) -> impl Iterator<Item = IpAddr> {
        self.addr_in.iter().copied()
    }
}

impl Relay {
    pub fn override_ipv4(&mut self, new_ipv4: Ipv4Addr) {
        self.ipv4_addr_in = new_ipv4;
        self.overridden_ipv4 = true;
    }

    pub fn override_ipv6(&mut self, new_ipv6: Ipv6Addr) {
        self.ipv6_addr_in = Some(new_ipv6);
        self.overridden_ipv6 = true;
    }

    pub const fn is_wireguard(&self) -> bool {
        matches!(self.endpoint_data, RelayEndpointData::Wireguard(_))
    }

    pub const fn is_openvpn(&self) -> bool {
        matches!(self.endpoint_data, RelayEndpointData::Openvpn)
    }
}

impl PartialEq for Relay {
    /// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its
    /// hostname.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mullvad_types::{relay_list::Relay, relay_list::{RelayEndpointData, WireguardRelayEndpointData}};
    /// # use talpid_types::net::wireguard::PublicKey;
    ///
    /// let relay = Relay {
    ///     hostname: "se9-wireguard".to_string(),
    ///     ipv4_addr_in: "185.213.154.68".parse().unwrap(),
    ///     # ipv6_addr_in: None,
    ///     # overridden_ipv4: false,
    ///     # overridden_ipv6: false,
    ///     # include_in_country: true,
    ///     # active: true,
    ///     # owned: true,
    ///     # provider: "provider0".to_string(),
    ///     # weight: 1,
    ///     # endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
    ///     #   public_key: PublicKey::from_base64(
    ///     #       "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
    ///     #   )
    ///     #   .unwrap(),
    ///     #   daita: false,
    ///     #   shadowsocks_extra_addr_in: Default::default(),
    ///     #   quic: None,
    ///     # }),
    ///     # location: mullvad_types::location::Location {
    ///     #   country: "Sweden".to_string(),
    ///     #   country_code: "se".to_string(),
    ///     #   city: "Gothenburg".to_string(),
    ///     #   city_code: "got".to_string(),
    ///     #   latitude: 57.71,
    ///     #   longitude: 11.97,
    ///     # },
    /// };
    ///
    /// let mut different_relay = relay.clone();
    /// // Modify the relay's IPv4 address - should not matter for the equality check
    /// different_relay.ipv4_addr_in = "1.3.3.7".parse().unwrap();
    /// assert_eq!(relay, different_relay);
    ///
    /// // What matter's for the equality check is the hostname of the relay
    /// different_relay.hostname = "dk-cph-wg-001".to_string();
    /// assert_ne!(relay, different_relay);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.hostname == other.hostname
    }
}

/// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its
/// hostname.
impl Eq for Relay {}

/// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its
/// hostname.
impl std::hash::Hash for Relay {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hostname.hash(state)
    }
}

/// Specifies the type of a relay or relay-specific endpoint data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayEndpointData {
    Openvpn,
    Bridge,
    Wireguard(WireguardRelayEndpointData),
}

/// Data needed to connect to OpenVPN endpoints.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpointData {
    pub ports: Vec<OpenVpnEndpoint>,
}

/// Data needed to connect to OpenVPN endpoints.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpoint {
    pub port: u16,
    pub protocol: TransportProtocol,
}

/// Contains data about all WireGuard endpoints, such as valid port ranges.
#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port_ranges: Vec<RangeInclusive<u16>>,
    /// Gateways to be used with the tunnel
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
    /// Shadowsocks port ranges available on all WireGuard relays
    pub shadowsocks_port_ranges: Vec<RangeInclusive<u16>>,
    pub udp2tcp_ports: Vec<u16>,
}

impl Default for WireguardEndpointData {
    fn default() -> Self {
        Self {
            port_ranges: vec![],
            ipv4_gateway: "0.0.0.0".parse().unwrap(),
            ipv6_gateway: "::".parse().unwrap(),
            shadowsocks_port_ranges: vec![],
            udp2tcp_ports: vec![],
        }
    }
}

/// Contains data about specific WireGuard endpoints, i.e. their public keys.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub struct WireguardRelayEndpointData {
    /// Public key used by the relay peer
    pub public_key: wireguard::PublicKey,
    /// Whether the relay supports DAITA
    #[serde(default)]
    pub daita: bool,
    /// Parameters for connecting to the masque-proxy running on the relay.
    #[serde(default)]
    pub quic: Option<Quic>,
    /// Optional IP addresses used by Shadowsocks
    #[serde(default)]
    pub shadowsocks_extra_addr_in: HashSet<IpAddr>,
}

impl WireguardRelayEndpointData {
    pub fn new(public_key: wireguard::PublicKey) -> Self {
        Self {
            public_key,
            daita: Default::default(),
            quic: Default::default(),
            shadowsocks_extra_addr_in: Default::default(),
        }
    }

    pub fn set_daita(self, enabled: bool) -> Self {
        Self {
            daita: enabled,
            ..self
        }
    }

    pub fn set_quic(self, quic: Quic) -> Self {
        Self {
            quic: Some(quic),
            ..self
        }
    }

    /// Add `in_addrs` to the existing shadowsocks extra in addressess.
    pub fn add_shadowsocks_extra_in_addrs(self, in_addrs: impl Iterator<Item = IpAddr>) -> Self {
        let in_addrs = self.shadowsocks_extra_in_addrs().copied().chain(in_addrs);
        Self {
            shadowsocks_extra_addr_in: HashSet::from_iter(in_addrs),
            ..self
        }
    }

    pub fn shadowsocks_extra_in_addrs(&self) -> impl Iterator<Item = &IpAddr> {
        self.shadowsocks_extra_addr_in.iter()
    }

    // Is this really needed if `self.quic` is pub?
    pub fn quic(&self) -> Option<&Quic> {
        self.quic.as_ref()
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct BridgeEndpointData {
    pub shadowsocks: Vec<ShadowsocksEndpointData>,
}

/// Data needed to connect to Shadowsocks endpoints.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ShadowsocksEndpointData {
    pub port: u16,
    pub cipher: String,
    pub password: String,
    pub protocol: TransportProtocol,
}

impl ShadowsocksEndpointData {
    pub fn to_proxy_settings(&self, addr: IpAddr) -> Shadowsocks {
        Shadowsocks {
            endpoint: SocketAddr::new(addr, self.port),
            password: self.password.clone(),
            cipher: self.cipher.clone(),
        }
    }
}
