use super::{FirewallPolicy, FirewallT};

error_chain! {}

/// The Android stub implementation for the firewall.
pub struct Firewall;

impl FirewallT for Firewall {
    type Error = Error;

    fn new() -> Result<Self> {
        Ok(Firewall)
    }

    fn apply_policy(&mut self, _policy: FirewallPolicy) -> Result<()> {
        Ok(())
    }

    fn reset_policy(&mut self) -> Result<()> {
        Ok(())
    }
}
