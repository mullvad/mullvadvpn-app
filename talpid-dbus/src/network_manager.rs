//! NetworkManager is the one-stop-shop of network configuration on Linux.
use std::fs;
use std::net::IpAddr;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};

use futures::StreamExt;
use tokio::time::timeout;
use zbus::{Array, Connection, ObjectPath, OwnedObjectPath, OwnedVariant, Str, Variant, proxy};

use super::systemd_resolved;
use crate::get_connection_zbus;

const DNS_FIRST_PRIORITY: i32 = -2147483647;

const NM_DEVICE_STATE_IP_CHECK: u32 = 80;
const NM_DEVICE_STATE_SECONDARY: u32 = 90;
const NM_DEVICE_STATE_ACTIVATED: u32 = 100;

const NM_ADD_CONNECTION_VOLATILE: u32 = 0x2;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while communicating over D-Bus")]
    Dbus(#[from] zbus::Error),

    #[error(
        "NM is configured to manage DNS via systemd-resolved but systemd-resolved is not managing /etc/resolv.conf: {0}"
    )]
    SystemdResolvedNotManagingResolvconf(systemd_resolved::Error),

    #[error("Configuration has no device associated to it")]
    NoDevice,

    #[error("Device inactive: {0}")]
    DeviceNotReady(u32),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("NetworkManager not detected")]
    NetworkManagerNotDetected,

    #[error("NetworkManager is using dnsmasq to manage DNS")]
    UsingDnsmasq,

    #[error("NetworkManager is not managing DNS")]
    NetworkManagerNotManagingDns,

    #[error("Failed to get devices from NetworkManager object")]
    ObtainDevices,
}

/// settings are a{sa{sv}}
pub type DeviceConfig = HashMap<String, HashMap<String, zbus::OwnedVariant>>;

/// Implements functionality to control NetworkManager over DBus.
pub struct NetworkManagerConnection {
    connection: Connection,
}

impl NetworkManagerConnection {
    pub async fn new() -> Result<Self, Error> {
        let connection = get_connection_zbus().await?;
        Ok(Self { connection })
    }

    pub async fn create_network_device(&self, config: DeviceConfig) -> Result<NMDevice, Error> {
        let device = self.create_network_device_inner(config).await?;
        if let Err(err) = self
            .wait_until_device_is_ready(device.device_path.clone())
            .await
        {
            if let Err(removal_error) = self.remove_network_device(device).await {
                log::error!(
                    "Failed to remove network device after it not becoming ready fast enough: {}",
                    removal_error
                );
            }
            return Err(err);
        }

        Ok(device)
    }

    pub async fn get_interface_name(&self, network_device: &NMDevice) -> Result<String, Error> {
        let interface =
            DeviceProxy::new_from_path(network_device.device_path.clone(), &self.connection)
                .await?
                .interface()
                .await?;
        Ok(interface)
    }

    pub async fn reapply(
        &self,
        device: OwnedObjectPath,
        settings: HashMap<String, HashMap<String, OwnedVariant>>,
    ) -> Result<(), Error> {
        let version_id = 0;
        DeviceProxy::new_from_path(device, &self.connection)
            .await?
            .reapply(settings, version_id, 0u32)
            .await?;
        Ok(())
    }

    /// The current state of the device.
    ///
    /// <https://networkmanager.pages.freedesktop.org/NetworkManager/NetworkManager/gdbus-org.freedesktop.NetworkManager.Device.html#gdbus-property-org-freedesktop-NetworkManager-Device.State>
    pub async fn get_device_state(&self, device: zbus::OwnedObjectPath) -> Result<u32, Error> {
        let state = DeviceProxy::new_from_path(device, &self.connection)
            .await?
            .state()
            .await?;
        Ok(state)
    }

    async fn create_network_device_inner(&self, config: DeviceConfig) -> Result<NMDevice, Error> {
        let settings = SettingsProxy::new(&self.connection).await?;
        let config_path = match settings
            .add_connection2(config, NM_ADD_CONNECTION_VOLATILE, Default::default())
            .await
        {
            Ok((path, _result)) => path,
            // Err(zbus::Error::XXX) => todo!("Handle DBUS_UNKNOWN_METHOD case"),
            Err(err) => {
                log::error!(
                    "Failed to create a new interface via AddConnection2: {}",
                    err
                );
                return Err(err.into());
            }
        };

        let nm = NetworkManagerProxy::new(&self.connection).await?;
        let connection_path = nm
            .activate_connection(
                &config_path.as_ref(),
                &ObjectPath::from_str_unchecked("/"),
                &ObjectPath::from_str_unchecked("/"),
            )
            .await?;

        let device_paths = nm.devices().await?;
        let device_path = device_paths.into_iter().next().ok_or(Error::NoDevice)?;

        Ok(NMDevice {
            config_path,
            connection_path,
            device_path,
        })
    }

