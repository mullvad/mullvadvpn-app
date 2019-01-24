use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

pub mod openvpn;
pub mod wireguard;


#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum TunnelParameters {
    OpenVpn(openvpn::TunnelParameters),
    Wireguard(wireguard::TunnelParameters),
}


impl TunnelParameters {
    pub fn host(&self) -> SocketAddr {
        match self {
            TunnelParameters::OpenVpn(params) => params.config.host,
            TunnelParameters::Wireguard(params) => params.connection.peer.endpoint,
        }
    }

    pub fn get_endpoint(&self) -> Endpoint {
        Endpoint {
            address: self.host(),
            protocol: match &self {
                TunnelParameters::OpenVpn(params) => params.config.protocol,
                TunnelParameters::Wireguard(_params) => TransportProtocol::Udp,
            },
        }
    }

    pub fn get_generic_options(&self) -> &GenericTunnelOptions {
        match &self {
            TunnelParameters::OpenVpn(params) => &params.generic_options,
            TunnelParameters::Wireguard(params) => &params.generic_options,
        }
    }
}

impl From<wireguard::TunnelParameters> for TunnelParameters {
    fn from(wg_params: wireguard::TunnelParameters) -> TunnelParameters {
        TunnelParameters::Wireguard(wg_params)
    }
}

impl From<openvpn::TunnelParameters> for TunnelParameters {
    fn from(params: openvpn::TunnelParameters) -> TunnelParameters {
        TunnelParameters::OpenVpn(params)
    }
}


/// Represents a network layer IP address together with the transport layer protocol and port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.address, self.protocol)
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
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TransportProtocol::Udp => "UDP".fmt(fmt),
            TransportProtocol::Tcp => "TCP".fmt(fmt),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportProtocolParseError;

impl fmt::Display for TransportProtocolParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TunnelOptions {
    /// openvpn holds OpenVPN specific tunnel options.
    pub openvpn: openvpn::TunnelOptions,
    /// Contains wireguard tunnel options.
    pub wireguard: wireguard::TunnelOptions,
    /// Contains generic tunnel options that may apply to more than a single tunnel type.
    pub generic: GenericTunnelOptions,
}

impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            openvpn: openvpn::TunnelOptions::default(),
            wireguard: wireguard::TunnelOptions::default(),
            generic: GenericTunnelOptions::default(),
        }
    }
}


/// Holds optional settings that can apply to different kinds of tunnels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct GenericTunnelOptions {
    /// Enable configuration of IPv6 on the tunnel interface, allowing IPv6 communication to be
    /// forwarded through the tunnel. By default, this is set to `true`.
    pub enable_ipv6: bool,
}

impl Default for GenericTunnelOptions {
    fn default() -> Self {
        Self { enable_ipv6: false }
    }
}
