use crate::{gotatun::GotaTun, wireguard_kernel::nm_tunnel};

use super::{super::stats::StatsMap, Config, Tunnel, TunnelError};
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

/// [`NetworkManagerDevice`] is a generic network device. It can be used to configure system-wide DNS
/// on systems which use NetworkManager to configure their network(s). The [`NetworkManagerTunnel`]
/// type ties the lifetime of this generic network device to an instance of another WireGuard
/// implementation.
pub type NetworkManagerTunnel = (NetworkManagerDevice, GotaTun);

pub struct NetworkManagerDevice {
    network_manager: NetworkManager,
    network_manager_device: NMDevice,
    interface: String,
}

impl NetworkManagerDevice {
    pub fn dns(config: &Config, interface: String) -> std::result::Result<Self, nm_tunnel::Error> {
        let network_manager = NetworkManager::new().map_err(Error::NetworkManager)?;
        let config_map = convert_config_to_dbus(config, interface.clone());
        let network_manager_device = network_manager.create_network_device(&config_map)?;
        Ok(NetworkManagerDevice {
            network_manager,
            network_manager_device,
            interface,
        })
    }

    pub fn interface_name(&self) -> &str {
        &self.interface
    }
}

#[async_trait::async_trait]
impl Tunnel for NetworkManagerTunnel {
    fn get_interface_name(&self) -> String {
        self.1.get_interface_name()
    }

    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError> {
        if let Err(err) = self
            .0
            .network_manager
            .remove_network_device(self.0.network_manager_device)
        {
            log::error!("Failed to remove NetworkManager device: {}", err);
        }
        Box::new(self.1).stop()
    }

    async fn get_tunnel_stats(&self) -> std::result::Result<StatsMap, TunnelError> {
        self.1.get_tunnel_stats().await
    }
}

fn convert_config_to_dbus(config: &Config, interface: String) -> DeviceConfig {
    let mut ipv6_config: VariantMap = HashMap::new();
    let mut ipv4_config: VariantMap = HashMap::new();
    let mut connection_config: VariantMap = HashMap::new();

    connection_config.insert("type".into(), Variant(Box::new("dummy".to_string())));
    connection_config.insert("id".into(), Variant(Box::new(interface.clone())));
    connection_config.insert("interface-name".into(), Variant(Box::new(interface)));
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
