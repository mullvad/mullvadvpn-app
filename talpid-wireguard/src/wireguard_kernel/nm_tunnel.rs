use crate::{TunnelType, wireguard_kernel::nm_tunnel};

use super::{super::stats::StatsMap, Config, Tunnel, TunnelError};
use std::collections::HashMap;
use talpid_dbus::network_manager::{
    DeviceConfig, Error as NetworkManagerError, NMDevice, NetworkManagerConnection,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("NetworkManager error")]
    NetworkManager(#[from] NetworkManagerError),
}

/// [`NetworkManagerDevice`] is a generic network device. It can be used to configure system-wide DNS
/// on systems which use NetworkManager to configure their network(s). The [`NetworkManagerTunnel`]
/// type ties the lifetime of this generic network device to an instance of another WireGuard
/// implementation.
pub type NetworkManagerTunnel = (NetworkManagerDevice, TunnelType);

pub struct NetworkManagerDevice {
    network_manager: NetworkManagerConnection,
    network_manager_device: NMDevice,
    interface: String,
    handle: tokio::runtime::Handle,
}

impl NetworkManagerDevice {
    pub async fn dns(
        config: &Config,
        interface: String,
    ) -> std::result::Result<Self, nm_tunnel::Error> {
        let network_manager = NetworkManagerConnection::new()
            .await
            .map_err(Error::NetworkManager)?;
        let config_map = convert_config_to_dbus(config, interface.clone());
        let network_manager_device = network_manager.create_network_device(config_map).await?;
        Ok(NetworkManagerDevice {
            network_manager,
            network_manager_device,
            interface,
            handle: tokio::runtime::Handle::current(),
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
        let nm = self.0.network_manager;
        if let Err(err) = self
            .0
            .handle
            .block_on(nm.remove_network_device(self.0.network_manager_device))
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
    use zbus::{Array, OwnedVariant, Str};
    let mut ipv6_config = HashMap::new();
    let mut ipv4_config = HashMap::new();
    let mut connection_config = HashMap::new();

    connection_config.insert("type".into(), OwnedVariant::from(Str::from_static("dummy")));
    connection_config.insert(
        "id".into(),
        OwnedVariant::from(Str::from(interface.clone())),
    );
    connection_config.insert(
        "interface-name".into(),
        OwnedVariant::from(Str::from(interface)),
    );
    connection_config.insert("autoconnect".into(), OwnedVariant::from(true));

    let ipv4_addrs: Vec<_> = config
        .tunnel
        .addresses
        .iter()
        .filter(|ip| ip.is_ipv4())
        .map(NetworkManagerConnection::convert_address_to_dbus)
        .collect();

    let ipv6_addrs: Vec<_> = config
        .tunnel
        .addresses
        .iter()
        .filter(|ip| ip.is_ipv6())
        .map(NetworkManagerConnection::convert_address_to_dbus)
        .collect();

    ipv4_config.insert(
        "address-data".into(),
        OwnedVariant::try_from(Array::from(ipv4_addrs)).unwrap(),
    );
    ipv4_config.insert("ignore-auto-routes".into(), OwnedVariant::from(true));
    ipv4_config.insert("ignore-auto-dns".into(), OwnedVariant::from(true));
    ipv4_config.insert("may-fail".into(), OwnedVariant::from(true));
    ipv4_config.insert(
        "method".into(),
        OwnedVariant::from(Str::from_static("manual")),
    );
    ipv4_config.insert("never-default".into(), OwnedVariant::from(true));

    if !ipv6_addrs.is_empty() {
        ipv6_config.insert(
            "method".into(),
            OwnedVariant::from(Str::from_static("manual")),
        );
        ipv6_config.insert(
            "address-data".into(),
            OwnedVariant::try_from(Array::from(ipv6_addrs)).unwrap(),
        );
        ipv6_config.insert("ignore-auto-routes".into(), OwnedVariant::from(true));
        ipv6_config.insert("ignore-auto-dns".into(), OwnedVariant::from(true));
        ipv6_config.insert("may-fail".into(), OwnedVariant::from(true));
    }

    let mut settings = HashMap::new();
    settings.insert("ipv4".into(), ipv4_config);
    if !ipv6_config.is_empty() {
        settings.insert("ipv6".into(), ipv6_config);
    }
    settings.insert("connection".into(), connection_config);

    settings
}
