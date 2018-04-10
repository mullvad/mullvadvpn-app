use error_chain::ChainedError;

use super::{Firewall, SecurityPolicy};

mod dns;

use self::dns::DnsMonitor;

error_chain! {
    links {
        DnsMonitor(self::dns::Error, self::dns::ErrorKind) #[doc = "DNS error"];
    }
}

/// The Linux implementation for the `Firewall` trait.
pub struct Netfilter {
    dns_monitor: DnsMonitor,
}

impl Firewall for Netfilter {
    type Error = Error;

    fn new() -> Result<Self> {
        Ok(Netfilter {
            dns_monitor: DnsMonitor::new()?,
        })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        match policy {
            SecurityPolicy::Connected { tunnel, .. } => {
                self.dns_monitor.set_dns(vec![tunnel.gateway.to_string()])?;
            }
            _ => (),
        }

        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        if let Err(error) = self.dns_monitor.reset() {
            warn!("Failed to reset DNS settings: {}", error.display_chain());
        }

        Ok(())
    }
}