    pub async fn version_string(&self) -> Result<String, Error> {
        let version = NetworkManagerProxy::new(&self.connection)
            .await?
            .version()
            .await?;
        Ok(version)
    }

    async fn wait_until_device_is_ready(&self, device: zbus::OwnedObjectPath) -> Result<(), Error> {
        let device_state = self.get_device_state(device.clone()).await?;
        if device_is_ready(device_state) {
            return Ok(());
        }

        let device = DeviceProxy::new_from_path(device, &self.connection).await?;
        let mut state_changes = device.receive_device_state_changed().await?;

        const DEVICE_READY_TIMEOUT: Duration = Duration::from_secs(15);
        let deadline = Instant::now() + DEVICE_READY_TIMEOUT;

        while let Some(state_update) = timeout(DEVICE_READY_TIMEOUT, state_changes.next())
            .await
            .map_err(|_| Error::DeviceNotReady(device_state))?
        {
            let state = state_update.args()?;
            log::debug!("Received new tunnel state change: {:?}", state);
            let new_state = state.new_state;
            if device_is_ready(new_state) {
                return Ok(());
            } else if Instant::now() >= deadline {
                return Err(Error::DeviceNotReady(new_state));
            }
        }

        Ok(())
    }

    pub async fn remove_network_device(&self, device: NMDevice) -> Result<(), Error> {
        let nm = NetworkManagerProxy::new(&self.connection).await?;
        let deactivation_result = nm.deactivate_connection(&device.connection_path).await;

        let device = DeviceProxy::new_from_path(device.config_path, &self.connection).await?;
        let config_result = device.delete().await;

        deactivation_result?;
        config_result?;
        Ok(())
    }

    /// Ensures NetworkManager's connectivity check is disabled and returns the connectivity check
    /// previous state. Returns true only if the connectivity check was enabled and is now
    /// disabled. Disabling the connectivity check should be done before a firewall is applied
    /// due to the fact that blocking DNS requests can make it hang:
    /// <https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/404>
    pub async fn disable_connectivity_check(&self) -> Option<bool> {
        let nm = NetworkManagerProxy::new(&self.connection).await.ok()?;
        match nm.connectivity_check_enabled().await {
            Ok(true) => {
                if let Err(err) = nm.set_connectivity_check_enabled(false).await {
                    log::error!(
                        "Failed to disable NetworkManager connectivity check: {}",
                        err
                    );
                    Some(false)
                } else {
                    Some(true)
                }
            }
            Ok(false) => Some(false),
            Err(_) => None,
        }
    }

    /// Enabled NetworkManager's connectivity check. Fails silently.
    pub async fn enable_connectivity_check(&self) -> zbus::Result<()> {
        let nm = NetworkManagerProxy::new(&self.connection).await?;
        let result = nm.set_connectivity_check_enabled(true).await;
        if let Err(err) = result.as_ref() {
            log::error!("Failed to reset NetworkManager connectivity check: {}", err);
        }
        result
    }

    pub async fn ensure_network_manager_exists(&self) -> Result<(), Error> {
        let nm = NetworkManagerProxy::new(&self.connection).await?;
        if let Err(err) = nm.version().await {
            log::error!("Failed to read version of NetworkManager {}", err);
            return Err(Error::NetworkManagerNotDetected);
        }
        Ok(())
    }

    pub async fn ensure_can_be_used_to_manage_dns(&self) -> Result<(), Error> {
        self.ensure_resolv_conf_is_managed().await?;
        self.ensure_network_manager_exists().await?;
        Ok(())
    }

    pub async fn ensure_resolv_conf_is_managed(&self) -> Result<(), Error> {
        let dns_manager = DnsManagerProxy::new(&self.connection).await?;
        // check if NM is set to manage resolv.conf
        let management_mode = dns_manager.rc_manager().await?;

        if management_mode == "unmanaged" {
            return Err(Error::NetworkManagerNotManagingDns);
        }

        if management_mode == "systemd-resolved" {
            return match systemd_resolved::SystemdResolved::new().await {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::SystemdResolvedNotManagingResolvconf(err)),
            };
        }

        match dns_manager.mode().await?.as_ref() {
            // NM can't setup config for multiple interfaces with dnsmasq
            "dnsmasq" => return Err(Error::UsingDnsmasq),
            // If NetworkManager isn't managing DNS for us, it's useless.
            "none" => return Err(Error::NetworkManagerNotManagingDns),
            _ => (),
        };

