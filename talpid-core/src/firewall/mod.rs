use std::path::Path;
use talpid_types::net::Endpoint;


/// A enum that describes firewall rules strategy
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
pub trait Firewall {
    /// The error type thrown by the implementer of this trait
    type Error: ::std::error::Error;

    /// Create new instance of Firewall
    fn new<P: AsRef<Path>>(cache_dir: P) -> ::std::result::Result<Self, Self::Error>
    where
        Self: Sized;

    /// Enable firewall and set firewall rules based on SecurityPolicy
    fn apply_policy(&mut self, policy: SecurityPolicy) -> ::std::result::Result<(), Self::Error>;

    /// Remove firewall rules applied by active SecurityPolicy and
    /// revert firewall to its original state
    fn reset_policy(&mut self) -> ::std::result::Result<(), Self::Error>;
}


#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::{Error, ErrorKind, PacketFilter as FirewallProxy, Result};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{Error, ErrorKind, Netfilter as FirewallProxy, Result};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{Error, ErrorKind, Result, WindowsFirewall as FirewallProxy};
