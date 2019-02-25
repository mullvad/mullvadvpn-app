#[cfg(unix)]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(unix)]
use lazy_static::lazy_static;
use std::fmt;
#[cfg(windows)]
use std::net::IpAddr;
#[cfg(unix)]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use talpid_types::net::Endpoint;


#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

pub use self::imp::Error;


#[cfg(unix)]
lazy_static! {
    static ref PRIVATE_NETS: [IpNetwork; 4] = [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()),
    ];
    static ref LOCAL_INET6_NET: IpNetwork =
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap());
    static ref MULTICAST_NET: IpNetwork =
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 24).unwrap());
    static ref MULTICAST_INET6_NET: IpNetwork =
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe02, 0, 0, 0, 0, 0, 0, 0), 16).unwrap());
    static ref SSDP_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(239, 255, 255, 250));
    static ref DHCPV6_SERVER_ADDRS: [IpAddr; 2] = [
        IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2)),
        IpAddr::V6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 1, 3)),
    ];
}

/// A enum that describes network security strategy
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FirewallPolicy {
    /// Allow traffic only to server
    Connecting {
        /// The peer endpoint that should be allowed.
        peer_endpoint: Endpoint,
        /// Hosts that should be pingable whilst connecting.
        pingable_hosts: Vec<IpAddr>,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },

    /// Allow traffic only to server and over tunnel interface
    Connected {
        /// The peer endpoint that should be allowed.
        peer_endpoint: Endpoint,
        /// Metadata about the tunnel and tunnel interface.
        tunnel: crate::tunnel::TunnelMetadata,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },

    /// Block all network traffic in and out from the computer.
    Blocked {
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },
}

impl fmt::Display for FirewallPolicy {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            FirewallPolicy::Connecting {
                peer_endpoint,
                pingable_hosts,
                allow_lan,
            } => write!(
                f,
                "Connecting to {} with gateways {}, {} LAN",
                peer_endpoint,
                pingable_hosts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(","),
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
            } => write!(
                f,
                "Connected to {} over \"{}\" (ip: {}, v4 gw: {}, v6 gw; {:?}), {} LAN",
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
            FirewallPolicy::Blocked { allow_lan } => write!(
                f,
                "Blocked, {} LAN",
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
        }
    }
}

/// Manages network security of the computer/device. Can apply and enforce firewall policies
/// by manipulating the OS firewall and DNS settings.
pub struct Firewall {
    inner: imp::Firewall,
}

impl Firewall {
    /// Returns a new `Firewall`, ready to apply policies.
    pub fn new() -> Result<Self, Error> {
        Ok(Firewall {
            inner: imp::Firewall::new()?,
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

/// Abstract firewall interaction trait. Used by the OS specific implementations.
trait FirewallT: Sized {
    /// The error type thrown by the implementer of this trait
    type Error: ::std::error::Error;

    /// Create new instance
    fn new() -> ::std::result::Result<Self, Self::Error>;

    /// Enable the given FirewallPolicy
    fn apply_policy(&mut self, policy: FirewallPolicy) -> ::std::result::Result<(), Self::Error>;

    /// Revert the system firewall state to what it was before this instance started
    /// modifying the system.
    fn reset_policy(&mut self) -> ::std::result::Result<(), Self::Error>;
}
