use crate::location::{CityCode, Coordinates, CountryCode, Location};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::RangeInclusive,
};
use talpid_types::net::{TransportProtocol, proxy::Shadowsocks, wireguard};
use vec1::Vec1;

/// Stores a list of relays for each country obtained from the API using
/// `mullvad_api::RelayListProxy`. This can also be passed to frontends.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub countries: Vec<RelayListCountry>,
    // TODO: Rename to `endpoint(s)`
    pub wireguard: EndpointData,
}

/// Stores a list of bridges for each country obtained from the API using
/// `mullvad_api::RelayListProxy`.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct BridgeList {
    pub bridges: Vec<Bridge>,
    // TODO: Rename to `endpoint(s)`
    pub bridge_endpoint: BridgeEndpointData,
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

    pub fn lookup_country_code_by_name(&self, country_name: &str) -> Option<CountryCode> {
        self.countries
            .iter()
            .find(|country| country.name == country_name)
            .map(|country| country.code.clone())
    }

    /// Returns the closest (geographical distance) country that has a relay for a given location.
    pub fn get_nearest_country_with_relay(
        &self,
        location: impl Into<Coordinates>,
    ) -> Option<CountryCode> {
        if self.countries.is_empty() {
            return None;
        }

        let location = location.into();

        let mut min_dist = f64::MAX;
        let mut min_dist_country = &self.countries[0];

        for country in &self.countries {
            for city in &country.cities {
                for relay in &city.relays {
                    let distance = relay.inner.location.distance_from(location);
                    if distance < min_dist {
                        min_dist = distance;
                        min_dist_country = country;
                    }
                }
            }
        }
        Some(min_dist_country.code.clone())
    }

    /// Return a flat iterator of all [`Relay`]s
    pub fn relays(&self) -> impl Iterator<Item = &WireguardRelay> + Clone + '_ {
        self.countries
            .iter()
            .flat_map(|country| country.cities.iter())
            .flat_map(|city| city.relays.iter())
    }

    /// Return a consuming flat iterator of all [`Relay`]s
    pub fn into_relays(self) -> impl Iterator<Item = WireguardRelay> + Clone {
        self.countries
            .into_iter()
            .flat_map(|country| country.cities)
            .flat_map(|city| city.relays)
    }
}

impl BridgeList {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn bridges(&self) -> &[Bridge] {
        &self.bridges
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
    pub relays: Vec<WireguardRelay>,
}

/// Stores information for a relay returned by the API at `v1/relays` using
/// `mullvad_api::RelayListProxy`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WireguardRelay {
    // NOTE: Probably a better design choice would be to store the overridden IP addresses
    // instead of a boolean override flags. This would allow us to access the original IPs.
    pub overridden_ipv4: bool,
    pub overridden_ipv6: bool,
    pub include_in_country: bool,
    pub owned: bool,
    pub provider: String,
    pub endpoint_data: WireguardRelayEndpointData,
    pub inner: Relay,
}

impl WireguardRelay {
    pub fn new(
        overridden_ipv4: bool,
        overridden_ipv6: bool,
        include_in_country: bool,
        owned: bool,
        provider: String,
        endpoint_data: WireguardRelayEndpointData,
        inner: Relay,
    ) -> Self {
        Self {
            overridden_ipv4,
            overridden_ipv6,
            include_in_country,
            owned,
            provider,
            endpoint_data,
            inner,
        }
    }

    /// If self is a Wireguard relay, we sometimes want to peek on its extra data.
    pub fn endpoint(&self) -> &WireguardRelayEndpointData {
        &self.endpoint_data
    }

    pub fn override_ipv4(&mut self, new_ipv4: Ipv4Addr) {
        self.inner.ipv4_addr_in = new_ipv4;
        self.overridden_ipv4 = true;
    }

    pub fn override_ipv6(&mut self, new_ipv6: Ipv6Addr) {
        self.inner.ipv6_addr_in = Some(new_ipv6);
        self.overridden_ipv6 = true;
    }
}

