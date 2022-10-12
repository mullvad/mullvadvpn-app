use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use lazy_static::lazy_static;
#[cfg(not(target_os = "android"))]
use std::net::IpAddr;
#[cfg(windows)]
use std::path::PathBuf;
use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{AllowedEndpoint, AllowedTunnelTraffic, Endpoint};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use self::imp::Error;

lazy_static! {
    /// When "allow local network" is enabled the app will allow traffic to and from these networks.
    pub(crate) static ref ALLOWED_LAN_NETS: [IpNetwork; 6] = [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7).unwrap()),
    ];
    /// When "allow local network" is enabled the app will allow traffic to these networks.
    pub(crate) static ref ALLOWED_LAN_MULTICAST_NETS: [IpNetwork; 8] = [
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
    ];
    static ref IPV6_LINK_LOCAL: Ipv6Network = Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap();
    /// The allowed target addresses of outbound DHCPv6 requests
    static ref DHCPV6_SERVER_ADDRS: [Ipv6Addr; 2] = [
        Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2),
        Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 1, 3),
    ];
    static ref ROUTER_SOLICITATION_OUT_DST_ADDR: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 2);
    static ref SOLICITED_NODE_MULTICAST: Ipv6Network = Ipv6Network::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 1, 0xFF00, 0), 104).unwrap();
    static ref LOOPBACK_NETS: [IpNetwork; 2] = [
        IpNetwork::V4(ipnetwork::Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap()),
        IpNetwork::V6(ipnetwork::Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap()),
    ];
}
#[cfg(all(unix, not(target_os = "android")))]
const DHCPV4_SERVER_PORT: u16 = 67;
#[cfg(all(unix, not(target_os = "android")))]
const DHCPV4_CLIENT_PORT: u16 = 68;
#[cfg(all(unix, not(target_os = "android")))]
const DHCPV6_SERVER_PORT: u16 = 547;
#[cfg(all(unix, not(target_os = "android")))]
const DHCPV6_CLIENT_PORT: u16 = 546;
#[cfg(all(unix, not(target_os = "android")))]
const ROOT_UID: u32 = 0;

#[cfg(any(all(unix, not(target_os = "android")), target_os = "windows"))]
/// Returns whether an address belongs to a private subnet.
pub fn is_local_address(address: &IpAddr) -> bool {
    let address = *address;
    (*ALLOWED_LAN_NETS)
        .iter()
        .chain(&*LOOPBACK_NETS)
        .any(|net| net.contains(address))
}

/// A enum that describes network security strategy
///
/// # Firewall block/allow specification.
///
/// See the [security](../../../docs/security.md) document for the specification on how to
/// implement these policies and what should and should not be allowed to flow.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FirewallPolicy {
    /// Allow traffic only to server
    Connecting {
        /// The peer endpoint that should be allowed.
        peer_endpoint: Endpoint,
        /// Metadata about the tunnel and tunnel interface.
        tunnel: Option<crate::tunnel::TunnelMetadata>,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Host that should be reachable while connecting.
        allowed_endpoint: AllowedEndpoint,
        /// Networks for which to permit in-tunnel traffic.
        allowed_tunnel_traffic: AllowedTunnelTraffic,
        /// A process that is allowed to send packets to the relay.
        #[cfg(windows)]
        relay_client: PathBuf,
    },

    /// Allow traffic only to server and over tunnel interface
    Connected {
        /// The peer endpoint that should be allowed.
        peer_endpoint: Endpoint,
        /// Metadata about the tunnel and tunnel interface.
        tunnel: crate::tunnel::TunnelMetadata,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Servers that are allowed to respond to DNS requests.
        #[cfg(not(target_os = "android"))]
        dns_servers: Vec<IpAddr>,
        /// A process that is allowed to send packets to the relay.
        #[cfg(windows)]
        relay_client: PathBuf,
    },

    /// Block all network traffic in and out from the computer.
    Blocked {
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Host that should be reachable while in the blocked state.
        allowed_endpoint: Option<AllowedEndpoint>,
        /// Desination port for DNS traffic redirection. Traffic destined to `127.0.0.1:53` will be
        /// redirected to `127.0.0.1:$dns_redirect_port`.
        #[cfg(target_os = "macos")]
        dns_redirect_port: u16,
    },
}

