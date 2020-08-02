use dbus::{
    arg::{RefArg, Variant},
    stdintf::*,
    BusType, Member, Message,
};
use lazy_static::lazy_static;
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

    #[error(display = "Failed to construct DBus member")]
    DbusMemberConstruct(String),

    #[error(display = "Failed to match the returned D-Bus object with expected type")]
    MatchDBusTypeError(#[error(source)] dbus::arg::TypeMismatchError),

    #[error(display = "DNS is managed by systemd-resolved - NM can't enforce DNS globally")]
    SystemdResolved,

    #[error(display = "Failed to find obtain devices from network manager")]
    ObtainDevices,

    #[error(display = "Failed to find link interface in network manager")]
    LinkNotFound,

    #[error(display = "Device inactive: {}", _0)]
    DeviceNotReady(u32),
}

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_TOP_OBJECT: &str = "org.freedesktop.NetworkManager";
const NM_DNS_MANAGER: &str = "org.freedesktop.NetworkManager.DnsManager";
const NM_DNS_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";
const NM_OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";
const NM_DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const NM_IP4_CONFIG: &str = "org.freedesktop.NetworkManager.IP4Config";
const NM_IP6_CONFIG: &str = "org.freedesktop.NetworkManager.IP6Config";
const RPC_TIMEOUT_MS: i32 = 3000;
const GLOBAL_DNS_CONF_KEY: &str = "GlobalDnsConfiguration";
const RC_MANAGEMENT_MODE_KEY: &str = "RcManager";
const DNS_MODE_KEY: &str = "Mode";
const DNS_FIRST_PRIORITY: i32 = -2147483647;

const NM_DEVICE_STATE_ACTIVATED: u32 = 100;