        if !verify_etc_resolv_conf_contents() {
            log::debug!(
                "/etc/resolv.conf differs from reference resolv.conf, therefore NM is not managing DNS"
            );
            return Err(Error::NetworkManagerNotManagingDns);
        }

        Ok(())
    }

    pub async fn set_dns(
        &mut self,
        interface_name: &str,
        servers: &[IpAddr],
    ) -> Result<DeviceConfig, Error> {
        let device_path = self.fetch_device(interface_name).await?;
        self.wait_until_device_is_ready(device_path.clone()).await?;
        let device = DeviceProxy::new_from_path(device_path.clone(), &self.connection).await?;

        // Get the last applied connection
        let (mut settings, version_id) = device.get_applied_connection(0u32).await?;

        // Keep changed routes.
        // These routes were modified outside NM, likely by RouteManager.
        if let Some(ipv4_settings) = settings.get_mut("ipv4") {
            let device_ip4_config = device.ip4_config().await?;
            let ipv4_config =
                IP4ConfigProxy::new_from_path(device_ip4_config, &self.connection).await?;

            let device_routes = ipv4_config.routes().await?;
            let device_route_data = ipv4_config.route_data().await?;

            ipv4_settings.insert("route-metric".to_string(), OwnedVariant::from(0u32));
            ipv4_settings.insert("routes".to_string(), {
                OwnedVariant::try_from(Array::from(device_routes)).unwrap()
            });
            ipv4_settings.insert("route-data".to_string(), {
                OwnedVariant::try_from(Array::from(device_route_data)).unwrap()
            });
        }

        if let Some(ipv6_settings) = settings.get_mut("ipv6") {
            let device_ip6_config = device.ip6_config().await?;
            let ipv6_config =
                IP6ConfigProxy::new_from_path(device_ip6_config, &self.connection).await?;

            let device_addresses6 = ipv6_config.addresses().await?;
            let device_routes6 = ipv6_config.routes().await?;
            let device_route6_data = ipv6_config.route_data().await?;

            ipv6_settings.insert("route-metric".to_string(), OwnedVariant::from(0u32));
            ipv6_settings.insert(
                "routes".to_string(),
                OwnedVariant::try_from(Array::from(device_routes6)).unwrap(),
            );
            ipv6_settings.insert(
                "route-data".to_string(),
                OwnedVariant::try_from(Array::from(device_route6_data)).unwrap(),
            );
            // if the link contains link local addresses, addresses shouldn't be reset
            if ipv6_settings
                .get("method")
                .map(|method| {
                    // if IPv6 isn't enabled, IPv6 method will be set to "ignore", in which case we
                    // shouldn't reapply any config for ipv6
                    let method = method.try_into();
                    method != Ok("link-local") && method != Ok("ignore")
                })
                .unwrap_or(true)
            {
                ipv6_settings.insert(
                    "addresses".to_string(),
                    OwnedVariant::try_from(Array::from(device_addresses6)).unwrap(),
                );
            }
        }

        let mut settings_backup = HashMap::new();
        for (top_key, map) in settings.iter() {
            let mut inner_dict = HashMap::new();
            for (key, variant) in map.iter() {
                inner_dict.insert(key.clone(), variant.clone());
            }
            settings_backup.insert(top_key.clone(), inner_dict);
        }

        // Update the DNS config
        let v4_dns: Vec<u32> = servers
            .iter()
            .filter_map(|server| {
                match server {
                    // Network-byte order
                    IpAddr::V4(server) => Some(u32::to_be((*server).into())),
                    IpAddr::V6(_) => None,
                }
            })
            .collect();
        if !v4_dns.is_empty() {
            let ip_protocol = "ipv4";
            let settings = match settings.get_mut(ip_protocol) {
                Some(ip_protocol) => ip_protocol,
                None => {
                    settings.insert(ip_protocol.to_string(), HashMap::new());
                    settings.get_mut(ip_protocol).unwrap()
                }
            };
            settings.insert(
                "method".to_string(),
                Str::from_static("manual").into(),
                //OwnedVariant::try_from("manual".to_string()).unwrap(),
            );
            settings.insert(
                "dns-priority".to_string(),
                OwnedVariant::from(DNS_FIRST_PRIORITY),
            );
            settings.insert(
                "dns".to_string(),
                OwnedVariant::try_from(Array::from(v4_dns)).unwrap(),
            );
            settings.insert(
                "dns-search".to_string(),
                OwnedVariant::try_from(Array::from(vec![Str::from_static("~.")])).unwrap(),
            );
        }

        let v6_dns: Vec<Vec<u8>> = servers
            .iter()
            .filter_map(|server| match server {
                IpAddr::V4(_) => None,
                IpAddr::V6(server) => Some(server.octets().to_vec()),
            })
            .collect();
        if !v6_dns.is_empty() {
            let ip_protocol = "ipv6";
            let settings = match settings.get_mut(ip_protocol) {
                Some(ip_protocol) => ip_protocol,
                None => {
                    settings.insert(ip_protocol.to_string(), HashMap::new());
                    settings.get_mut(ip_protocol).unwrap()
                }
            };
            settings.insert("method".to_string(), Str::from_static("manual").into());
            settings.insert(
                "dns-priority".to_string(),
                OwnedVariant::from(DNS_FIRST_PRIORITY),
            );
            settings.insert(
                "dns".to_string(),
                OwnedVariant::try_from(Array::from(v6_dns)).unwrap(),
            );
            settings.insert(
                "dns-search".to_string(),
                OwnedVariant::try_from(Array::from(vec![Str::from_static("~.")])).unwrap(),
            );
        }

        let device = DeviceProxy::new_from_path(device_path, &self.connection).await?;
        device.reapply(settings, version_id, 0u32).await?;
        Ok(settings_backup)
    }

    pub async fn fetch_device(&self, interface_name: &str) -> Result<OwnedObjectPath, Error> {
        let nm = NetworkManagerProxy::new(&self.connection).await?;
        let devices = nm.devices().await?;
        if devices.is_empty() {
            return Err(Error::ObtainDevices);
        }
        for device_path in devices {
            let device = DeviceProxy::new_from_path(device_path.clone(), &self.connection).await?;
            let device_name = device.interface().await?;

            if device_name != interface_name {
                continue;
            }

            return Ok(device_path);
        }
        Err(Error::DeviceNotFound)
    }

    pub fn convert_address_to_dbus(address: &IpAddr) -> HashMap<String, zbus::OwnedVariant> {
        let mut map = HashMap::new();
        map.insert(
            "address".to_string(),
            OwnedVariant::from(Str::from(address.to_string())),
        );
        let prefix: u32 = if address.is_ipv4() { 32 } else { 128 };
        map.insert("prefix".to_string(), OwnedVariant::from(prefix));
        map
    }
}

