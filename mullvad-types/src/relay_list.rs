use crate::{
    endpoint::MullvadEndpoint,
    location::{CityCode, CountryCode, Location},
};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{
    openvpn::{ProxySettings, ShadowsocksProxySettings},
    wireguard, Endpoint, TransportProtocol,
};


/// Stores a list of relays for each country obtained from the API using
/// `mullvad_rpc::RelayListProxy`. This can also be passed to frontends.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayList {
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub etag: Option<String>,
    pub countries: Vec<RelayListCountry>,
}

impl RelayList {
    pub fn empty() -> Self {
        Self {
            etag: None,
            countries: Vec::new(),
        }
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
/// `mullvad_rpc::RelayListProxy`.
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
    #[serde(skip_serializing_if = "RelayTunnels::is_empty", default)]
    pub tunnels: RelayTunnels,
    #[serde(skip_serializing_if = "RelayBridges::is_empty", default)]
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub bridges: RelayBridges,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub location: Option<Location>,
}

/// Provides protocol-specific information about a [`Relay`].
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayTunnels {
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub openvpn: Vec<OpenVpnEndpointData>,
    pub wireguard: Vec<WireguardEndpointData>,
}

impl RelayTunnels {
    pub fn is_empty(&self) -> bool {
        self.openvpn.is_empty() && self.wireguard.is_empty()
    }

    pub fn clear(&mut self) {
        self.openvpn.clear();
        self.wireguard.clear();
    }
}

/// Data needed to connect to an OpenVPN endpoint at a [`Relay`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpointData {
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl OpenVpnEndpointData {
    pub fn into_mullvad_endpoint(self, host: IpAddr) -> MullvadEndpoint {
        MullvadEndpoint::OpenVpn(Endpoint::new(host, self.port, self.protocol))
    }
}

impl fmt::Display for OpenVpnEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} port {}", self.protocol, self.port)
    }
}

/// Data needed to connect to a WireGuard endpoint at a [`Relay`].
#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[cfg_attr(target_os = "android", jnix(skip_all))]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port_ranges: Vec<(u16, u16)>,
    /// Gateways to be used with the tunnel
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
    /// The peer's public key
    pub public_key: wireguard::PublicKey,
}

impl fmt::Display for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "gateways {} - {} port_ranges {{ {} }} public_key {}",
            self.ipv4_gateway,
            self.ipv6_gateway,
            self.port_ranges
                .iter()
                .map(|range| format!("[{} - {}]", range.0, range.1))
                .collect::<Vec<_>>()
                .join(","),
            self.public_key,
        )
    }
}

/// Used by `mullvad_rpc::RelayListProxy` to store bridge servers for a [`Relay`].
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayBridges {
    pub shadowsocks: Vec<ShadowsocksEndpointData>,
}

impl RelayBridges {
    pub fn is_empty(&self) -> bool {
        self.shadowsocks.is_empty()
    }

    pub fn clear(&mut self) {
        self.shadowsocks.clear();
    }
}

/// Data needed to connect to a Shadowsocks endpoint at a [`Relay`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ShadowsocksEndpointData {
    pub port: u16,
    pub cipher: String,
    pub password: String,
    pub protocol: TransportProtocol,
}

impl ShadowsocksEndpointData {
    pub fn to_proxy_settings(&self, addr: IpAddr) -> ProxySettings {
        ProxySettings::Shadowsocks(ShadowsocksProxySettings {
            peer: SocketAddr::new(addr, self.port),
            password: self.password.clone(),
            cipher: self.cipher.clone(),
        })
    }
}
