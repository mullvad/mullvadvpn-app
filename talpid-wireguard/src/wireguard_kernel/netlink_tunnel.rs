use std::pin::Pin;

use futures::Future;

use super::{
    super::stats::{Stats, StatsMap},
    wg_message::DeviceNla,
    Config, Error, Handle, Tunnel, TunnelError, MULLVAD_INTERFACE_NAME,
};

pub struct NetlinkTunnel {
    interface_index: u32,
    netlink_connections: Handle,
    tokio_handle: tokio::runtime::Handle,
}

impl NetlinkTunnel {
    pub fn new(tokio_handle: tokio::runtime::Handle, config: &Config) -> Result<Self, Error> {
        tokio_handle.clone().block_on(async {
            let mut netlink_connections = Handle::connect().await?;
            let interface_index = netlink_connections
                .create_device(MULLVAD_INTERFACE_NAME.to_string(), config.mtu as u32)
                .await?;

            let mut tunnel = Self {
                interface_index,
                netlink_connections,
                tokio_handle,
            };

            if let Err(err) = tunnel.setup(config).await {
                if let Err(teardown_err) = tunnel
                    .netlink_connections
                    .delete_device(interface_index)
                    .await
                {
                    log::error!(
                        "Failed to tear down WireGuard interface after failing to apply config: {}",
                        teardown_err
                    );
                }
                return Err(err);
            }

            Ok(tunnel)
        })
    }

    async fn setup(&mut self, config: &Config) -> Result<(), Error> {
        self.netlink_connections
            .wg_handle
            .set_config(self.interface_index, config)
            .await?;

        for tunnel_ip in config.tunnel.addresses.iter() {
            self.netlink_connections
                .set_ip_address(self.interface_index, *tunnel_ip)
                .await?;
        }

        Ok(())
    }
}

impl Tunnel for NetlinkTunnel {
    fn get_interface_name(&self) -> String {
        let mut wg = self.netlink_connections.wg_handle.clone();
        let result = self.tokio_handle.block_on(async move {
            let device = wg.get_by_index(self.interface_index).await?;
            for nla in device.nlas {
                if let DeviceNla::IfName(name) = nla {
                    return Ok(name);
                }
            }
            Err(Error::Truncated)
        });

        match result {
            Ok(name) => name.to_string_lossy().to_string(),
            Err(err) => {
                log::error!("Failed to deduce interface name at runtime, will attempt to use the default name. {}", err);
                MULLVAD_INTERFACE_NAME.to_string()
            }
        }
    }

    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError> {
        let Self {
            mut netlink_connections,
            interface_index,
            tokio_handle,
        } = *self;
        tokio_handle.block_on(async move {
            if let Err(err) = netlink_connections.delete_device(interface_index).await {
                log::error!("Failed to remove WireGuard device: {}", err);
                Err(TunnelError::FatalStartWireguardError)
            } else {
                Ok(())
            }
        })
    }

    fn get_tunnel_stats(&self) -> std::result::Result<StatsMap, TunnelError> {
        let mut wg = self.netlink_connections.wg_handle.clone();
        let interface_index = self.interface_index;
        self.tokio_handle.block_on(async move {
            let device = wg.get_by_index(interface_index).await.map_err(|err| {
                log::error!("Failed to fetch WireGuard device config: {}", err);
                TunnelError::GetConfigError
            })?;
            Ok(Stats::parse_device_message(&device))
        })
    }

    fn set_config(
        &self,
        config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send + 'static>> {
        let mut wg = self.netlink_connections.wg_handle.clone();
        let interface_index = self.interface_index;
        Box::pin(async move {
            wg.set_config(interface_index, &config)
                .await
                .map_err(|err| {
                    log::error!("Failed to fetch WireGuard device config: {}", err);
                    TunnelError::SetConfigError
                })
        })
    }
}
