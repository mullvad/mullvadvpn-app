use super::{
    super::stats::{Error as StatsError, Stats},
    Config, Error as WgKernelError, Tunnel, TunnelError, MULLVAD_INTERFACE_NAME,
};
use std::collections::HashMap;
use talpid_dbus::{
    dbus,
    network_manager::{
        DeviceConfig, Error as NetworkManagerError, NetworkManager, Variant, VariantMap,
        WireguardTunnel,
    },
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
        let config_map = convert_config_to_dbus(config);
        let tunnel = network_manager
            .create_wg_tunnel(&config_map)
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

fn convert_config_to_dbus(config: &Config) -> DeviceConfig {
    let mut ipv6_config: VariantMap = HashMap::new();
    let mut ipv4_config: VariantMap = HashMap::new();
    let mut wireguard_config: VariantMap = HashMap::new();
    let mut connection_config: VariantMap = HashMap::new();
    let mut peer_configs = vec![];

    wireguard_config.insert("mtu".into(), Variant(Box::new(config.mtu as u32)));
    wireguard_config.insert("fwmark".into(), Variant(Box::new(config.fwmark as u32)));
    wireguard_config.insert("peer-routes".into(), Variant(Box::new(false)));
    wireguard_config.insert(
        "private-key".into(),
        Variant(Box::new(config.tunnel.private_key.to_base64())),
    );
    wireguard_config.insert("private-key-flags".into(), Variant(Box::new(0x0u32)));

    for peer in config.peers.iter() {
        let mut peer_config: VariantMap = HashMap::new();
        let allowed_ips = peer
            .allowed_ips
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();


        peer_config.insert("allowed-ips".into(), Variant(Box::new(allowed_ips)));
        peer_config.insert(
            "endpoint".into(),
            Variant(Box::new(peer.endpoint.to_string())),
        );
        peer_config.insert(
            "public-key".into(),
            Variant(Box::new(peer.public_key.to_base64())),
        );

        peer_configs.push(peer_config);
    }
    wireguard_config.insert("peers".into(), Variant(Box::new(peer_configs)));

    connection_config.insert("type".into(), Variant(Box::new("wireguard".to_string())));
    connection_config.insert(
        "id".into(),
        Variant(Box::new(MULLVAD_INTERFACE_NAME.to_string())),
    );
    connection_config.insert(
        "interface-name".into(),
        Variant(Box::new(MULLVAD_INTERFACE_NAME.to_string())),
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
    settings.insert("wireguard".into(), wireguard_config);
    settings.insert("connection".into(), connection_config);

    settings
}
