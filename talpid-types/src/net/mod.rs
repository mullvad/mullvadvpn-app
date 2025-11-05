use crate::net::obfuscation::ObfuscatorConfig;

#[cfg(target_os = "android")]
use jnix::FromJava;
use obfuscation::Obfuscators;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::path::PathBuf;
use std::{
    fmt, iter,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

pub mod obfuscation;
pub mod proxy;
pub mod wireguard;

mod allowed_nets;

pub use allowed_nets::*;

/// A tunnel endpoint is broadcast during the connecting and connected states of the tunnel state
/// machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TunnelEndpoint {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    pub quantum_resistant: bool,
    pub proxy: Option<proxy::ProxyEndpoint>,
    pub obfuscation: Option<ObfuscationInfo>,
    pub entry_endpoint: Option<Endpoint>,
    pub tunnel_interface: Option<String>,
    #[cfg(daita)]
    pub daita: bool,
}

impl fmt::Display for TunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "WireGuard ")?;
        if self.quantum_resistant {
            write!(f, "(quantum resistant) ")?;
        }
        write!(f, "- {}", self.endpoint)?;
        if let Some(ref entry_endpoint) = self.entry_endpoint {
            write!(f, " via {entry_endpoint}")?;
        }
        if let Some(ref obfuscation) = self.obfuscation {
            write!(f, " via {obfuscation}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "obfuscation_type")]
pub enum ObfuscationType {
    #[serde(rename = "udp2tcp")]
    Udp2Tcp,
    Shadowsocks,
    Quic,
    Lwo,
}

impl fmt::Display for ObfuscationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ObfuscationType::Udp2Tcp => "Udp2Tcp".fmt(f),
            ObfuscationType::Shadowsocks => "Shadowsocks".fmt(f),
            ObfuscationType::Quic => "QUIC".fmt(f),
            ObfuscationType::Lwo => "LWO".fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "obfuscation_info")]
pub enum ObfuscationInfo {
    /// Single obfuscator
    Single(ObfuscationEndpoint),
    /// Multiplexer obfuscator
    Multiplexer {
        /// Direct endpoint, without obfuscation, if set
        direct: Option<Endpoint>,
        /// All other obfuscators
        obfuscators: Vec<ObfuscationEndpoint>,
    },
}

impl ObfuscationInfo {
    pub fn get_endpoints(&self) -> Vec<Endpoint> {
        match self {
            ObfuscationInfo::Single(ep) => vec![ep.endpoint],
            ObfuscationInfo::Multiplexer {
                direct,
                obfuscators,
            } => {
                let mut v = vec![];
                if let Some(direct) = direct {
                    v.push(*direct);
                }
                obfuscators.iter().for_each(|obfs| v.push(obfs.endpoint));
                v
            }
        }
    }
}

impl fmt::Display for ObfuscationInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ObfuscationInfo::Single(obfs) => obfs.fmt(f),
            ObfuscationInfo::Multiplexer {
                direct,
                obfuscators,
            } => {
                write!(f, "multiplex ")?;

                write!(f, "{{ ")?;
                if let Some(direct) = direct {
                    write!(f, "direct {direct}")?;
                } else {
                    write!(f, "no direct")?;
                }
                for obfuscator in obfuscators {
                    write!(f, " | {obfuscator}")?;
                }
                write!(f, " }}")
            }
        }
    }
}

impl From<&Obfuscators> for ObfuscationInfo {
    fn from(config: &Obfuscators) -> Self {
        match config {
            Obfuscators::Multiplexer {
                direct,
                configs: (first_obfs, remaining_obfs),
            } => ObfuscationInfo::Multiplexer {
                direct: direct.map(|direct| Endpoint {
                    address: direct,
                    protocol: TransportProtocol::Udp,
                }),
                obfuscators: iter::once(first_obfs)
                    .chain(remaining_obfs)
                    .map(ObfuscationEndpoint::from)
                    .collect(),
            },
            Obfuscators::Single(obfs) => ObfuscationInfo::Single(ObfuscationEndpoint::from(obfs)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "obfuscation_endpoint")]
pub struct ObfuscationEndpoint {
    pub endpoint: Endpoint,
    pub obfuscation_type: ObfuscationType,
}

impl From<&ObfuscatorConfig> for ObfuscationEndpoint {
    fn from(config: &ObfuscatorConfig) -> ObfuscationEndpoint {
        let obfuscation_type = match config {
            ObfuscatorConfig::Udp2Tcp { .. } => ObfuscationType::Udp2Tcp,
            ObfuscatorConfig::Shadowsocks { .. } => ObfuscationType::Shadowsocks,
            ObfuscatorConfig::Quic { .. } => ObfuscationType::Quic,
            ObfuscatorConfig::Lwo { .. } => ObfuscationType::Lwo,
        };

        ObfuscationEndpoint {
            endpoint: config.endpoint(),
            obfuscation_type,
        }
    }
}

impl fmt::Display for ObfuscationEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} {}", self.obfuscation_type, self.endpoint)
    }
}

