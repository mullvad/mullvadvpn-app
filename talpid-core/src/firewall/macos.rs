use super::{Firewall, SecurityPolicy};

// alias used to instantiate firewall implementation
pub type ConcreteFirewall = PacketFilter;

error_chain!{}

pub struct PacketFilter;
impl Firewall<Error> for PacketFilter {
    fn new() -> Result<Self> {
        Ok(PacketFilter)
    }

    fn apply_policy(&mut self, _policy: SecurityPolicy) -> Result<()> {
        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        Ok(())
    }
}
