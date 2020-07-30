use dbus::{
    arg::{RefArg, Variant},
    stdintf::*,
    BusType, Message,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::IpAddr,
    path::Path,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "NetworkManager not detected")]
    NetworkManagerNotDetected(#[error(source)] dbus::Error),

    #[error(display = "NetworkManager is too old")]
    TooOldNetworkManager(#[error(source)] dbus::Error),

    #[error(display = "NetworkManager is not managing DNS")]
    NetworkManagerNotManagingDns,

    #[error(display = "Error while communicating over Dbus")]
    Dbus(#[error(source)] dbus::Error),

    #[error(display = "Failed to construct DBus method call message")]
    DbusMethodCall(String),

    #[error(display = "Failed to match the returned D-Bus object with expected type")]
    MatchDBusTypeError(#[error(source)] dbus::arg::TypeMismatchError),

    #[error(display = "DNS is managed by systemd-resolved - NM can't enforce DNS globally")]
    SystemdResolved,

    #[error(display = "Failed to find obtain devices from network manager")]
    ObtainDevices,

    #[error(display = "Failed to find link interface in network manager")]
    LinkNotFound,
}

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_TOP_OBJECT: &str = "org.freedesktop.NetworkManager";
const NM_DNS_MANAGER: &str = "org.freedesktop.NetworkManager.DnsManager";
const NM_DNS_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";
const NM_OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";
const NM_DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const NM_CONNECTION_ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const NM_SETTINGS_CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const RPC_TIMEOUT_MS: i32 = 3000;
const GLOBAL_DNS_CONF_KEY: &str = "GlobalDnsConfiguration";
const RC_MANAGEMENT_MODE_KEY: &str = "RcManager";
const DNS_MODE_KEY: &str = "Mode";
const DNS_FIRST_PRIORITY: i32 = -2147483647;

pub struct NetworkManager {
    dbus_connection: dbus::Connection,
}


impl NetworkManager {
    pub fn new() -> Result<Self> {
        let dbus_connection =
            dbus::Connection::get_private(BusType::System).map_err(Error::Dbus)?;
        let manager = NetworkManager { dbus_connection };
        manager.ensure_resolv_conf_is_managed()?;
        manager.ensure_network_manager_exists()?;
        Ok(manager)
    }

    fn ensure_network_manager_exists(&self) -> Result<()> {
        let _: Box<dyn RefArg> = self
            .as_manager()
            .get(&NM_TOP_OBJECT, GLOBAL_DNS_CONF_KEY)
            .map_err(Error::NetworkManagerNotDetected)?;
        Ok(())
    }

    fn ensure_resolv_conf_is_managed(&self) -> Result<()> {
        // check if NM is set to manage resolv.conf
        let management_mode: String = self
            .dbus_connection
            .with_path(NM_BUS, NM_DNS_MANAGER_PATH, RPC_TIMEOUT_MS)
            .get(NM_DNS_MANAGER, RC_MANAGEMENT_MODE_KEY)
            .map_err(Error::TooOldNetworkManager)?;
        if management_mode == "unmanaged" {
            return Err(Error::NetworkManagerNotManagingDns);
        }

        let dns_mode: String = self
            .dbus_connection
            .with_path(NM_BUS, NM_DNS_MANAGER_PATH, RPC_TIMEOUT_MS)
            .get(NM_DNS_MANAGER, DNS_MODE_KEY)
            .map_err(Error::Dbus)?;

        match dns_mode.as_ref() {
            // Managed by systemd-resolved
            "systemd-resolved" => return Err(Error::SystemdResolved),
            // If NetworkManager isn't managing DNS for us, it's useless.
            "none" => return Err(Error::NetworkManagerNotManagingDns),
            _ => (),
        };


        let expected_resolv_conf = "/var/run/NetworkManager/resolv.conf";
        let actual_resolv_conf = "/etc/resolv.conf";
        if !eq_file_content(&expected_resolv_conf, &actual_resolv_conf) {
            log::debug!("/etc/resolv.conf differs from reference resolv.conf, therefore NM is not managing DNS");
            return Err(Error::NetworkManagerNotManagingDns);
        }

        Ok(())
    }

    fn as_manager(&self) -> dbus::ConnPath<'_, &dbus::Connection> {
        self.dbus_connection
            .with_path(NM_BUS, NM_OBJECT_PATH, RPC_TIMEOUT_MS)
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let connection = self.fetch_connection(interface_name)?;

        // Obtain settings for this connection
        let get_settings =
            Message::new_method_call(NM_BUS, &connection, NM_SETTINGS_CONNECTION, "GetSettings")
                .map_err(Error::DbusMethodCall)?;

        let results = self
            .dbus_connection
            .send_with_reply_and_block(get_settings, RPC_TIMEOUT_MS)
            .map_err(Error::Dbus)?;
        let mut settings: HashMap<&str, HashMap<&str, Variant<Box<dyn RefArg>>>> =
            results.read1().map_err(Error::MatchDBusTypeError)?;

        // Update the DNS config for this link

        let v4_dns: Vec<u32> = servers
            .iter()
            .filter_map(|server| {
                match server {
                    // Network-byte order
                    IpAddr::V4(server) => Some(u32::to_be(server.clone().into())),
                    IpAddr::V6(_) => None,
                }
            })
            .collect();
        if !v4_dns.is_empty() {
            Self::update_dns_config(&mut settings, "ipv4", v4_dns);
        }

        let v6_dns: Vec<Vec<u8>> = servers
            .iter()
            .filter_map(|server| match server {
                IpAddr::V4(_) => None,
                IpAddr::V6(server) => Some(server.octets().to_vec()),
            })
            .collect();
        if !v6_dns.is_empty() {
            Self::update_dns_config(&mut settings, "ipv6", v6_dns);
        }

        // Apply changes

        let update =
            Message::new_method_call(NM_BUS, &connection, NM_SETTINGS_CONNECTION, "Update")
                .map_err(Error::DbusMethodCall)?
                .append1(settings);

        self.dbus_connection
            .send_with_reply_and_block(update, RPC_TIMEOUT_MS)
            .map_err(Error::Dbus)?;

        // Re-activate the connection to update /etc/resolv.conf

        let activate =
            Message::new_method_call(NM_BUS, NM_OBJECT_PATH, NM_TOP_OBJECT, "ActivateConnection")
                .map_err(Error::DbusMethodCall)?
                .append_ref(&[
                    connection,
                    dbus::Path::new("/").unwrap(),
                    dbus::Path::new("/").unwrap(),
                ]);
        self.dbus_connection
            .send_with_reply_and_block(activate, RPC_TIMEOUT_MS)
            .map_err(Error::Dbus)?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn update_dns_config<'a, T>(
        settings: &mut HashMap<&str, HashMap<&str, Variant<Box<dyn RefArg + 'a>>>>,
        ip_protocol: &'static str,
        servers: T,
    ) where
        T: RefArg + 'a,
    {
        let settings = match settings.get_mut(ip_protocol) {
            Some(ip_protocol) => ip_protocol,
            None => {
                settings.insert(ip_protocol, HashMap::new());
                settings.get_mut(ip_protocol).unwrap()
            }
        };

        settings.insert("method", Variant(Box::new("manual".to_string())));
        settings.insert("ignore-auto-dns", Variant(Box::new(true)));
        settings.insert("dns-priority", Variant(Box::new(DNS_FIRST_PRIORITY)));
        settings.insert("dns", Variant(Box::new(servers)));
    }

    fn fetch_connection(&self, interface_name: &str) -> Result<dbus::Path<'static>> {
        let devices: Box<dyn RefArg> = self
            .as_manager()
            .get(NM_TOP_OBJECT, "Devices")
            .map_err(Error::Dbus)?;
        let mut iter = devices.as_iter().ok_or(Error::ObtainDevices)?;

        while let Some(key) = iter.next() {
            let key = key.as_str().ok_or(Error::ObtainDevices)?;

            let device: String = self
                .dbus_connection
                .with_path(NM_BUS, key, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "Interface")
                .map_err(Error::Dbus)?;

            if device != interface_name {
                continue;
            }

            let active_connection: dbus::Path<'_> = self
                .dbus_connection
                .with_path(NM_BUS, key, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "ActiveConnection")
                .map_err(Error::Dbus)?;

            return self
                .dbus_connection
                .with_path(NM_BUS, active_connection, RPC_TIMEOUT_MS)
                .get(NM_CONNECTION_ACTIVE, "Connection")
                .map_err(Error::Dbus);
        }
        Err(Error::LinkNotFound)
    }
}

fn eq_file_content<P: AsRef<Path>>(a: &P, b: &P) -> bool {
    let file_a = match File::open(a).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed top open file {}: {}", a.as_ref().display(), e);
            return false;
        }
    };
    let file_b = match File::open(b).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed top open file {}: {}", b.as_ref().display(), e);
            return false;
        }
    };

    file_a
        .lines()
        .zip(file_b.lines())
        .all(|(a, b)| match (a, b) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        })
}