/// Represents a network layer IP address together with the transport layer protocol and port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Endpoint {
    /// The socket address for the endpoint
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

    pub const fn from_socket_address(address: SocketAddr, protocol: TransportProtocol) -> Self {
        Endpoint { address, protocol }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.address, self.protocol)
    }
}

/// Host that should be reachable in any tunnel state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AllowedEndpoint {
    /// How to connect to a certain `endpoint`.
    pub endpoint: Endpoint,
    /// Clients that should be allowed to communicate with `endpoint`.
    pub clients: AllowedClients,
}

impl fmt::Display for AllowedEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[cfg(not(windows))]
        write!(f, "{}", self.endpoint)?;
        #[cfg(windows)]
        {
            let clients = if self.clients.allow_all() {
                "any executable".to_string()
            } else {
                self.clients
                    .iter()
                    .map(|client| {
                        client
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or(std::borrow::Cow::Borrowed("<UNKNOWN>"))
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            write!(
                f,
                "{endpoint} for {clients}",
                endpoint = self.endpoint,
                clients = clients
            )?;
        }
        Ok(())
    }
}

/// Clients which should be able to reach an allowed host in any tunnel state.
///
/// # Note
/// On Windows, there is no predetermined binary which should be allowed to leak
/// traffic outside of the tunnel. Thus, [`std::default::Default`] is not
/// implemented for [`AllowedClients`].
#[cfg(windows)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AllowedClients(std::sync::Arc<[PathBuf]>);

#[cfg(windows)]
impl std::ops::Deref for AllowedClients {
    type Target = [PathBuf];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(windows)]
impl From<Vec<PathBuf>> for AllowedClients {
    fn from(value: Vec<PathBuf>) -> Self {
        Self(value.into())
    }
}

#[cfg(windows)]
impl AllowedClients {
    /// Allow all clients to leak traffic to an allowed [`Endpoint`].
    pub fn all() -> Self {
        vec![].into()
    }

    pub fn allow_all(&self) -> bool {
        self.is_empty()
    }
}

/// Clients which should be able to reach an allowed host in any tunnel state.
#[cfg(unix)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AllowedClients {
    /// Allow only clients running as `root` to leak traffic to an allowed [`Endpoint`].
    ///
    /// # Note
    /// The most secure client(s) is our own, which runs as root.
    Root,
    /// Allow *all* clients to leak traffic to an allowed [`Endpoint`].
    ///
    /// This is necessary on platforms which does not have proper support for
    /// split-tunneling, but which wants to support running local proxies which
    /// may not run as root.
    All,
}

#[cfg(unix)]
impl AllowedClients {
    pub fn allow_all(&self) -> bool {
        matches!(self, AllowedClients::All)
    }
}

/// What [`Endpoint`]s to allow the client to send traffic to and receive from.
///
/// In some cases we want to restrict what IP addresses the client may communicate with even
/// inside of the tunnel, for example while negotiating a PQ-safe PSK with an ephemeral peer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AllowedTunnelTraffic {
    /// Block all traffic inside the tunnel.
    None,
    /// Allow all traffic inside the tunnel. This is the normal mode of operation.
    All,
    /// Only allow communication with this specific endpoint. This will usually be a relay during a
    /// short amount of time.
    One(Endpoint),
    /// Only allow communication with these two specific endpoints. The intended use case for this
    /// is while negotiating for example a PSK with both the entry & exit relays in a multihop setup.
    Two(Endpoint, Endpoint),
}

impl AllowedTunnelTraffic {
    /// Do we currently allow traffic to all endpoints?
    pub fn all(&self) -> bool {
        matches!(self, AllowedTunnelTraffic::All)
    }
}

impl fmt::Display for AllowedTunnelTraffic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            AllowedTunnelTraffic::None => "None".fmt(f),
            AllowedTunnelTraffic::All => "All".fmt(f),
            AllowedTunnelTraffic::One(endpoint) => endpoint.fmt(f),
            AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
                endpoint1.fmt(f)?;
                f.write_str(", ")?;
                endpoint2.fmt(f)
            }
        }
    }
}

/// IP protocol version.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IpVersion {
    #[default]
    V4,
    V6,
}

impl From<IpAddr> for IpVersion {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(_) => IpVersion::V4,
            IpAddr::V6(_) => IpVersion::V6,
        }
    }
}

impl fmt::Display for IpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            IpVersion::V4 => "IPv4".fmt(f),
            IpVersion::V6 => "IPv6".fmt(f),
        }
    }
}

impl FromStr for IpVersion {
    type Err = IpVersionParseError;

