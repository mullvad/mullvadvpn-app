use super::{FirewallArguments, FirewallPolicy, FirewallT};
use crate::tunnel::tun_provider::TunProvider;
use std::sync::Arc;
use talpid_types::BoxedError;

/// Stub error type for Firewall errors on Android.
#[derive(Debug, err_derive::Error)]
pub enum Error {
    /// Failed to close the VPN tunnel.
    #[error(display = "Failed to close the VPN tunnel to disable the firewall")]
    CloseTunnel(#[error(cause)] BoxedError),

    /// Failed to open VPN tunnel.
    #[error(display = "Failed to open VPN tunnel used by the firewall")]
    OpenTunnel(#[error(cause)] BoxedError),
}

/// The Android stub implementation for the firewall.
pub struct Firewall {
    tun_provider: Arc<dyn TunProvider>,
}

impl FirewallT for Firewall {
    type Error = Error;

    fn new(args: FirewallArguments) -> Result<Self, Self::Error> {
        Ok(Firewall {
            tun_provider: args.tun_provider,
        })
    }

    fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Self::Error> {
        match policy {
            FirewallPolicy::Connecting { .. } | FirewallPolicy::Blocked { .. } => {
                self.tun_provider.open_tun().map_err(Error::OpenTunnel)
            }
            FirewallPolicy::Connected { .. } => Ok(()),
        }
    }

    fn reset_policy(&mut self) -> Result<(), Self::Error> {
        self.tun_provider.close_tun().map_err(Error::CloseTunnel)
    }
}
