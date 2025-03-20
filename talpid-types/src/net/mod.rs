use self::proxy::{CustomProxy, Socks5Local};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(target_os = "android")]
use jnix::FromJava;
use obfuscation::ObfuscatorConfig;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::path::PathBuf;
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
    sync::LazyLock,
};

pub mod obfuscation;
pub mod openvpn;
pub mod proxy;
pub mod wireguard;

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub static ALLOWED_LAN_NETS: LazyLock<[IpNetwork; 6]> = LazyLock::new(|| {
    [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7).unwrap()),
    ]
});
/// When "allow local network" is enabled the app will allow traffic to these networks.
pub static ALLOWED_LAN_MULTICAST_NETS: LazyLock<[IpNetwork; 8]> = LazyLock::new(|| {
    [
        // Local network broadcast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(255, 255, 255, 255), 32).unwrap()),
        // Local subnetwork multicast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 24).unwrap()),
        // Admin-local IPv4 multicast.
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(239, 0, 0, 0), 8).unwrap()),
        // Interface-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Realm-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Admin-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Site-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
    ]
});

/// TunnelParameters are used to encapsulate all the data needed to start a tunnel. This is enum
/// should be generated by implementations of the trait
/// `talpid-core::tunnel_state_machine::TunnelParametersGenerator`
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TunnelParameters {
    OpenVpn(openvpn::TunnelParameters),
    Wireguard(wireguard::TunnelParameters),
}

impl TunnelParameters {
    pub fn get_tunnel_endpoint(&self) -> TunnelEndpoint {
        match self {
            TunnelParameters::OpenVpn(params) => TunnelEndpoint {
                tunnel_type: TunnelType::OpenVpn,
                quantum_resistant: false,
                endpoint: params.config.endpoint,
                proxy: params
                    .proxy
                    .as_ref()
                    .map(|proxy| proxy.get_remote_endpoint()),
                obfuscation: None,
                entry_endpoint: None,
                tunnel_interface: None,
                #[cfg(daita)]
                daita: false,
            },
            TunnelParameters::Wireguard(params) => TunnelEndpoint {
                tunnel_type: TunnelType::Wireguard,
                quantum_resistant: params.options.quantum_resistant,
                endpoint: params
                    .connection
                    .get_exit_endpoint()
                    .unwrap_or_else(|| params.connection.get_endpoint()),
                proxy: None,
                obfuscation: params.obfuscation.as_ref().map(ObfuscationEndpoint::from),
                entry_endpoint: params
                    .connection
                    .get_exit_endpoint()
                    .map(|_| params.connection.get_endpoint()),
                tunnel_interface: None,
                #[cfg(daita)]
                daita: params.options.daita,
            },
        }
    }

    /// Returns the endpoint that will be connected to
    pub fn get_next_hop_endpoint(&self) -> Endpoint {
        match self {
            TunnelParameters::OpenVpn(params) => params
                .proxy
                .as_ref()
                .map(|proxy| proxy.get_remote_endpoint().endpoint)
                .unwrap_or(params.config.endpoint),
            TunnelParameters::Wireguard(params) => params.get_next_hop_endpoint(),
        }
    }

    // Returns the exit endpoint, if it differs from the next hop endpoint
    pub fn get_exit_hop_endpoint(&self) -> Option<Endpoint> {
        match self {
            TunnelParameters::OpenVpn(_params) => None,
            TunnelParameters::Wireguard(params) => params.connection.get_exit_endpoint(),
        }
    }

    pub fn get_generic_options(&self) -> &GenericTunnelOptions {
        match &self {
            TunnelParameters::OpenVpn(params) => &params.generic_options,
            TunnelParameters::Wireguard(params) => &params.generic_options,
        }
    }

    pub fn get_openvpn_local_proxy_settings(&self) -> Option<&Socks5Local> {
        match &self {
            TunnelParameters::OpenVpn(params) => {
                params
                    .proxy
                    .as_ref()
                    .and_then(|proxy_settings| match proxy_settings {
                        CustomProxy::Socks5Local(local_settings) => Some(local_settings),
                        _ => None,
                    })
            }
            _ => None,
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

/// The tunnel protocol used by a [`TunnelEndpoint`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "tunnel_type")]
pub enum TunnelType {
    #[serde(rename = "openvpn")]
    OpenVpn,
    #[serde(rename = "wireguard")]
    #[default]
    Wireguard,
}

impl fmt::Display for TunnelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let tunnel = match self {
            TunnelType::OpenVpn => "OpenVPN",
            TunnelType::Wireguard => "WireGuard",
        };
        write!(f, "{tunnel}")
    }
}

impl FromStr for TunnelType {
    type Err = TunnelTypeParseError;

    fn from_str(s: &str) -> Result<TunnelType, Self::Err> {
        match s {
            "openvpn" => Ok(TunnelType::OpenVpn),
            "wireguard" => Ok(TunnelType::Wireguard),
            _ => Err(TunnelTypeParseError),
        }
    }
}

/// Returned when `TunnelType::from_str` fails to convert a string into a
/// [`TunnelType`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid tunnel protocol")]
pub struct TunnelTypeParseError;

/// A tunnel endpoint is broadcast during the connecting and connected states of the tunnel state
/// machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TunnelEndpoint {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    pub tunnel_type: TunnelType,
    pub quantum_resistant: bool,
    pub proxy: Option<proxy::ProxyEndpoint>,
    pub obfuscation: Option<ObfuscationEndpoint>,
    pub entry_endpoint: Option<Endpoint>,
    pub tunnel_interface: Option<String>,
    #[cfg(daita)]
    pub daita: bool,
}