    fn from_str(s: &str) -> Result<IpVersion, Self::Err> {
        match s {
            "v4" | "ipv4" => Ok(IpVersion::V4),
            "v6" | "ipv6" => Ok(IpVersion::V6),
            _ => Err(IpVersionParseError),
        }
    }
}

/// Returned when `IpVersion::from_str` fails to convert a string into a
/// [`IpVersion`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid IP protocol")]
pub struct IpVersionParseError;

/// Representation of a transport protocol, either UDP or TCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    /// Represents the UDP transport protocol.
    Udp,
    /// Represents the TCP transport protocol.
    Tcp,
}

impl FromStr for TransportProtocol {
    type Err = TransportProtocolParseError;

    fn from_str(s: &str) -> std::result::Result<TransportProtocol, Self::Err> {
        if s.eq_ignore_ascii_case("udp") {
            return Ok(TransportProtocol::Udp);
        }
        if s.eq_ignore_ascii_case("tcp") {
            return Ok(TransportProtocol::Tcp);
        }
        Err(TransportProtocolParseError)
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

/// Returned when `TransportProtocol::from_str` fails to convert a string into a
/// [`TransportProtocol`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid transport protocol")]
pub struct TransportProtocolParseError;

/// Holds optional settings that can apply to different kinds of tunnels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct GenericTunnelOptions {
    /// Enable configuration of IPv6 on the tunnel interface, allowing IPv6 communication to be
    /// forwarded through the tunnel.
    pub enable_ipv6: bool,
}

/// Details about the hosts's connectivity.
///
/// Information about the host's connectivity, such as the preesence of
/// configured IPv4 and/or IPv6.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.model"))]
pub enum Connectivity {
    /// Host is offline
    Offline,
    /// The connectivity status is unknown, but presumed to be online
    PresumeOnline,
    /// Host is online with the given IP versions available
    Online(IpAvailability),
}

impl Connectivity {
    /// Inverse of [`Connectivity::is_offline`].
    pub fn is_online(&self) -> bool {
        !self.is_offline()
    }

    /// If no IP4 nor IPv6 routes exist, we have no way of reaching the internet
    /// so we consider ourselves offline.
    pub fn is_offline(&self) -> bool {
        *self == Connectivity::Offline
    }

    /// Convert `self` to `IpAvailability`. Return `None` when `self` is `Connectivity::Offline`.
    ///
    /// If connectivity is unknown, return the default value of `IpAvailability` (IPv4).
    pub fn availability(&self) -> Option<IpAvailability> {
        match *self {
            Connectivity::Online(availability) => Some(availability),
            Connectivity::PresumeOnline => Some(IpAvailability::default()),
            Connectivity::Offline => None,
        }
    }

    /// Whether IPv4 connectivity seems to be available on the host.
    ///
    /// If IPv4 status is unknown, `true` is returned.
    pub fn has_ipv4(&self) -> bool {
        self.availability()
            .as_ref()
            .map(IpAvailability::has_ipv4)
            .unwrap_or(false)
    }

    /// Whether IPv6 connectivity seems to be available on the host.
    ///
    /// If IPv6 status is unknown, `false` is returned.
    pub fn has_ipv6(&self) -> bool {
        self.availability()
            .as_ref()
            .map(IpAvailability::has_ipv6)
            .unwrap_or(false)
    }

    /// Whether connectivity for `ip_version` seems to be available on the host.
    pub fn has_family(&self, ip_version: IpVersion) -> bool {
        match ip_version {
            IpVersion::V4 => self.has_ipv4(),
            IpVersion::V6 => self.has_ipv6(),
        }
    }

    pub fn new(ipv4: bool, ipv6: bool) -> Connectivity {
        match (ipv4, ipv6) {
            (true, true) => Connectivity::Online(IpAvailability::Ipv4AndIpv6),
            (true, false) => Connectivity::Online(IpAvailability::Ipv4),
            (false, true) => Connectivity::Online(IpAvailability::Ipv6),
            (false, false) => Connectivity::Offline,
        }
    }
}

impl fmt::Display for Connectivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Connectivity::Online(IpAvailability::Ipv4AndIpv6) => "Connected (IPv4 and IPv6)",
            Connectivity::Online(IpAvailability::Ipv4) => "Connected (IPv4)",
            Connectivity::Online(IpAvailability::Ipv6) => "Connected (IPv6)",
            Connectivity::PresumeOnline => "Online (assume IPv4)",
            Connectivity::Offline => "Offline",
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.model"))]
/// Available IP versions
pub enum IpAvailability {
    #[default]
    Ipv4,
    Ipv6,
    Ipv4AndIpv6,
}

impl IpAvailability {
    /// Whether IPv4 connectivity is available.
    pub fn has_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4 | Self::Ipv4AndIpv6)
    }

    /// Whether IPv6 connectivity is available.
    pub fn has_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6 | Self::Ipv4AndIpv6)
    }
}
