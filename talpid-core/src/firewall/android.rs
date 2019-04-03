use super::{FirewallPolicy, FirewallT};

/// Stub error type for Firewall errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Unknown Android Firewall error")]
pub struct Error;

/// The Android stub implementation for the firewall.
pub struct Firewall;

impl FirewallT for Firewall {
    type Error = Error;

    fn new() -> Result<Self, Self::Error> {
        Ok(Firewall)
    }

    fn apply_policy(&mut self, _policy: FirewallPolicy) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset_policy(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
