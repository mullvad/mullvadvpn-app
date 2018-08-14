use std::error::Error;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

/// Represents one tunnel endpoint. Address, plus extra parameters specific to tunnel protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TunnelEndpoint {
    pub address: IpAddr,
    pub tunnel: TunnelEndpointData,
}

impl TunnelEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        Endpoint::new(
            self.address,
            self.tunnel.port(),
            self.tunnel.transport_protocol(),
        )
    }
}

/// TunnelEndpointData contains data required to connect to a given tunnel endpoint.
/// Different endpoint types can require different types of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TunnelEndpointData {
    /// Extra parameters for an OpenVPN tunnel endpoint.
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnEndpointData),
    /// Extra parameters for a Wireguard tunnel endpoint.
    #[serde(rename = "wireguard")]
    Wireguard(WireguardEndpointData),
}

impl TunnelEndpointData {
    pub fn port(self) -> u16 {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => metadata.port,
            TunnelEndpointData::Wireguard(metadata) => metadata.port,
        }
    }

    pub fn transport_protocol(self) -> TransportProtocol {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => metadata.protocol,
            TunnelEndpointData::Wireguard(_) => TransportProtocol::Udp,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpointData {
    pub port: u16,
    pub protocol: TransportProtocol,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct WireguardEndpointData {
    pub port: u16,
}


/// Represents a network layer IP address together with the transport layer protocol and port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    /// The address part of this endpoint, contains the IP and port.
    pub address: SocketAddr,
    /// The protocol part of this endpoint.
    pub protocol: TransportProtocol,
}

impl Endpoint {
    /// Constructs a new `Endpoint` from the given parameters.
    pub fn new(address: impl Into<IpAddr>, port: u16, protocol: TransportProtocol) -> Self {
        Endpoint {
            address: SocketAddr::new(address.into(), port),
            protocol,
        }
    }
}

/// Representation of a transport protocol, either UDP or TCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    /// Represents the UDP transport protocol.
    Udp,
    /// Represents the TCP transport protocol.
    Tcp,
}

impl FromStr for TransportProtocol {
    type Err = TransportProtocolParseError;

    fn from_str(s: &str) -> ::std::result::Result<TransportProtocol, Self::Err> {
        match s {
            "udp" => Ok(TransportProtocol::Udp),
            "tcp" => Ok(TransportProtocol::Tcp),
            _ => Err(TransportProtocolParseError),
        }
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransportProtocol::Udp => "UDP".fmt(fmt),
            TransportProtocol::Tcp => "TCP".fmt(fmt),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportProtocolParseError;

impl fmt::Display for TransportProtocolParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for TransportProtocolParseError {
    fn description(&self) -> &str {
        "Not a valid transport protocol"
    }
}

/// TunnelOptions holds optional settings for tunnels, that are to be applied to any tunnel of the
/// appropriate type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TunnelOptions {
    /// openvpn holds OpenVPN specific tunnel options.
    pub openvpn: OpenVpnTunnelOptions,
}


/// OpenVpnTunnelOptions contains options for an openvpn tunnel that should be applied irrespective
/// of the relay parameters - i.e. have nothing to do with the particular OpenVPN server, but do
/// affect the connection.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenVpnTunnelOptions {
    /// Optional argument for openvpn to try and limit TCP packet size,
    /// as discussed [here](https://openvpn.net/archive/openvpn-users/2003-11/msg00154.html)
    pub mssfix: Option<u16>,
    /// Enable configuration of IPv6 on the tunnel interface, allowing IPv6 communication to be
    /// forwarded through the tunnel. By default, this is set to `true`.
    pub enable_ipv6: bool,
}

impl Default for OpenVpnTunnelOptions {
    fn default() -> Self {
        OpenVpnTunnelOptions {
            mssfix: None,
            enable_ipv6: true,
        }
    }
}