impl fmt::Display for TunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} ", self.tunnel_type)?;
        if self.quantum_resistant {
            write!(f, "(quantum resistant) ")?;
        }
        write!(f, "- {}", self.endpoint)?;
        match self.tunnel_type {
            TunnelType::OpenVpn => {
                if let Some(ref proxy) = self.proxy {
                    write!(f, " via {} {}", proxy.proxy_type, proxy.endpoint)?;
                }
            }
            TunnelType::Wireguard => {
                if let Some(ref entry_endpoint) = self.entry_endpoint {
                    write!(f, " via {entry_endpoint}")?;
                }
                if let Some(ref obfuscation) = self.obfuscation {
                    write!(f, " via {obfuscation}")?;
                }
            }
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
}

impl fmt::Display for ObfuscationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ObfuscationType::Udp2Tcp => "Udp2Tcp".fmt(f),
            ObfuscationType::Shadowsocks => "Shadowsocks".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "obfuscation_endpoint")]
pub struct ObfuscationEndpoint {
    pub endpoint: Endpoint,
    pub obfuscation_type: ObfuscationType,
}

impl From<&ObfuscatorConfig> for ObfuscationEndpoint {
    fn from(config: &ObfuscatorConfig) -> ObfuscationEndpoint {
        let (endpoint, obfuscation_type) = match config {
            ObfuscatorConfig::Udp2Tcp { endpoint } => (
                Endpoint {
                    address: *endpoint,
                    protocol: TransportProtocol::Tcp,
                },
                ObfuscationType::Udp2Tcp,
            ),
            ObfuscatorConfig::Shadowsocks { endpoint } => (
                Endpoint {
                    address: *endpoint,
                    protocol: TransportProtocol::Udp,
                },
                ObfuscationType::Shadowsocks,
            ),
        };

        ObfuscationEndpoint {
            endpoint,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Returns a vector of IP networks representing all of the internet, 0.0.0.0/0.
/// This may be used in [`crate::net::wireguard::PeerConfig`] to route all traffic
/// to the tunnel interface.
pub fn all_of_the_internet() -> Vec<ipnetwork::IpNetwork> {
    vec![
        "0.0.0.0/0".parse().expect("Failed to parse ipv6 network"),
        "::0/0".parse().expect("Failed to parse ipv6 network"),
    ]
}

/// Details about the hosts's connectivity.
///
/// Information about the host's connectivity, such as the preesence of
/// configured IPv4 and/or IPv6.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.model"))]
pub enum Connectivity {
    Online {
        ip_availability: IpAvailability,
    },
    Offline,
    /// On/offline status could not be verified, but we have no particular
    /// reason to believe that the host is offline.
    PresumeOnline,
}

impl Connectivity {
    /// Inverse of [`Connectivity::is_offline`].
    pub fn is_online(&self) -> bool {
        !self.is_offline()
    }

    /// If no IP4 nor IPv6 routes exist, we have no way of reaching the internet
    /// so we consider ourselves offline.
    pub fn is_offline(&self) -> bool {
        matches!(self, Connectivity::Offline)
    }

    /// Whether IPv4 connectivity seems to be available on the host.
    ///
    /// If IPv4 status is unknown, `true` is returned.
    pub fn has_ipv4(&self) -> bool {
        match self {
            Connectivity::Offline => false,
            Connectivity::PresumeOnline => true,
            Connectivity::Online { ip_availability } => ip_availability.has_ipv4(),
        }
    }

    /// Whether IPv6 connectivity seems to be available on the host.
    ///
    /// If IPv6 status is unknown, `false` is returned.
    pub fn has_ipv6(&self) -> bool {
        match self {
            Connectivity::Offline | Connectivity::PresumeOnline => false,
            Connectivity::Online { ip_availability } => ip_availability.has_ipv6(),
        }
    }

    pub fn new(ipv4: bool, ipv6: bool) -> Connectivity {
        if ipv4 && ipv6 {
            Connectivity::Online {
                ip_availability: IpAvailability::All,
            }
        } else if ipv4 {
            Connectivity::Online {
                ip_availability: IpAvailability::Ipv4,
            }
        } else if ipv6 {
            Connectivity::Online {
                ip_availability: IpAvailability::Ipv6,
            }
        } else {
            Connectivity::Offline
        }
    }
}

#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.model"))]
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum IpAvailability {
    Ipv4,
    Ipv6,
    All,
}

impl IpAvailability {
    pub fn has_ipv4(&self) -> bool {
        self.clone() == IpAvailability::Ipv4 || self.clone() == IpAvailability::All
    }

    pub fn has_ipv6(&self) -> bool {
        self.clone() == IpAvailability::Ipv6 || self.clone() == IpAvailability::All
    }
}
