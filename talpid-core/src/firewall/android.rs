use super::{FirewallArguments, FirewallPolicy, FirewallT};
use crate::tunnel::tun_provider::{TunConfig, TunProvider};
use ipnetwork::IpNetwork;
use std::{
    fs::File,
    mem,
    net::{IpAddr, Ipv4Addr},
    os::unix::io::{FromRawFd, RawFd},
    sync::Arc,
};
use talpid_types::BoxedError;

/// Stub error type for Firewall errors on Android.
#[derive(Debug, derive_more::From, err_derive::Error)]
pub enum Error {
    /// Failed to open VPN tunnel.
    #[error(display = "Failed to open VPN tunnel used by the firewall")]
    TunProvider(#[error(cause)] BoxedError),
}

/// The Android stub implementation for the firewall.
pub struct Firewall {
    tun_provider: Arc<dyn TunProvider>,
    active_tun: Option<RawFd>,
    blocking_config: TunConfig,
}

impl FirewallT for Firewall {
    type Error = Error;

    fn new(args: FirewallArguments) -> Result<Self, Self::Error> {
        Ok(Firewall {
            tun_provider: args.tun_provider,
            active_tun: None,
            blocking_config: TunConfig::blocking_config(),
        })
    }

    fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Self::Error> {
        match policy {
            FirewallPolicy::Connecting { .. } | FirewallPolicy::Blocked { .. } => {
                let tun = self.tun_provider.create_tun(self.blocking_config.clone())?;
                self.active_tun = Some(tun.as_raw_fd());
            }
            FirewallPolicy::Connected { .. } => self
                .reset_policy()
                .expect("Unexpected error from Firewall::reset_policy()"),
        }

        Ok(())
    }

    fn reset_policy(&mut self) -> Result<(), Self::Error> {
        if let Some(tun) = self.active_tun.take() {
            mem::drop(unsafe { File::from_raw_fd(tun) });
        }

        Ok(())
    }
}

impl TunConfig {
    /// Build a simple tunnel configuration to be used by the firewall to drop packets.
    ///
    /// The only field that matters is `routes`. It determines which packets end up in the tunnel,
    /// and therefore which packets are dropped. For this case, we simply drop all packets. All
    /// other fields are ignored.
    pub fn blocking_config() -> Self {
        TunConfig {
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            dns_servers: Vec::new(),
            routes: vec![IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                .expect("Invalid IP network prefix")],
            mtu: 1380,
        }
    }
}
