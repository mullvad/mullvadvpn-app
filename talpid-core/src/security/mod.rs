#[cfg(unix)]
use ipnetwork::Ipv4Network;
#[cfg(unix)]
use std::net::Ipv4Addr;
use std::path::Path;
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

pub use self::imp::{Error, ErrorKind};


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

    /// Block all network traffic in and out from the computer.
    Blocked {
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
    },
}

/// Manages network security of the computer/device. Can apply and enforce security policies
/// by manipulating the OS firewall and DNS settings.
pub struct NetworkSecurity {
    inner: imp::NetworkSecurity,
}

impl NetworkSecurity {
    /// Returns a new `NetworkSecurity`, ready to apply policies.
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(NetworkSecurity {
            inner: imp::NetworkSecurity::new(cache_dir)?,
        })
    }

    /// Applies and starts enforcing the given `SecurityPolicy` Makes sure it is being kept in place
    /// until this method is called again with another policy, or until `reset_policy` is called.
    pub fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<(), Error> {
        debug!("Setting security policy: {:?}", policy);
        self.inner.apply_policy(policy)
    }

    /// Resets/removes any currently enforced `SecurityPolicy`. Returns the system to the same state
    /// it had before any policy was applied through this `NetworkSecurity` instance.
    pub fn reset_policy(&mut self) -> Result<(), Error> {
        debug!("Resetting security policy");
        self.inner.reset_policy()
    }
}


/// Abstract firewall interaction trait. Used by the OS specific implementations.
trait NetworkSecurityT: Sized {
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
