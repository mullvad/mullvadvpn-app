use net::Endpoint;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(all(unix, not(target_os = "macos")))]
#[path = "unix.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

error_chain!{
    errors {
        /// Initialization error
        FirewallInitError {
            description("Failed to initialize firewall")
        }
        /// Firewall configuration error
        FirewallConfigurationError {
            description("Failed to configure firewall")
        }
    }
}

/// A enum that describes firewall rules strategy
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SecurityPolicy {
    /// Allow traffic only to relay server
    Connecting(Endpoint),

    /// Allow traffic only to relay server and over tunnel interface
    Connected(Endpoint, String),
}

/// Abstract firewall interaction trait
pub trait Firewall<E: ::std::error::Error> {
    /// Create new instance of Firewall
    fn new() -> ::std::result::Result<Self, E> where Self: Sized;

    /// Enable firewall and set firewall rules based on SecurityPolicy
    fn apply_policy(&mut self, policy: SecurityPolicy) -> ::std::result::Result<(), E>;

    /// Remove firewall rules applied by active SecurityPolicy and
    /// revert firewall to its original state
    fn reset_policy(&mut self) -> ::std::result::Result<(), E>;
}

/// An abstraction around platform specific firewall implementation
pub struct FirewallProxy(Box<Firewall<imp::Error>>);

impl Firewall<Error> for FirewallProxy {
    fn new() -> Result<Self> {
        let firewall = imp::ConcreteFirewall::new().chain_err(|| ErrorKind::FirewallInitError)?;
        Ok(FirewallProxy(Box::new(firewall) as Box<Firewall<_>>))
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        self.0.apply_policy(policy).chain_err(|| ErrorKind::FirewallConfigurationError)
    }

    fn reset_policy(&mut self) -> Result<()> {
        self.0.reset_policy().chain_err(|| ErrorKind::FirewallConfigurationError)
    }
}
