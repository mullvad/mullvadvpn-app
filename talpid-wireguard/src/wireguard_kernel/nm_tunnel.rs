use crate::{config::MULLVAD_DNS_NAME, gotatun::GotaTun};

use super::{super::stats::StatsMap, Config, Error as WgKernelError, Tunnel, TunnelError};
use std::collections::HashMap;
use talpid_dbus::{
    dbus,
    network_manager::{
        DeviceConfig, Error as NetworkManagerError, NMDevice, NetworkManager, Variant, VariantMap,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while communicating over Dbus")]
    Dbus(#[from] dbus::Error),

    #[error("NetworkManager error")]
    NetworkManager(#[from] NetworkManagerError),
}

pub struct NetworkManagerTunnel {
    network_manager: NetworkManager,
    network_manager_device: NMDevice,
    tunnel: GotaTun,
}

impl NetworkManagerTunnel {
    pub fn new(tunnel: GotaTun, config: &Config) -> std::result::Result<Self, WgKernelError> {
        let network_manager = NetworkManager::new()
            .map_err(Error::NetworkManager)
            .map_err(WgKernelError::NetworkManager)?;
        let config_map = convert_config_to_dbus(config);
        let _network_manager_device = network_manager
            .create_network_device(&config_map)
            .map_err(|err| WgKernelError::NetworkManager(err.into()))?;
        Ok(NetworkManagerTunnel {
            network_manager,
            tunnel,
            network_manager_device: _network_manager_device,
        })
    }
}

#[async_trait::async_trait]
impl Tunnel for NetworkManagerTunnel {
    fn get_interface_name(&self) -> String {
        self.tunnel.get_interface_name()
    }

    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError> {
        if let Err(err) = self
            .network_manager
            .remove_network_device(self.network_manager_device)
        {
            log::error!("Failed to remove WireGuard tunnel via NM: {}", err);
            // TODO: Propagate error ?
        }
        Box::new(self.tunnel).stop()
    }

    async fn get_tunnel_stats(&self) -> std::result::Result<StatsMap, TunnelError> {
        self.tunnel.get_tunnel_stats().await
    }
}

fn convert_config_to_dbus(config: &Config) -> DeviceConfig {
    let mut ipv6_config: VariantMap = HashMap::new();
    let mut ipv4_config: VariantMap = HashMap::new();
    let mut connection_config: VariantMap = HashMap::new();

    // TODO: Document `dummy` type.
    connection_config.insert("type".into(), Variant(Box::new("dummy".to_string())));
    //connection_config.insert("type".into(), Variant(Box::new("wireguard".to_string())));
    connection_config.insert("id".into(), Variant(Box::new(MULLVAD_DNS_NAME.to_string())));
    connection_config.insert(
        "interface-name".into(),
        Variant(Box::new(MULLVAD_DNS_NAME.to_string())),
    );
    connection_config.insert("autoconnect".into(), Variant(Box::new(true)));

    let ipv4_addrs: Vec<_> = config
        .tunnel
        .addresses
        .iter()
        .filter(|ip| ip.is_ipv4())
        .map(NetworkManager::convert_address_to_dbus)
        .collect();

    let ipv6_addrs: Vec<_> = config
        .tunnel
        .addresses
        .iter()
        .filter(|ip| ip.is_ipv6())
        .map(NetworkManager::convert_address_to_dbus)
        .collect();

    ipv4_config.insert("address-data".into(), Variant(Box::new(ipv4_addrs)));
    ipv4_config.insert("ignore-auto-routes".into(), Variant(Box::new(true)));
    ipv4_config.insert("ignore-auto-dns".into(), Variant(Box::new(true)));
    ipv4_config.insert("may-fail".into(), Variant(Box::new(true)));
    ipv4_config.insert("method".into(), Variant(Box::new("manual".to_string())));
    ipv4_config.insert("never-default".into(), Variant(Box::new(true)));

    if !ipv6_addrs.is_empty() {
        ipv6_config.insert("method".into(), Variant(Box::new("manual".to_string())));
        ipv6_config.insert("address-data".into(), Variant(Box::new(ipv6_addrs)));
        ipv6_config.insert("ignore-auto-routes".into(), Variant(Box::new(true)));
        ipv6_config.insert("ignore-auto-dns".into(), Variant(Box::new(true)));
        ipv6_config.insert("may-fail".into(), Variant(Box::new(true)));
    }

    let mut settings = HashMap::new();
    settings.insert("ipv4".into(), ipv4_config);
    if !ipv6_config.is_empty() {
        settings.insert("ipv6".into(), ipv6_config);
    }
    settings.insert("connection".into(), connection_config);

    settings
}