#[derive(Debug)]
pub struct NMDevice {
    config_path: OwnedObjectPath,
    connection_path: OwnedObjectPath,
    device_path: OwnedObjectPath,
}

/// Any state above `NM_DEVICE_STATE_IP_CONFIG` is considered to be an OK state to change the
/// DNS config. For the enums, see <https://developer.gnome.org/NetworkManager/stable/nm-dbus-types.html#NMDeviceState>
pub fn device_is_ready(device_state: u32) -> bool {
    const READY_STATES: [u32; 3] = [
        NM_DEVICE_STATE_IP_CHECK,
        NM_DEVICE_STATE_SECONDARY,
        NM_DEVICE_STATE_ACTIVATED,
    ];
    READY_STATES.contains(&device_state)
}

// Verify that the contents of /etc/resolv.conf match what NM expects them to be.
fn verify_etc_resolv_conf_contents() -> bool {
    let parse_resolv_conf_file = |path| {
        let buf = fs::read(path).ok()?;
        resolv_conf::Config::parse(&buf).ok()
    };

    let Some(actual_resolv_conf) = parse_resolv_conf_file("/etc/resolv.conf") else {
        return false;
    };
    let Some(expected_resolv_conf) = parse_resolv_conf_file("/var/run/NetworkManager/resolv.conf")
    else {
        return false;
    };

    actual_resolv_conf == expected_resolv_conf
}

#[proxy(interface = "org.freedesktop.NetworkManager", assume_defaults = true)]
pub trait NetworkManager {
    fn activate_connection(
        &self,
        connection: &zbus::ObjectPath<'_>,
        device: &zbus::ObjectPath<'_>,
        specific_object: &zbus::ObjectPath<'_>,
    ) -> zbus::Result<zbus::OwnedObjectPath>;

    /// DeactivateConnection method
    fn deactivate_connection(&self, active_connection: &zbus::ObjectPath<'_>) -> zbus::Result<()>;

    /// Devices property
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::OwnedObjectPath>>;

    /// ConnectivityCheckEnabled property
    #[zbus(property)]
    fn connectivity_check_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_connectivity_check_enabled(&self, value: bool) -> zbus::Result<()>;

    /// Version property
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Settings",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Settings",
    assume_defaults = true
)]
pub trait Settings {
    /// AddConnection2 method
    fn add_connection2(
        &self,
        //settings: HashMap<&str, HashMap<&str, Variant<'_>>>,
        settings: HashMap<String, HashMap<String, OwnedVariant>>,
        flags: u32,
        args: std::collections::HashMap<&str, Variant<'_>>,
    ) -> zbus::Result<(OwnedObjectPath, HashMap<String, OwnedVariant>)>;

