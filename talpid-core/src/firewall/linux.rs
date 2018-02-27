use super::{Firewall, SecurityPolicy};

error_chain!{}

/// The Linux implementation for the `Firewall` trait.
pub struct Netfilter;
impl Firewall for Netfilter {
    type Error = Error;

    fn new() -> Result<Self> {
        Ok(Netfilter)
    }

    fn apply_policy(&mut self, _policy: SecurityPolicy) -> Result<()> {
        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        Ok(())
    }
}
