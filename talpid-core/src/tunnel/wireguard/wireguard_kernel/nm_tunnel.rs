use super::{
    super::stats::Stats, wg_message::DeviceNla, Config, Error as WgKernelError, Handle, Tunnel,
    TunnelError, MULLVAD_INTERFACE_NAME,
};
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
const NM_CONNECTION_ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const NM_MANAGER: &str = "org.freedesktop.NetworkManager";
const NM_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";

const NM_ADD_CONNECTION_VOLATILE: u32 = 0x2;

const RPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

const DBUS_UNKNOWN_METHOD: &str = "org.freedesktop.DBus.Error.UnknownMethod";

const MINIMUM_SUPPORTED_MAJOR_VERSION: u32 = 1;
const MINIMUM_SUPPORTED_MINOR_VERSION: u32 = 16;


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

    #[error(display = "Error while removing ")]
    DeviceRemovalError(#[error(source)] dbus::Error),

    #[error(display = "Configuration has no device associated to it")]
    NoDevice,

    #[error(display = "NetworkManager is too old - {}", _0)]
    NMTooOld(String),

    #[error(display = "Cannot obtain the tunnel interface index")]
    FindInterfaceIndex,
}

pub struct NetworkManagerTunnel {
    dbus_connection: Connection,
    tunnel: Option<WireguardTunnel>,
    interface_index: u32,
    netlink_connections: Handle,
    tokio_handle: tokio::runtime::Handle,
}


type VariantRefArg = Variant<Box<dyn RefArg>>;
type VariantMap = HashMap<String, VariantRefArg>;
// settings are a{sa{sv}}
type DbusSettings = HashMap<String, VariantMap>;

impl NetworkManagerTunnel {
    pub fn new(
        tokio_handle: tokio::runtime::Handle,
        config: &Config,
    ) -> std::result::Result<Self, WgKernelError> {
        let mut dbus_connection = Connection::new_system()
            .map_err(|error| WgKernelError::NetworkManager(Error::Dbus(error)))?;
        Self::ensure_nm_is_new_enough(&dbus_connection).map_err(WgKernelError::NetworkManager)?;
        let tunnel = Some(
            Self::create_wg_tunnel(&mut dbus_connection, config)
                .map_err(WgKernelError::NetworkManager)?,
        );

        tokio_handle.clone().block_on(async move {
            let netlink_connections = Handle::connect().await?;
            let interface_index = Self::find_device_index(&netlink_connections)
                .await?
                .ok_or(WgKernelError::NetworkManager(Error::FindInterfaceIndex))?;

            Ok(NetworkManagerTunnel {
                dbus_connection,
                tunnel,
                interface_index,
                tokio_handle,
                netlink_connections,
            })
        })
    }

    async fn find_device_index(
        netlink_connections: &Handle,
    ) -> std::result::Result<Option<u32>, WgKernelError> {
        let mut wg = netlink_connections.wg_handle.clone();
        let device = wg.get_by_name(MULLVAD_INTERFACE_NAME.to_string()).await?;

        for nla in &device.nlas {
            if let DeviceNla::IfIndex(index) = nla {
                return Ok(Some(*index));
            }
        }
        Ok(None)
    }

    fn ensure_nm_is_new_enough(connection: &Connection) -> Result<()> {
        let manager = Self::nm_proxy(connection);
        let version_string: String = manager.get(NM_MANAGER, "Version").map_err(Error::Dbus)?;
        let version_too_old = || Error::NMTooOld(version_string.clone());
        let mut parts = version_string
            .split(".")
            .map(|part| part.parse().map_err(|_| version_too_old()));

        let major_version: u32 = parts.next().ok_or_else(|| version_too_old())??;
        let minor_version: u32 = parts.next().ok_or_else(|| version_too_old())??;

        if major_version < MINIMUM_SUPPORTED_MAJOR_VERSION
            || minor_version < MINIMUM_SUPPORTED_MINOR_VERSION
        {
            Err(version_too_old())
        } else {
            Ok(())
        }
    }

    fn nm_proxy<'a>(connection: &'a Connection) -> Proxy<'a, &Connection> {
        Proxy::new(NM_BUS, NM_MANAGER_PATH, RPC_TIMEOUT, connection)
    }


    fn create_wg_tunnel(dbus_connection: &Connection, config: &Config) -> Result<WireguardTunnel> {
        let settings_map = Self::convert_config_to_dbus(config);

        let config_path: Path<'static> = Self::add_connection_2(dbus_connection, &settings_map)
            .map(|(path, _result)| path)
            .or_else(|err| {
                log::error!("Failed to create a new interface via NM - {}", err);
                match err {
                    Error::Dbus(dbus_error) if dbus_error.name() == Some(DBUS_UNKNOWN_METHOD) => {
                        Self::add_connection_unsaved(dbus_connection, &settings_map)
                    }
                    err => Err(err),
                }
            })?;

        let manager = Self::nm_proxy(dbus_connection);
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

    fn add_connection_2(
        connection: &Connection,
        settings_map: &DbusSettings,
    ) -> Result<(Path<'static>, DbusSettings)> {
        let args: VariantMap = HashMap::new();
        let new_device = Message::new_method_call(
            NM_BUS,
            NM_SETTINGS_PATH,
            NM_INTERFACE_SETTINGS,
            "AddConnection2",
        )
        .map_err(Error::DbusMethodCall)?
        .append3(settings_map, NM_ADD_CONNECTION_VOLATILE, args);

        connection
            .send_with_reply_and_block(new_device, RPC_TIMEOUT)
            .map_err(Error::Dbus)?
            .read2()
            .map_err(Error::MatchDBusTypeError)
    }

    fn add_connection_unsaved(
        connection: &Connection,
        settings_map: &DbusSettings,
    ) -> Result<Path<'static>> {
        let new_connection = Message::new_method_call(
            NM_BUS,
            NM_SETTINGS_PATH,
            NM_INTERFACE_SETTINGS,
            "AddConnectionUnsaved",
        )
        .map_err(Error::DbusMethodCall)?
        .append1(settings_map);

        connection
            .send_with_reply_and_block(new_connection, RPC_TIMEOUT)
            .map_err(Error::Dbus)?
            .read1()
            .map_err(Error::MatchDBusTypeError)
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

impl Tunnel for NetworkManagerTunnel {
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
        MULLVAD_INTERFACE_NAME.to_string()
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
        let mut wg = self.netlink_connections.wg_handle.clone();
        let interface_index = self.interface_index;
        let result = self.tokio_handle.block_on(async move {
            let device = wg.get_by_index(interface_index).await.map_err(|err| {
                log::error!("Failed to fetch WireGuard device config: {}", err);
                TunnelError::GetConfigError
            })?;
            Ok(Stats::parse_device_message(&device))
        });

        result
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