lazy_static! {
    static ref NM_DEVICE_STATE_CHANGED: Member<'static> = Member::new("StateChanged").unwrap();
}

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
        let device = self.fetch_device(interface_name)?;

        // Get the last applied connection

        let get_applied_connection =
            Message::new_method_call(NM_BUS, &device, NM_DEVICE, "GetAppliedConnection")
                .map_err(Error::DbusMethodCall)?
                .append1(0u32);
        let applied_connection = self
            .dbus_connection
            .send_with_reply_and_block(get_applied_connection, RPC_TIMEOUT_MS)
            .map_err(Error::Dbus)?;

        let (mut settings, version_id): (
            HashMap<&str, HashMap<&str, Variant<Box<dyn RefArg>>>>,
            u64,
        ) = applied_connection
            .read2()
            .map_err(Error::MatchDBusTypeError)?;

        // Update the DNS config

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

        // Keep changed routes

        if let Some(ipv4_settings) = settings.get_mut("ipv4") {
            let device_ip4_config: dbus::Path<'_> = self
                .dbus_connection
                .with_path(NM_BUS, &device, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "Ip4Config")
                .map_err(Error::Dbus)?;

            let device_routes: Vec<Vec<u32>> = self
                .dbus_connection
                .with_path(NM_BUS, &device_ip4_config, RPC_TIMEOUT_MS)
                .get(NM_IP4_CONFIG, "Routes")
                .map_err(Error::Dbus)?;

            let device_route_data: Vec<HashMap<String, Variant<Box<dyn RefArg>>>> = self
                .dbus_connection
                .with_path(NM_BUS, &device_ip4_config, RPC_TIMEOUT_MS)
                .get(NM_IP4_CONFIG, "RouteData")
                .map_err(Error::Dbus)?;

            ipv4_settings.insert("route-metric", Variant(Box::new(0u32)));
            ipv4_settings.insert("routes", Variant(Box::new(device_routes)));
            ipv4_settings.insert("route-data", Variant(Box::new(device_route_data)));
        }

        if let Some(ipv6_settings) = settings.get_mut("ipv6") {
            let device_ip6_config: dbus::Path<'_> = self
                .dbus_connection
                .with_path(NM_BUS, &device, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "Ip6Config")
                .map_err(Error::Dbus)?;

            let device_routes6: Vec<(Vec<u8>, u32, Vec<u8>, u32)> = self
                .dbus_connection
                .with_path(NM_BUS, &device_ip6_config, RPC_TIMEOUT_MS)
                .get(NM_IP6_CONFIG, "Routes")
                .map_err(Error::Dbus)?;

            let device_route6_data: Vec<HashMap<String, Variant<Box<dyn RefArg>>>> = self
                .dbus_connection
                .with_path(NM_BUS, &device_ip6_config, RPC_TIMEOUT_MS)
                .get(NM_IP6_CONFIG, "RouteData")
                .map_err(Error::Dbus)?;

            ipv6_settings.insert("route-metric", Variant(Box::new(0u32)));
            ipv6_settings.insert("routes", Variant(Box::new(device_routes6)));
            ipv6_settings.insert("route-data", Variant(Box::new(device_route6_data)));
        }

        // Re-apply changes

        let reapply = Message::new_method_call(NM_BUS, &device, NM_DEVICE, "Reapply")
            .map_err(Error::DbusMethodCall)?
            .append3(settings, version_id, 0u32);
        self.dbus_connection
            .send_with_reply_and_block(reapply, RPC_TIMEOUT_MS)
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
        settings.insert("dns-priority", Variant(Box::new(DNS_FIRST_PRIORITY)));
        settings.insert("dns", Variant(Box::new(servers)));
    }

    fn fetch_device(&self, interface_name: &str) -> Result<dbus::Path<'_>> {
        let devices: Box<dyn RefArg> = self
            .as_manager()
            .get(NM_TOP_OBJECT, "Devices")
            .map_err(Error::Dbus)?;
        let mut iter = devices.as_iter().ok_or(Error::ObtainDevices)?;

        while let Some(device) = iter.next() {
            // Copy due to lifetime weirdness
            let device = device.box_clone();
            let device = device
                .as_any()
                .downcast_ref::<dbus::Path<'_>>()
                .ok_or(Error::ObtainDevices)?;

            let device_name: String = self
                .dbus_connection
                .with_path(NM_BUS, device, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "Interface")
                .map_err(Error::Dbus)?;

            if device_name != interface_name {
                continue;
            }

            let state: u32 = self
                .dbus_connection
                .with_path(NM_BUS, device, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "State")
                .map_err(Error::Dbus)?;

            if state != NM_DEVICE_STATE_ACTIVATED {
                let mut current_state = state;

                let match_rule = &format!(
                    "destination='{}',path='{}',interface='{}',member='{}'",
                    NM_BUS,
                    device,
                    NM_DEVICE,
                    NM_DEVICE_STATE_CHANGED.to_string()
                );
                self.dbus_connection
                    .add_match(match_rule)
                    .map_err(Error::Dbus)?;

                for message in self.dbus_connection.incoming(RPC_TIMEOUT_MS as u32) {
                    if message.member().as_ref() != Some(&NM_DEVICE_STATE_CHANGED) {
                        continue;
                    }
                    let (new_state, _old_state, _reason): (u32, u32, u32) = message
                        .read3()
                        .map_err(Error::MatchDBusTypeError)
                        .map_err(|error| {
                            let _ = self.dbus_connection.remove_match(match_rule);
                            error
                        })?;

                    current_state = new_state;
                    log::trace!("New tunnel device state: {}", current_state);
                    if current_state == NM_DEVICE_STATE_ACTIVATED {
                        break;
                    }
                }

                if let Err(error) = self.dbus_connection.remove_match(match_rule) {
                    log::warn!("Failed to remove signal listener: {}", error);
                }

                if current_state != NM_DEVICE_STATE_ACTIVATED {
                    return Err(Error::DeviceNotReady(state));
                }
            }

            return Ok(device.clone());
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
