use crate::location::{CityCode, CountryCode, Location};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use talpid_types::net::{
    openvpn::{ProxySettings, ShadowsocksProxySettings},
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
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub owned: bool,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub provider: String,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub weight: u64,
    pub endpoint_data: RelayEndpointData,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub location: Option<Location>,
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

impl RelayEndpointData {
    pub fn unwrap_wireguard_ref(&self) -> &WireguardRelayEndpointData {
        if let RelayEndpointData::Wireguard(wg) = &self {
            return wg;
        }
        panic!("not a wireguard endpoint");
    }
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
    pub fn to_proxy_settings(
        &self,
        addr: IpAddr,
        #[cfg(target_os = "linux")] fwmark: u32,
    ) -> ProxySettings {
        ProxySettings::Shadowsocks(ShadowsocksProxySettings {
            peer: SocketAddr::new(addr, self.port),
            password: self.password.clone(),
            cipher: self.cipher.clone(),
            #[cfg(target_os = "linux")]
            fwmark: Some(fwmark),
        })
    }
}
