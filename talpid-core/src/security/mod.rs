#[cfg(unix)]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(unix)]
use lazy_static::lazy_static;
#[cfg(unix)]
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{fmt, net::IpAddr, path::Path};
use talpid_types::net::Endpoint;


#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

pub use self::imp::{DnsError, Error};


#[cfg(unix)]
lazy_static! {
    static ref PRIVATE_NETS: [IpNetwork; 3] = [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
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
pub enum SecurityPolicy {
    /// Allow traffic only to server
    Connecting {
        /// The peer endpoint that should be allowed.
        peer_endpoint: Endpoint,
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

impl fmt::Display for SecurityPolicy {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            SecurityPolicy::Connecting {
                peer_endpoint,
                allow_lan,
            } => write!(
                f,
                "Connecting to {}, {} LAN",
                peer_endpoint,
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
            SecurityPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
            } => write!(
                f,
                "Connected to {} over \"{}\" (ip: {}, gw: {}), {} LAN",
                peer_endpoint,
                tunnel.interface,
                tunnel
                    .ip
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                tunnel.gateway,
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
            SecurityPolicy::Blocked { allow_lan } => write!(
                f,
                "Blocked, {} LAN",
                if *allow_lan { "Allowing" } else { "Blocking" }
            ),
        }
    }
}

/// Manages network security of the computer/device. Can apply and enforce security policies
/// by manipulating the OS firewall and DNS settings.
pub struct NetworkSecurity {
    inner: imp::NetworkSecurity,
}

impl NetworkSecurity {
    /// Returns a new `NetworkSecurity`, ready to apply policies.
    pub fn new() -> Result<Self, Error> {
        Ok(NetworkSecurity {
            inner: imp::NetworkSecurity::new()?,
        })
    }

    /// Applies and starts enforcing the given `SecurityPolicy` Makes sure it is being kept in place
    /// until this method is called again with another policy, or until `reset_policy` is called.
    pub fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<(), Error> {
        log::info!("Applying security policy: {}", policy);
        self.inner.apply_policy(policy)
    }

    /// Resets/removes any currently enforced `SecurityPolicy`. Returns the system to the same state
    /// it had before any policy was applied through this `NetworkSecurity` instance.
    pub fn reset_policy(&mut self) -> Result<(), Error> {
        log::info!("Resetting security policy");
        self.inner.reset_policy()
    }
}

/// Sets and monitors system DNS settings. Makes sure the desired DNS servers are being used.
pub struct DnsMonitor {
    inner: imp::DnsMonitor,
}

impl DnsMonitor {
    /// Returns a new `DnsMonitor` that can set and monitor the system DNS.
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self, DnsError> {
        Ok(DnsMonitor {
            inner: imp::DnsMonitor::new(cache_dir)?,
        })
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), DnsError> {
        log::info!(
            "Setting DNS servers to {}",
            servers
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        self.inner.set(interface, servers)
    }

    /// Reset system DNS settings to what it was before being set by this instance.
    pub fn reset(&mut self) -> Result<(), DnsError> {
        log::info!("Resetting DNS");
        self.inner.reset()
    }
}


/// Abstract firewall interaction trait. Used by the OS specific implementations.
trait NetworkSecurityT: Sized {
    /// The error type thrown by the implementer of this trait
    type Error: ::std::error::Error;

    /// Create new instance
    fn new() -> ::std::result::Result<Self, Self::Error>;

    /// Enable the given SecurityPolicy
    fn apply_policy(&mut self, policy: SecurityPolicy) -> ::std::result::Result<(), Self::Error>;

    /// Revert the system network security state to what it was before this instance started
    /// modifying the system.
    fn reset_policy(&mut self) -> ::std::result::Result<(), Self::Error>;
}

trait DnsMonitorT: Sized {
    type Error: ::std::error::Error;

    fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Self::Error>;

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;
}
