#[cfg(unix)]
use ipnetwork::Ipv4Network;
#[cfg(unix)]
use std::net::Ipv4Addr;
use std::path::Path;
use talpid_types::net::Endpoint;

#[cfg(unix)]
lazy_static! {
    static ref PRIVATE_NETS: [Ipv4Network; 3] = [
        Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap(),
        Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap(),
        Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap(),
    ];
    static ref MULTICAST_NET: Ipv4Network =
        Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 24).unwrap();
}

/// A enum that describes network security strategy
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SecurityPolicy {
    /// Allow traffic only to relay server
    Connecting {
        /// The relay endpoint that should be allowed.
        relay_endpoint: Endpoint,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },

    /// Allow traffic only to relay server and over tunnel interface
    Connected {
        /// The relay endpoint that should be allowed.
        relay_endpoint: Endpoint,
        /// Metadata about the tunnel and tunnel interface.
        tunnel: ::tunnel::TunnelMetadata,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },
}

/// Abstract firewall interaction trait
pub trait NetworkSecurity: Sized {
    /// The error type thrown by the implementer of this trait
    type Error: ::std::error::Error;

    /// Create new instance
    fn new(cache_dir: impl AsRef<Path>) -> ::std::result::Result<Self, Self::Error>;

    /// Enable the given SecurityPolicy
    fn apply_policy(&mut self, policy: SecurityPolicy) -> ::std::result::Result<(), Self::Error>;

    /// Revert the system network security state to what it was before this instance started
    /// modifying the system.
    fn reset_policy(&mut self) -> ::std::result::Result<(), Self::Error>;
}


#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::{Error, ErrorKind, MacosNetworkSecurity as NetworkSecurityImpl, Result};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{Error, ErrorKind, LinuxNetworkSecurity as NetworkSecurityImpl, Result};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{Error, ErrorKind, Result, WindowsNetworkSecurity as NetworkSecurityImpl};