impl fmt::Display for FirewallPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
                ..
            } => {
                if let Some(tunnel) = tunnel {
                    write!(
                        f,
                        "Connecting to {} over \"{}\" (ip: {}, v4 gw: {}, v6 gw: {:?}, allowed in-tunnel traffic: {}), {} LAN. Allowing endpoint {}",
                        peer_endpoint,
                        tunnel.interface,
                        tunnel
                            .ips
                            .iter()
                            .map(|ip| ip.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                        tunnel.ipv4_gateway,
                        tunnel.ipv6_gateway,
                        allowed_tunnel_traffic,
                        if *allow_lan { "Allowing" } else { "Blocking" },
                        allowed_endpoint,
                    )
                } else {
                    write!(
                        f,
                        "Connecting to {}, {} LAN, interface: none. Allowing endpoint {}",
                        peer_endpoint,
                        if *allow_lan { "Allowing" } else { "Blocking" },
                        allowed_endpoint,
                    )
                }
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
                ..
            } => write!(
                f,
                "Connected to {} over \"{}\" (ip: {}, v4 gw: {}, v6 gw: {:?}), {} LAN",
                peer_endpoint,
                tunnel.interface,
                tunnel
                    .ips
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                tunnel.ipv4_gateway,
                tunnel.ipv6_gateway,
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
                ..
            } => write!(
                f,
                "Blocked. {} LAN. Allowing endpoint: {}",
                if *allow_lan { "Allowing" } else { "Blocking" },
                allowed_endpoint
                    .as_ref()
                    .map(|endpoint| -> &dyn std::fmt::Display { endpoint })
                    .unwrap_or(&"none"),
            ),
        }
    }
}

/// Manages network security of the computer/device. Can apply and enforce firewall policies
/// by manipulating the OS firewall and DNS settings.
pub struct Firewall {
    inner: imp::Firewall,
}

/// Arguments required when first initializing the firewall.
pub struct FirewallArguments {
    /// Initial firewall state to enter during init.
    pub initial_state: InitialFirewallState,
    /// This argument is required for the blocked state to configure the firewall correctly.
    pub allow_lan: bool,
    /// Specifies the firewall mark used to identify traffic that is allowed to be excluded from
    /// the tunnel and _leaked_ during blocked states.
    #[cfg(target_os = "linux")]
    pub fwmark: u32,
}

/// State to enter during firewall init.
pub enum InitialFirewallState {
    /// Do not set any policy.
    None,
    /// Atomically enter the blocked state.
    Blocked(AllowedEndpoint),
}

impl Firewall {
    /// Creates a firewall instance with the given arguments.
    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        Ok(Firewall {
            inner: imp::Firewall::from_args(args)?,
        })
    }

    /// Createsa new firewall instance.
    pub fn new(#[cfg(target_os = "linux")] fwmark: u32) -> Result<Self, Error> {
        Ok(Firewall {
            inner: imp::Firewall::new(
                #[cfg(target_os = "linux")]
                fwmark,
            )?,
        })
    }

    /// Applies and starts enforcing the given `FirewallPolicy` Makes sure it is being kept in place
    /// until this method is called again with another policy, or until `reset_policy` is called.
    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        log::info!("Applying firewall policy: {}", policy);
        self.inner.apply_policy(policy)
    }

    /// Resets/removes any currently enforced `FirewallPolicy`. Returns the system to the same state
    /// it had before any policy was applied through this `Firewall` instance.
    pub fn reset_policy(&mut self) -> Result<(), Error> {
        log::info!("Resetting firewall policy");
        self.inner.reset_policy()
    }
}