    /// AddConnectionUnsaved method
    fn add_connection_unsaved(
        &self,
        //connection: HashMap<&str, HashMap<&str, Variant<'_>>>,
        connection: HashMap<&str, HashMap<&str, Variant<'_>>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

// const NM_DNS_MANAGER: &str = "org.freedesktop.NetworkManager.DnsManager";
// const NM_DNS_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";
#[proxy(
    default_path = "/org/freedesktop/NetworkManager/DnsManager",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.DnsManager",
    assume_defaults = true
)]
pub trait DnsManager {
    /// Configuration property
    #[zbus(property)]
    fn configuration(&self) -> zbus::Result<Vec<HashMap<String, OwnedVariant>>>;

    /// Mode property
    #[zbus(property)]
    fn mode(&self) -> zbus::Result<String>;

    /// RcManager property
    #[zbus(property)]
    fn rc_manager(&self) -> zbus::Result<String>;
}

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/Devices",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device",
    assume_defaults = true
)]
pub trait Device {
    /// Delete method
    fn delete(&self) -> zbus::Result<()>;

    /// GetAppliedConnection method
    fn get_applied_connection(
        &self,
        flags: u32,
    ) -> zbus::Result<(HashMap<String, HashMap<String, zbus::OwnedVariant>>, u64)>;

    /// Attempts to update the configuration of a device without deactivating it.
    ///
    /// <https://networkmanager.pages.freedesktop.org/NetworkManager/NetworkManager/gdbus-org.freedesktop.NetworkManager.Device.html#gdbus-method-org-freedesktop-NetworkManager-Device.Reapply>
    fn reapply(
        &self,
        //connection: HashMap<&str, HashMap<&str, zbus::Variant<'_>>>,
        connection: HashMap<String, HashMap<String, zbus::OwnedVariant>>,
        version_id: u64,
        flags: u32,
    ) -> zbus::Result<()>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::OwnedObjectPath>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::OwnedObjectPath>;

    /// Interface property
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateChanged signal
    #[zbus(signal, name = "StateChanged")]
    fn device_state_changed(&self, new_state: u32, old_state: u32, reason: u32)
    -> zbus::Result<()>;
}

impl DeviceProxy<'_> {
    pub async fn new_from_path(
        device_path: zbus::OwnedObjectPath,
        connection: &Connection,
    ) -> zbus::Result<DeviceProxy<'_>> {
        DeviceProxy::builder(connection)
            .path(device_path)
            .build()
            .await
    }
}

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/IP4Config",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.IP4Config",
    assume_defaults = true
)]
pub trait IP4Config {
    /// Routes property
    #[zbus(property)]
    fn routes(&self) -> zbus::Result<Vec<Vec<u32>>>;

    /// RouteData property
    #[zbus(property)]
    fn route_data(&self) -> zbus::Result<Vec<HashMap<String, zbus::OwnedVariant>>>;
}

impl IP4ConfigProxy<'_> {
    pub async fn new_from_path(
        device_path: zbus::OwnedObjectPath,
        connection: &Connection,
    ) -> zbus::Result<IP4ConfigProxy<'_>> {
        IP4ConfigProxy::builder(connection)
            .path(device_path)
            .build()
            .await
    }
}

#[proxy(
    default_path = "/org/freedesktop/NetworkManager/IP6Config",
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.IP6Config",
    assume_defaults = true
)]
pub trait IP6Config {
    /// Addresses property
    #[zbus(property)]
    fn addresses(&self) -> zbus::Result<Vec<(Vec<u8>, u32, Vec<u8>)>>;

    /// Routes property
    #[zbus(property)]
    fn routes(&self) -> zbus::Result<Vec<(Vec<u8>, u32, Vec<u8>, u32)>>;

    /// RouteData property
    #[zbus(property)]
    fn route_data(&self) -> zbus::Result<Vec<HashMap<String, zbus::OwnedVariant>>>;
}

impl IP6ConfigProxy<'_> {
    pub async fn new_from_path(
        device_path: zbus::OwnedObjectPath,
        connection: &Connection,
    ) -> zbus::Result<IP6ConfigProxy<'_>> {
        IP6ConfigProxy::builder(connection)
            .path(device_path)
            .build()
            .await
    }
}
