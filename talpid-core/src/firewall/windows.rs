use super::{Firewall, SecurityPolicy};

// alias used to instantiate firewall implementation
pub type ConcreteFirewall = WindowsFirewall;

error_chain!{}

pub struct WindowsFirewall;
impl Firewall<Error> for WindowsFirewall {
    fn new() -> Result<Self> {
        Ok(WindowsFirewall)
    }

    fn apply_policy(&mut self, _policy: SecurityPolicy) -> Result<()> {
        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        Ok(())
    }
}
