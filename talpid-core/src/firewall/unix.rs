use super::{Firewall, SecurityPolicy};

// alias used to instantiate firewall implementation
pub type ConcreteFirewall = Netfilter;

error_chain!{}

pub struct Netfilter;
impl Firewall<Error> for Netfilter {
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
