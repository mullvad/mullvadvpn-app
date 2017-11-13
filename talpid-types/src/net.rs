use std::error::Error;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

/// Represents one tunnel endpoint. Tunnel technology plus address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TunnelEndpoint {
    /// An OpenVPN tunnel endpoint.
    OpenVpn(Endpoint),
    /// A Wireguard tunnel endpoint.
    Wireguard(SocketAddr),
}

impl TunnelEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        match *self {
            TunnelEndpoint::OpenVpn(endpoint) => endpoint,
            TunnelEndpoint::Wireguard(address) => Endpoint {
                address,
                protocol: TransportProtocol::Udp,
            },
        }
    }
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
    pub fn new<T: Into<IpAddr>>(address: T, port: u16, protocol: TransportProtocol) -> Self {
        Endpoint {
            address: SocketAddr::new(address.into(), port),
            protocol: protocol,
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
