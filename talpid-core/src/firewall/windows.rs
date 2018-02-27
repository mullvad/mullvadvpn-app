use super::{Firewall, SecurityPolicy};

error_chain!{}

/// The Windows implementation for the `Firewall` trait.
pub struct WindowsFirewall;
impl Firewall for WindowsFirewall {
    type Error = Error;

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
