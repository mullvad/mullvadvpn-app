use dbus::{
    arg::{Append, RefArg, Variant},
    ffidisp::{stdintf::*, BusType, ConnPath, Connection},
    message::Message,
    strings::Member,
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::IpAddr,
    path::Path,
    time::{Duration, Instant},
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
const DEVICE_READY_TIMEOUT: Duration = Duration::from_secs(15);
const GLOBAL_DNS_CONF_KEY: &str = "GlobalDnsConfiguration";
const RC_MANAGEMENT_MODE_KEY: &str = "RcManager";
const DNS_MODE_KEY: &str = "Mode";
const DNS_FIRST_PRIORITY: i32 = -2147483647;

const NM_DEVICE_STATE_IP_CHECK: u32 = 80;
const NM_DEVICE_STATE_SECONDARY: u32 = 90;
const NM_DEVICE_STATE_ACTIVATED: u32 = 100;

lazy_static! {
    static ref NM_DEVICE_STATE_CHANGED: Member<'static> = Member::new("StateChanged").unwrap();
}

pub struct NetworkManager {
    dbus_connection: Connection,
    device: Option<dbus::Path<'static>>,
    settings_backup: Option<HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>>,
}


impl NetworkManager {
    pub fn new() -> Result<Self> {
        let dbus_connection = Connection::get_private(BusType::System).map_err(Error::Dbus)?;
        let manager = NetworkManager {
            dbus_connection,
            device: None,
            settings_backup: None,
        };
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

    fn as_manager(&self) -> ConnPath<'_, &Connection> {
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

        // Keep changed routes.
        // These routes were modified outside NM, likely by RouteManager.

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

            let device_addresses6: Vec<(Vec<u8>, u32, Vec<u8>)> = self
                .dbus_connection
                .with_path(NM_BUS, &device_ip6_config, RPC_TIMEOUT_MS)
                .get(NM_IP6_CONFIG, "Addresses")
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
            // if the link contains link local addresses, addresses shouldn't be reset
            if ipv6_settings
                .get("method")
                .map(|method| {
                    // if IPv6 isn't enabled, IPv6 method will be set to "ignore", in which case we
                    // shouldn't reapply any config for ipv6
                    method.as_str() != Some("link-local") && method.as_str() != Some("ignore")
                })
                .unwrap_or(true)
            {
                ipv6_settings.insert("addresses", Variant(Box::new(device_addresses6)));
            }
        }

        let mut settings_backup =
            HashMap::<String, HashMap<String, Variant<Box<dyn RefArg>>>>::new();
        for (top_key, map) in settings.iter() {
            let mut inner_dict = HashMap::<String, Variant<Box<dyn RefArg>>>::new();
            for (key, variant) in map.iter() {
                inner_dict.insert(key.to_string(), Variant(variant.0.box_clone()));
            }
            settings_backup.insert(top_key.to_string(), inner_dict);
        }

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

        if let Some(wg_config) = settings.get_mut("wireguard") {
            wg_config.insert(
                "fwmark",
                Variant(Box::new(crate::linux::TUNNEL_FW_MARK) as Box<dyn RefArg>),
            );
        }

        self.reapply_settings(&device, settings, version_id)?;

        self.device = Some(device);
        self.settings_backup = Some(settings_backup);

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(settings_backup) = self.settings_backup.take() {
            let device = self.device.take().ok_or(Error::LinkNotFound)?;
            self.reapply_settings(&device, settings_backup, 0u64)?;
        } else {
            log::trace!("No DNS settings to reset");
        }
        Ok(())
    }

    fn reapply_settings<Settings: Append>(
        &self,
        device: &dbus::Path<'_>,
        settings: Settings,
        version_id: u64,
    ) -> Result<()> {
        let reapply = Message::new_method_call(NM_BUS, device, NM_DEVICE, "Reapply")
            .map_err(Error::DbusMethodCall)?
            .append3(settings, version_id, 0u32);
        self.dbus_connection
            .send_with_reply_and_block(reapply, RPC_TIMEOUT_MS)
            .map_err(Error::Dbus)?;
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

    fn fetch_device(&self, interface_name: &str) -> Result<dbus::Path<'static>> {
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

            let mut device_state: u32 = self
                .dbus_connection
                .with_path(NM_BUS, device, RPC_TIMEOUT_MS)
                .get(NM_DEVICE, "State")
                .map_err(Error::Dbus)?;

            if !device_is_ready(device_state) {
                let deadline = Instant::now() + DEVICE_READY_TIMEOUT;
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

                // a separate loopis used here because `connection.incoming(TIMEOUT)` will sleep
                // for TIMEOUT after the last message was received - if the device is thrashing
                // between states, we should probably give up rather than block indefinitely.
                while Instant::now() < deadline && !device_is_ready(device_state) {
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

                        device_state = new_state;
                        log::trace!("New tunnel device state: {}", device_state);
                        if device_is_ready(device_state) {
                            break;
                        }
                    }
                }

                if let Err(error) = self.dbus_connection.remove_match(match_rule) {
                    log::warn!("Failed to remove signal listener: {}", error);
                }
                if !device_is_ready(device_state) {
                    return Err(Error::DeviceNotReady(device_state));
                }
            }

            return Ok(device.clone());
        }
        Err(Error::LinkNotFound)
    }
}

fn device_is_ready(device_state: u32) -> bool {
    /// Any state above `NM_DEVICE_STATE_IP_CONFIG` is considered to be an OK state to change the
    /// DNS config. For the enums, see https://developer.gnome.org/NetworkManager/stable/nm-dbus-types.html#NMDeviceState
    const READY_STATES: [u32; 3] = [
        NM_DEVICE_STATE_IP_CHECK,
        NM_DEVICE_STATE_SECONDARY,
        NM_DEVICE_STATE_ACTIVATED,
    ];
    READY_STATES.contains(&device_state)
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
