use crate::location::{CityCode, CountryCode, Location};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use talpid_types::net::{
    proxy::{CustomProxy, Shadowsocks},
    wireguard, TransportProtocol,
};

/// Stores a list of relays for each country obtained from the API using
/// `mullvad_api::RelayListProxy`. This can also be passed to frontends.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayList {
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub etag: Option<String>,
    pub countries: Vec<RelayListCountry>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    #[serde(rename = "openvpn")]
    pub openvpn: OpenVpnEndpointData,
    #[cfg_attr(target_os = "android", jnix(skip))]
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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayListCity {
    pub name: String,
    pub code: CityCode,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub latitude: f64,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub longitude: f64,
    pub relays: Vec<Relay>,
}

/// Stores information for a relay returned by the API at `v1/relays` using
/// `mullvad_api::RelayListProxy`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Relay {
    pub hostname: String,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub ipv4_addr_in: Ipv4Addr,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub ipv6_addr_in: Option<Ipv6Addr>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub include_in_country: bool,
    pub active: bool,
    pub owned: bool,
    pub provider: String,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub weight: u64,
    pub endpoint_data: RelayEndpointData,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub location: Option<Location>,
}

impl PartialEq for Relay {
    /// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its hostname.
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
    ///     # }),
    ///     # location: None,
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

/// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its hostname.
impl Eq for Relay {}

/// Hostnames are assumed to be unique per relay, i.e. a relay can be uniquely identified by its hostname.
impl std::hash::Hash for Relay {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hostname.hash(state)
    }
}

/// Specifies the type of a relay or relay-specific endpoint data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case")]
pub struct WireguardEndpointData {
    /// Port to connect to
    #[cfg_attr(
        target_os = "android",
        jnix(
            map = "|ranges| ranges.iter().map(|r| PortRange { from: r.0 as i32, to: r.1 as i32 } ).collect::<Vec<PortRange>>()"
        )
    )]
    pub port_ranges: Vec<(u16, u16)>,
    /// Gateways to be used with the tunnel
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub ipv4_gateway: Ipv4Addr,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub ipv6_gateway: Ipv6Addr,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub udp2tcp_ports: Vec<u16>,
}

impl Default for WireguardEndpointData {
    fn default() -> Self {
        Self {
            port_ranges: vec![],
            ipv4_gateway: "0.0.0.0".parse().unwrap(),
            ipv6_gateway: "::".parse().unwrap(),
            udp2tcp_ports: vec![],
        }
    }
}

/// Used for jni conversion
#[cfg(target_os = "android")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
struct PortRange {
    from: i32,
    to: i32,
}

/// Contains data about specific WireGuard endpoints, i.e. their public keys.
#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[cfg_attr(target_os = "android", jnix(skip_all))]
pub struct WireguardRelayEndpointData {
    /// Public key used by the relay peer
    pub public_key: wireguard::PublicKey,
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
    pub fn to_proxy_settings(&self, addr: IpAddr) -> CustomProxy {
        CustomProxy::Shadowsocks(Shadowsocks {
            endpoint: SocketAddr::new(addr, self.port),
            password: self.password.clone(),
            cipher: self.cipher.clone(),
        })
    }
}