impl PartialEq for WireguardRelay {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for WireguardRelay {}

impl std::hash::Hash for WireguardRelay {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl std::ops::Deref for WireguardRelay {
    type Target = Relay;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for WireguardRelay {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Stores information for a bridge returned by the API at `v1/relays` using
/// `mullvad_api::RelayListProxy`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bridge(pub Relay);

impl std::ops::Deref for Bridge {
    type Target = Relay;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub ipv6_addr_in: Option<Ipv6Addr>,
    pub active: bool,
    pub weight: u64,
    pub location: Location,
}

/// Parameters for setting up a QUIC obfuscator (connecting to a masque-proxy running on a relay).
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Quic {
    /// In-addresses for the QUIC obfuscator.
    ///
    /// # Note
    ///
    /// This set must be non-empty.
    ///
    /// The primary IPs of the relay will be included if and only if they are listed here.
    addr_in: HashSet<IpAddr>,
    /// Authorization token
    token: String,
    /// Hostname where masque proxy is hosted
    domain: String,
}

impl Quic {
    pub fn new(addr_in: Vec1<IpAddr>, token: String, domain: String) -> Self {
        Self {
            addr_in: addr_in.into_iter().collect(),
            token,
            domain,
        }
    }

    /// Return IPv4 in-addresses.
    ///
    /// Use this if you want to connect to the masque-proxy using IPv4.
    pub fn in_ipv4(&self) -> impl Iterator<Item = Ipv4Addr> {
        let ipv4 = |ipaddr: &IpAddr| match ipaddr {
            IpAddr::V4(ipv4_addr) => Some(*ipv4_addr),
            IpAddr::V6(_) => None,
        };
        self.addr_in.iter().filter_map(ipv4)
    }

    /// Return IPv6 in-addresses.
    ///
    /// Use this if you want to connect to the masque-proxy using IPv6.
    pub fn in_ipv6(&self) -> impl Iterator<Item = Ipv6Addr> {
        let ipv6 = |ipaddr: &IpAddr| match ipaddr {
            IpAddr::V4(_) => None,
            IpAddr::V6(ipv6_addr) => Some(*ipv6_addr),
        };
        self.addr_in.iter().filter_map(ipv6)
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
    ///     #   lwo: false,
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
    Bridge,
    Wireguard(WireguardRelayEndpointData),
}

/// Contains data about all WireGuard endpoints, such as valid port ranges.
#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct EndpointData {
    /// Port to connect to
    pub port_ranges: Vec<RangeInclusive<u16>>,
    /// Gateways to be used with the tunnel
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
    /// Shadowsocks port ranges available on all WireGuard relays
    pub shadowsocks_port_ranges: Vec<RangeInclusive<u16>>,
    pub udp2tcp_ports: Vec<u16>,
}

impl Default for EndpointData {
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
    /// Whether the relay supports LWO
    #[serde(default)]
    pub lwo: bool,
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
            lwo: Default::default(),
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

    pub fn set_lwo(self, enabled: bool) -> Self {
        Self {
            lwo: enabled,
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

#[cfg(test)]
mod test {
    use super::*;
    use talpid_types::net::wireguard::PublicKey;

    #[test]
    fn test_get_nearest_country_with_relay() {
        let location_sweden = Location {
            country: "Sweden".to_string(),
            country_code: "se".to_string(),
            city: "Gothenburg".to_string(),
            city_code: "got".to_string(),
            latitude: 57.71,
            longitude: 11.97,
        };

        let location_japan = Location {
            country: "Japan".to_string(),
            country_code: "jp".to_string(),
            city: "Osaka".to_string(),
            city_code: "osa".to_string(),
            latitude: 34.67231,
            longitude: 135.484802,
        };

        let location_south_korea = Location {
            country: "South Korea".to_string(),
            country_code: "sk".to_string(),
            city: "Seoul".to_string(),
            city_code: "seo".to_string(),
            latitude: 37.532600,
            longitude: 127.024612,
        };

        let location_germany = Location {
            country: "Germany".to_string(),
            country_code: "ger".to_string(),
            city: "Berlin".to_string(),
            city_code: "ber".to_string(),
            latitude: 52.5200080,
            longitude: 13.404954,
        };

        let countries = vec![
            RelayListCountry {
                name: "Sweden".to_string(),
                code: "se".to_string(),
                cities: vec![RelayListCity {
                    name: "Gothenburg".to_string(),
                    code: "got".to_string(),
                    latitude: 57.70887,
                    longitude: 11.97456,
                    relays: vec![WireguardRelay {
                        inner: Relay {
                            hostname: "se9-wireguard".to_string(),
                            ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                            ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                            active: true,
                            location: location_sweden.clone(),
                            weight: 1,
                        },
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: true,
                        owned: true,
                        provider: "provider0".to_string(),
                        endpoint_data: WireguardRelayEndpointData::new(
                            PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                                .unwrap(),
                        ),
                    }],
                }],
            },
            //  34.672314, 135.484802.
            RelayListCountry {
                name: "Japan".to_string(),
                code: "jp".to_string(),
                cities: vec![RelayListCity {
                    name: "Osaka".to_string(),
                    code: "osa".to_string(),
                    latitude: 34.672314,
                    longitude: 135.484802,
                    relays: vec![WireguardRelay {
                        inner: Relay {
                            hostname: "jp9-wireguard".to_string(),
                            ipv4_addr_in: "194.114.136.3".parse().unwrap(),
                            ipv6_addr_in: Some("2404:1b20:5:f011::a09f".parse().unwrap()),
                            active: true,
                            weight: 1,
                            location: location_japan.clone(),
                        },
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: true,
                        owned: true,
                        provider: "provider0".to_string(),
                        endpoint_data: WireguardRelayEndpointData::new(
                            PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                                .unwrap(),
                        ),
                    }],
                }],
            },
        ];

        let relay_list = RelayList {
            countries,
            ..Default::default()
        };

        assert_eq!(
            relay_list.get_nearest_country_with_relay(location_sweden),
            Some("se".to_string())
        );
        assert_eq!(
            relay_list.get_nearest_country_with_relay(location_japan),
            Some("jp".to_string())
        );
        assert_eq!(
            relay_list.get_nearest_country_with_relay(location_germany),
            Some("se".to_string())
        );
        assert_eq!(
            relay_list.get_nearest_country_with_relay(location_south_korea),
            Some("jp".to_string())
        );
    }
}
