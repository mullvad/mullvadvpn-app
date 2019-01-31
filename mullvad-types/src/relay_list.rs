use crate::{
    endpoint::MullvadEndpoint,
    location::{CityCode, CountryCode, Location},
    relay_constraints::{Constraint, WireguardConstraints},
};
use ipnetwork::IpNetwork;
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{wireguard, Endpoint, TransportProtocol};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub countries: Vec<RelayListCountry>,
}

impl RelayList {
    pub fn empty() -> Self {
        Self {
            countries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCountry {
    pub name: String,
    pub code: CountryCode,
    pub cities: Vec<RelayListCity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCity {
    pub name: String,
    pub code: CityCode,
    pub latitude: f64,
    pub longitude: f64,
    pub relays: Vec<Relay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub include_in_country: bool,
    pub weight: u64,
    #[serde(skip_serializing_if = "RelayTunnels::is_empty", default)]
    pub tunnels: RelayTunnels,
    #[serde(skip)]
    pub location: Option<Location>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayTunnels {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpointData {
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl OpenVpnEndpointData {
    pub fn to_mullvad_endpoint(self, host: IpAddr) -> MullvadEndpoint {
        MullvadEndpoint::OpenVpn(Endpoint::new(host, self.port, self.protocol))
    }
}

impl fmt::Display for OpenVpnEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} port {}", self.protocol, self.port)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port_ranges: Vec<[u16; 2]>,
    /// Gateways to be used with the tunnel
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
    /// The peer's public key
    pub public_key: wireguard::PublicKey,
}

impl WireguardEndpointData {
    pub fn get_port(&self, constraints: &WireguardConstraints, rng: &mut impl Rng) -> Option<u16> {
        match constraints.port {
            Constraint::Any => {
                let range = rng.choose(&self.port_ranges)?;

                // since our upper port range is inclusive and rng.gen() panics if min >= max,
                // then we're padding
                let port = rng.gen_range(range[0], range[1] + 1);
                Some(port)
            }
            Constraint::Only(port) => {
                if self
                    .port_ranges
                    .iter()
                    .any(|range| (range[0] < port && port <= range[1]))
                {
                    Some(port)
                } else {
                    None
                }
            }
        }
    }

    pub fn to_mullvad_endpoint(
        self,
        host: IpAddr,
        constraints: &crate::relay_constraints::WireguardConstraints,
        rng: &mut impl Rng,
    ) -> Option<crate::endpoint::MullvadEndpoint> {
        let port = self.get_port(constraints, rng)?;
        let peer_config = wireguard::PeerConfig {
            public_key: self.public_key,
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
        };
        Some(crate::endpoint::MullvadEndpoint::Wireguard {
            peer: peer_config,
            gateway: self.ipv4_gateway.into(),
        })
    }
}

fn all_of_the_internet() -> Vec<IpNetwork> {
    vec![
        "0.0.0.0/0".parse().expect("Failed to parse ipv6 network"),
        "::0/0".parse().expect("Failed to parse ipv6 network"),
    ]
}

impl fmt::Debug for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct(&"WireguardEndpointData")
            .field("port_ranges", &self.port_ranges)
            .field("ipv4_gateway", &self.ipv4_gateway)
            .field("ipv6_gateway", &self.ipv6_gateway)
            .field("public_key", &self.public_key)
            .finish()
    }
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
                .map(|range| format!("[{} - {}]", range[0], range[1]))
                .collect::<Vec<_>>()
                .join(","),
            self.public_key,
        )
    }
}
