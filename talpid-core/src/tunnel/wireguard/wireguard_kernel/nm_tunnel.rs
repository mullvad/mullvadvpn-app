use super::{
    super::stats::{Error as StatsError, Stats},
    Config, Error as WgKernelError, Tunnel, TunnelError, MULLVAD_INTERFACE_NAME,
};
use crate::linux::network_manager::{
    Error as NetworkManagerError, NetworkManager, WireguardTunnel,
};
use talpid_types::ErrorExt;


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Error while communicating over Dbus")]
    Dbus(#[error(source)] dbus::Error),

    #[error(display = "NetworkManager error")]
    NetworkManager(#[error(source)] NetworkManagerError),
}

const TRAFFIC_STATS_REFRESH_RATE_MS: u32 = 1000;
const INITIAL_TRAFFIC_STATS_REFRESH_RATE_MS: u32 = 10;

pub struct NetworkManagerTunnel {
    network_manager: NetworkManager,
    tunnel: Option<WireguardTunnel>,
}


impl NetworkManagerTunnel {
    pub fn new(config: &Config) -> std::result::Result<Self, WgKernelError> {
        let network_manager = NetworkManager::new()
            .map_err(Error::NetworkManager)
            .map_err(WgKernelError::NetworkManager)?;
        let tunnel = network_manager
            .create_wg_tunnel(config)
            .map_err(|err| WgKernelError::NetworkManager(err.into()))?;

        network_manager
            .set_stats_refresh_rate(&tunnel, INITIAL_TRAFFIC_STATS_REFRESH_RATE_MS)
            .map_err(|err| WgKernelError::NetworkManager(err.into()))?;


        Ok(NetworkManagerTunnel {
            network_manager,
            tunnel: Some(tunnel),
        })
    }
}

impl Tunnel for NetworkManagerTunnel {
    fn get_interface_name(&self) -> String {
        if let Some(tunnel) = self.tunnel.as_ref() {
            match self.network_manager.get_interface_name(tunnel) {
                Ok(name) => {
                    return name;
                }
                Err(error) => log::error!("Failed to fetch interface name from NM: {}", error),
            }
        }
        MULLVAD_INTERFACE_NAME.to_string()
    }

    fn stop(mut self: Box<Self>) -> std::result::Result<(), TunnelError> {
        if let Some(tunnel) = self.tunnel.take() {
            if let Err(err) = self.network_manager.remove_tunnel(tunnel) {
                log::error!("Failed to remove WireGuard tunnel via NM: {}", err);
                Err(TunnelError::StopWireguardError { status: 0 })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn get_tunnel_stats(&self) -> std::result::Result<Stats, TunnelError> {
        let tunnel = self
            .tunnel
            .as_ref()
            .ok_or(TunnelError::StatsError(StatsError::NoTunnelDevice))?;
        let (tx_bytes, rx_bytes) = self
            .network_manager
            .get_tunnel_stats(tunnel)
            .map_err(|_| TunnelError::StatsError(StatsError::KeyNotFoundError))?;

        Ok(Stats { tx_bytes, rx_bytes })
    }

    fn slow_stats_refresh_rate(&self) {
        if let Some(tunnel) = self.tunnel.as_ref() {
            if let Err(err) = self
                .network_manager
                .set_stats_refresh_rate(tunnel, TRAFFIC_STATS_REFRESH_RATE_MS)
            {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to reset stats refresh rate: ")
                );
            }
        }
    }
}
