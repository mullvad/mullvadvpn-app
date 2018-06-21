use error_chain::ChainedError;
use std::path::Path;

use super::{Firewall, SecurityPolicy};

mod dns;

use self::dns::DnsSettings;

error_chain! {
    links {
        DnsSettings(self::dns::Error, self::dns::ErrorKind) #[doc = "DNS error"];
    }
}

/// The Linux implementation for the `Firewall` trait.
pub struct Netfilter {
    dns_settings: DnsSettings,
}

impl Firewall for Netfilter {
    type Error = Error;

    fn new<P: AsRef<Path>>(_cache_dir: P) -> Result<Self> {
        Ok(Netfilter {
            dns_settings: DnsSettings::new()?,
        })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        match policy {
            SecurityPolicy::Connected { tunnel, .. } => {
                self.dns_settings.set_dns(vec![tunnel.gateway.into()])?;
            }
            _ => (),
        }

        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        if let Err(error) = self.dns_settings.reset() {
            warn!("Failed to reset DNS settings: {}", error.display_chain());
        }

        Ok(())
    }
}
