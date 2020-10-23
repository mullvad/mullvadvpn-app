use super::{config::Config, stats::Stats, Tunnel, TunnelError};
use dbus::{
    arg::{RefArg, Variant},
    blocking::{stdintf::org_freedesktop_dbus::Properties, BlockingSender, Connection, Proxy},
    message::Message,
    strings::Path,
};
use std::{collections::HashMap, net::IpAddr};

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_INTERFACE_SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";
const NM_INTERFACE_SETTINGS_CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const NM_SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";
const NM_DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const NM_DEVICE_STATISTICS: &str = "org.freedesktop.NetworkManager.Device.Statistics";
const NM_CONNECTION_ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const NM_MANAGER: &str = "org.freedesktop.NetworkManager";
const NM_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";

const NM_ADD_CONNECTION_VOLATILE: u32 = 0x2;

const TRAFFIC_STATS_REFRESH_RATE_MS: u32 = 1000;
const RPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Error while communicating over Dbus")]
    Dbus(#[error(source)] dbus::Error),

    #[error(display = "Failed to construct a DBus method call message")]
    DbusMethodCall(String),

    #[error(display = "Failed to match the returned D-Bus object with expected type")]
    MatchDBusTypeError(#[error(source)] dbus::arg::TypeMismatchError),

    #[error(display = "Failed to set statistics refresh rate")]
    SetStatsRefreshError(#[error(source)] dbus::Error),

    #[error(display = "Error while removing ")]
    DeviceRemovalError(#[error(source)] dbus::Error),

    #[error(display = "Configuration has no device associated to it")]
    NoDevice,
}

pub struct NetworkManager {
    dbus_connection: Connection,
    tunnel: Option<WireguardTunnel>,
}


type VariantRefArg = Variant<Box<dyn RefArg>>;
type VariantMap = HashMap<String, VariantRefArg>;
// settings are a{sa{sv}}
type DbusSettings = HashMap<String, VariantMap>;

impl NetworkManager {
    pub fn new(config: &Config) -> Result<Self> {
        let mut dbus_connection = Connection::new_system().map_err(Error::Dbus)?;
        let tunnel = Some(Self::create_wg_tunnel(&mut dbus_connection, config)?);
        let manager = NetworkManager {
            dbus_connection,
            tunnel,
        };

        manager.set_stats_refresh_rate()?;
        Ok(manager)
    }


    fn create_wg_tunnel(dbus_connection: &Connection, config: &Config) -> Result<WireguardTunnel> {
        let settings_map = Self::convert_config_to_dbus(config);
        let args: VariantMap = HashMap::new();
        let new_device = Message::new_method_call(
            NM_BUS,
            NM_SETTINGS_PATH,
            NM_INTERFACE_SETTINGS,
            "AddConnection2",
        )
        .map_err(Error::DbusMethodCall)?
        .append3(settings_map, NM_ADD_CONNECTION_VOLATILE, args);

        let application = dbus_connection
            .send_with_reply_and_block(new_device, RPC_TIMEOUT)
            .map_err(Error::Dbus)?;

        let (config_path, _result): (Path<'static>, DbusSettings) =
            application.read2().map_err(Error::MatchDBusTypeError)?;

        let manager = Proxy::new(NM_BUS, NM_MANAGER_PATH, RPC_TIMEOUT, dbus_connection);
        let (connection_path,): (Path<'static>,) = manager
            .method_call(
                NM_MANAGER,
                "ActivateConnection",
                (
                    &config_path,
                    &Path::new("/").unwrap(),
                    &Path::new("/").unwrap(),
                ),
            )
            .map_err(Error::Dbus)?;

        let connection = Proxy::new(NM_BUS, &connection_path, RPC_TIMEOUT, dbus_connection);
        let device_paths: Vec<Path<'static>> = connection
            .get(NM_CONNECTION_ACTIVE, "Devices")
            .map_err(Error::Dbus)?;
        let device_path = device_paths.into_iter().next().ok_or(Error::NoDevice)?;

        Ok(WireguardTunnel {
            config_path,
            connection_path,
            device_path,
        })
    }

    fn convert_config_to_dbus(config: &Config) -> DbusSettings {
        let mut ipv6_config: VariantMap = HashMap::new();
        let mut ipv4_config: VariantMap = HashMap::new();
        let mut wireguard_config: VariantMap = HashMap::new();
        let mut connection_config: VariantMap = HashMap::new();
        let mut peer_configs = vec![];

        wireguard_config.insert("mtu".into(), Variant(Box::new(config.mtu as u32)));
        wireguard_config.insert("fwmark".into(), Variant(Box::new(config.fwmark as u32)));
        wireguard_config.insert(
            "private-key".into(),
            Variant(Box::new(config.tunnel.private_key.to_base64())),
        );
        wireguard_config.insert("private-key-flags".into(), Variant(Box::new(0x0u32)));
        wireguard_config.insert("ip4-auto-default-route".into(), Variant(Box::new(0u32)));
        wireguard_config.insert("ip6-auto-default-route".into(), Variant(Box::new(0u32)));

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
        connection_config.insert("id".into(), Variant(Box::new("wg-mullvad".to_string())));
        connection_config.insert(
            "interface-name".into(),
            Variant(Box::new("wg-mullvad".to_string())),
        );
        connection_config.insert("autoconnect".into(), Variant(Box::new(true)));


        let ipv4_addrs: Vec<_> = config
            .tunnel
            .addresses
            .iter()
            .filter(|ip| ip.is_ipv4())
            .map(Self::convert_address_to_dbus)
            .collect();

        let ipv6_addrs: Vec<_> = config
            .tunnel
            .addresses
            .iter()
            .filter(|ip| ip.is_ipv6())
            .map(Self::convert_address_to_dbus)
            .collect();

        ipv4_config.insert("address-data".into(), Variant(Box::new(ipv4_addrs)));
        ipv4_config.insert("ignore-auto-routes".into(), Variant(Box::new(true)));
        ipv4_config.insert("ignore-auto-dns".into(), Variant(Box::new(true)));
        ipv4_config.insert("may-fail".into(), Variant(Box::new(true)));
        ipv4_config.insert("method".into(), Variant(Box::new("manual".to_string())));

        if !ipv6_addrs.is_empty() {
            ipv6_config.insert("address-data".into(), Variant(Box::new(ipv6_addrs)));
            ipv6_config.insert("ignore-auto-routes".into(), Variant(Box::new(true)));
            ipv6_config.insert("ignore-auto-dns".into(), Variant(Box::new(true)));
            ipv6_config.insert("may-fail".into(), Variant(Box::new(true)));
            ipv6_config.insert("method".into(), Variant(Box::new("manual".to_string())));
        }


        let mut settings = HashMap::new();
        settings.insert("ipv4".into(), ipv4_config);
        settings.insert("ipv6".into(), ipv6_config);
        settings.insert("wireguard".into(), wireguard_config);
        settings.insert("connection".into(), connection_config);

        settings
    }

    fn set_stats_refresh_rate(&self) -> Result<()> {
        if let Some(tunnel) = self.tunnel.as_ref() {
            tunnel
                .device_proxy(&self.dbus_connection)
                .set(
                    NM_DEVICE_STATISTICS,
                    "RefreshRateMs",
                    TRAFFIC_STATS_REFRESH_RATE_MS,
                )
                .map_err(Error::SetStatsRefreshError)?;
        }
        Ok(())
    }

    fn convert_address_to_dbus(address: &IpAddr) -> VariantMap {
        let mut map: VariantMap = HashMap::new();
        map.insert(
            "address".to_string(),
            Variant(Box::new(address.to_string())),
        );
        let prefix: u32 = if address.is_ipv4() { 32 } else { 128 };
        map.insert("prefix".to_string(), Variant(Box::new(prefix)));

        map
    }

    fn remove_config(&mut self) -> Result<()> {
        if let Some(tunnel) = self.tunnel.take() {
            let deactivation_result: Result<()> =
                Proxy::new(NM_BUS, NM_MANAGER_PATH, RPC_TIMEOUT, &self.dbus_connection)
                    .method_call(
                        NM_MANAGER,
                        "DeactivateConnection",
                        (&tunnel.connection_path,),
                    )
                    .map_err(Error::Dbus);

            let device_result: Result<()> = tunnel
                .device_proxy(&self.dbus_connection)
                .method_call(NM_DEVICE, "Delete", ())
                .map_err(Error::DeviceRemovalError);

            let config_result: Result<()> = tunnel
                .config_proxy(&self.dbus_connection)
                .method_call(NM_INTERFACE_SETTINGS_CONNECTION, "Delete", ())
                .map_err(Error::DeviceRemovalError);
            deactivation_result?;
            device_result?;
            config_result?;
        }
        Ok(())
    }
}

impl Tunnel for NetworkManager {
    fn get_interface_name(&self) -> String {
        if let Some(tunnel) = self.tunnel.as_ref() {
            let interface_name = tunnel
                .device_proxy(&self.dbus_connection)
                .get(NM_DEVICE, "Interface");

            match interface_name {
                Ok(name) => {
                    return name;
                }
                Err(error) => log::error!("Failed to fetch interface name from NM: {}", error),
            }
        }
        "wg-mullvad".to_string()
    }

    fn stop(mut self: Box<Self>) -> std::result::Result<(), TunnelError> {
        if let Err(err) = self.remove_config() {
            log::error!("Failed to remove WireGuard tunnel via NM: {}", err);
            Err(TunnelError::StopWireguardError { status: 0 })
        } else {
            Ok(())
        }
    }

    fn get_tunnel_stats(&self) -> std::result::Result<Stats, TunnelError> {
        if let Some(tunnel) = self.tunnel.as_ref() {
            let device = tunnel.device_proxy(&self.dbus_connection);
            let get_device_stats = || -> std::result::Result<Stats, dbus::Error> {
                let rx_bytes = device.get(NM_DEVICE_STATISTICS, "RxBytes")?;
                let tx_bytes = device.get(NM_DEVICE_STATISTICS, "TxBytes")?;

                Ok(Stats { rx_bytes, tx_bytes })
            };

            match get_device_stats() {
                Ok(stats) => Ok(stats),
                Err(err) => {
                    log::error!("Failed to read tunnel stats from NM: {}", err);
                    Err(TunnelError::StatsError(super::stats::Error::NoTunnelDevice))
                }
            }
        } else {
            Err(TunnelError::StatsError(super::stats::Error::NoTunnelDevice))
        }
    }
}


struct WireguardTunnel {
    config_path: Path<'static>,
    connection_path: Path<'static>,
    device_path: Path<'static>,
}

impl WireguardTunnel {
    fn device_proxy<'a>(&'a self, connection: &'a Connection) -> Proxy<'a, &Connection> {
        Proxy::new(NM_BUS, &self.device_path, RPC_TIMEOUT, connection)
    }

    fn config_proxy<'a>(&'a self, connection: &'a Connection) -> Proxy<'a, &Connection> {
        Proxy::new(NM_BUS, &self.config_path, RPC_TIMEOUT, connection)
    }
}
