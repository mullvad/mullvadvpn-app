use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

/// Represents one tunnel endpoint. Address, plus extra parameters specific to tunnel protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TunnelEndpointData {
    /// Extra parameters for an OpenVPN tunnel endpoint.
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnEndpointData),
    /// Extra parameters for a Wireguard tunnel endpoint.
    #[serde(rename = "wireguard")]
    Wireguard(WireguardEndpointData),
}

impl fmt::Display for TunnelEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TunnelEndpointData::OpenVpn(openvpn_data) => {
                write!(f, "OpenVPN ")?;
                openvpn_data.fmt(f)
            }
            TunnelEndpointData::Wireguard(wireguard_data) => {
                write!(f, "Wireguard ")?;
                wireguard_data.fmt(f)
            }
        }
    }
}

impl TunnelEndpointData {
    pub fn port(&self) -> u16 {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => metadata.port,
            TunnelEndpointData::Wireguard(metadata) => metadata.port,
        }
    }

    pub fn transport_protocol(&self) -> TransportProtocol {
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

impl fmt::Display for OpenVpnEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} port {}", self.protocol, self.port)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port: u16,
    /// Link addresses
    pub addresses: Vec<IpAddr>,
    /// Peer's IP address
    pub gateway: IpAddr,
    #[serde(skip)]
    /// Keys required to connect to the tunnel
    pub keys: Option<WireguardKeys>,
}

impl WireguardEndpointData {
    /// Set keys for a given wireguard config.
    pub fn apply_keys(&mut self, keys: WireguardKeys) {
        self.keys = Some(keys);
    }
}

/// Keys for a wireguard connection between the client and a single host
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct WireguardKeys {
    /// Private key of client
    pub private_key: WgPrivateKey,
    /// Public key of peer
    pub public_key: WgPublicKey,
}

impl fmt::Debug for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("WireguardEndpointData")
            .field("gateway", &self.gateway.to_string())
            .field("port", &self.port.to_string())
            .field("keys", &self.keys)
            .field(
                "addresses",
                &self
                    .addresses
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .finish()
    }
}

impl fmt::Display for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "port {}", self.port)
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
    pub openvpn: OpenVpnTunnelOptions,
    /// Contains wireguard tunnel options.
    pub wireguard: WireguardTunnelOptions,
    /// Enable configuration of IPv6 on the tunnel interface, allowing IPv6 communication to be
    /// forwarded through the tunnel. By default, this is set to `true`.
    pub enable_ipv6: bool,
}

impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            openvpn: OpenVpnTunnelOptions::default(),
            wireguard: WireguardTunnelOptions::default(),
            enable_ipv6: false,
        }
    }
}


/// OpenVpnTunnelOptions contains options for an openvpn tunnel that should be applied irrespective
/// of the relay parameters - i.e. have nothing to do with the particular OpenVPN server, but do
/// affect the connection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OpenVpnTunnelOptions {
    /// Optional argument for openvpn to try and limit TCP packet size,
    /// as discussed [here](https://openvpn.net/archive/openvpn-users/2003-11/msg00154.html)
    pub mssfix: Option<u16>,
    /// Proxy settings, for when the relay connection should be via a proxy.
    pub proxy: Option<OpenVpnProxySettings>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenVpnProxySettings {
    Local(LocalOpenVpnProxySettings),
    Remote(RemoteOpenVpnProxySettings),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalOpenVpnProxySettings {
    pub port: u16,
    pub peer: SocketAddr,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RemoteOpenVpnProxySettings {
    pub address: SocketAddr,
    pub auth: Option<OpenVpnProxyAuth>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnProxyAuth {
    pub username: String,
    pub password: String,
}

pub struct OpenVpnProxySettingsValidation;

impl OpenVpnProxySettingsValidation {
    pub fn validate(proxy: &OpenVpnProxySettings) -> Result<(), String> {
        match proxy {
            OpenVpnProxySettings::Local(local) => {
                if local.port == 0 {
                    return Err(String::from("Invalid local port number"));
                }
                if local.peer.ip().is_loopback() {
                    return Err(String::from(
                        "localhost is not a valid peer in this context",
                    ));
                }
                if local.peer.port() == 0 {
                    return Err(String::from("Invalid remote port number"));
                }
            }
            OpenVpnProxySettings::Remote(remote) => {
                if remote.address.port() == 0 {
                    return Err(String::from("Invalid port number"));
                }
                if remote.address.ip().is_loopback() {
                    return Err(String::from("localhost is not a valid remote server"));
                }
            }
        };
        Ok(())
    }
}

/// Wireguard tunnel options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WireguardTunnelOptions {
    /// MTU for the wireguard tunnel
    pub mtu: Option<u16>,
    /// firewall mark
    pub fwmark: Option<i32>,
}


impl Default for WireguardTunnelOptions {
    fn default() -> Self {
        Self {
            mtu: None,
            fwmark: None,
        }
    }
}

/// Wireguard x25519 private key
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WgPrivateKey {
    private_key: [u8; 32],
}

impl fmt::Debug for WgPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("WgPrivateKey")
            .field("private_key", &"[bytes]")
            .finish()
    }
}

impl fmt::Display for WgPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", "[WgPrivateKey]")
    }
}

impl WgPrivateKey {
    pub fn data(&self) -> &[u8; 32] {
        &self.private_key
    }
}

/// Wireguard x25519 public key
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct WgPublicKey {
    public_key: [u8; 32],
}

impl WgPublicKey {
    pub fn data(&self) -> &[u8; 32] {
        &self.public_key
    }
}


impl fmt::Debug for WgPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("WgPublicKey")
            .field("public_key", &base64::encode(&self.public_key))
            .finish()
    }
}

impl fmt::Display for WgPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", &base64::encode(&self.public_key))
    }
}
