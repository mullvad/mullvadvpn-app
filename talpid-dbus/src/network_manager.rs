//! NetworkManager is the one-stop-shop of network configuration on Linux.
use super::systemd_resolved;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::Path;

use serde::Deserialize;
use zbus::blocking::{Connection, Proxy};
use zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

const NM_BUS: &str = "org.freedesktop.NetworkManager";
const NM_MANAGER: &str = "org.freedesktop.NetworkManager";
const NM_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";
const CONNECTIVITY_CHECK_KEY: &str = "ConnectivityCheckEnabled";

const NM_DNS_MANAGER: &str = "org.freedesktop.NetworkManager.DnsManager";
const NM_DNS_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";
const NM_DEVICE: &str = "org.freedesktop.NetworkManager.Device";

const NM_IP4_CONFIG: &str = "org.freedesktop.NetworkManager.IP4Config";
const NM_IP6_CONFIG: &str = "org.freedesktop.NetworkManager.IP6Config";
const RC_MANAGEMENT_MODE_KEY: &str = "RcManager";
const DNS_MODE_KEY: &str = "Mode";
const DNS_FIRST_PRIORITY: i32 = -2147483647;

const NM_DEVICE_STATE_IP_CHECK: u32 = 80;
const NM_DEVICE_STATE_SECONDARY: u32 = 90;
const NM_DEVICE_STATE_ACTIVATED: u32 = 100;

const NM_SETTINGS_INTERFACE: &str = "org.freedesktop.NetworkManager.Settings";
const NM_SETTINGS_CONNECTION_INTERFACE: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const NM_SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";
const NM_CONNECTION_ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";

/// Blocks auto-connect on the new profile.
const NM_ADD_CONNECTION_VOLATILE: u32 = 0x2;

const MINIMUM_SUPPORTED_MAJOR_VERSION: u32 = 1;
const MINIMUM_SUPPORTED_MINOR_VERSION: u32 = 16;

const MAXIMUM_SUPPORTED_MAJOR_VERSION: u32 = 1;
const MAXIMUM_SUPPORTED_MINOR_VERSION: u32 = 26;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while communicating over Dbus")]
    Dbus(#[from] zbus::Error),

    // A Proxy is a helper to interact with an interface on a remote object.
    #[error("Failed to create proxy for object {0}")]
    Proxy(#[source] zbus::Error),

    #[error("Failed while setting property {0}")]
    SetProperty(#[source] zbus::fdo::Error),

    #[error("Failed while getting property {0}")]
    GetProperty(#[source] zbus::Error),

    #[error("Failed to match the returned D-Bus object with expected type")]
    MatchDBusTypeError(#[from] zvariant::Error),

    #[error(
        "NM is configured to manage DNS via systemd-resolved but systemd-resolved is not managing /etc/resolv.conf: {0}"
    )]
    SystemdResolvedNotManagingResolvconf(systemd_resolved::Error),

    #[error("Configuration has no device associated to it")]
    NoDevice,

    #[error("NetworkManager is too old: {0}.{1}")]
    NMTooOld(u32, u32),

    #[error("NetworkManager is too new to manage DNS: {0}.{1}")]
    NMTooNewFroDns(u32, u32),

    #[error("Failed to parse NetworkManager version string: {0}")]
    ParseNmVersionError(String),

    #[error("Device inactive: {0}")]
    DeviceNotReady(u32),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("NetworkManager not detected")]
    NetworkManagerNotDetected,

    #[error("NetworkManager is using dnsmasq to manage DNS")]
    UsingDnsmasq,

    #[error("NetworkManager is too old: {0}")]
    TooOldNetworkManager(String),

    #[error("NetworkManager is not managing DNS")]
    NetworkManagerNotManagingDns,

    #[error("Failed to get devices from NetworkManager object")]
    ObtainDevices,
}

/// NetworkManager device configurations. A map from device to configuration options.
///
/// # DBUS
///
/// This corresponds to the DBUS type signature `a{sa{sv}}`
///
/// ### Breakdown:
///
/// - `s`        : `string`
/// - `v`        : `variant`
/// - `a{}`      : `map`
/// - `a{sv}`    : `map<string, variant>`
/// - `a{sa{sv}}`: `map<string, map<string, variant>`
pub type DeviceConfig = HashMap<String, HashMap<String, OwnedValue>>;

/// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device.html#gdbus-method-org-freedesktop-NetworkManager-Device.GetAppliedConnection
// TODO: Check if this should be a shared version of [`DeviceConfig`], aka HashMap<String, HashMap<String, Value<'a'>>>;
type NetworkSettings = DeviceConfig;

/// A type encapsulating IPv4 and IPv6 addresses. All addresses will include "address" (an IP address string), and "prefix" (a uint). Some addresses may include additional attributes.
///
/// This type is only documented in the `AddressData` property in the IP4 and IP6 interfaces AFAICT.
///
/// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP6Config.html#gdbus-property-org-freedesktop-NetworkManager-IP6Config.Routes
///
/// # DBUS
///
/// This corresponds to the DBUS type signature `a{sv}`
type Address = HashMap<String, OwnedValue>;

/// Array of IP route data objects.
///
/// See upstream docs for more information. This type is shared between IP4 and IP6 modules:
/// - https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP4Config.html#gdbus-property-org-freedesktop-NetworkManager-IP4Config.RouteData
/// - https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP6Config.html#gdbus-property-org-freedesktop-NetworkManager-IP6Config.RouteData
///
/// # DBUS
/// This corresponds to the DBUS type signature `aa{sv}`
type RouteData = Vec<HashMap<String, OwnedValue>>;

/*
// TODO: Check if this is every needed.
/// https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMConnectivityState
#[repr(u32)]
enum NMConnectivityState {
    /// Network connectivity is unknown.
    UNKNOWN = 1,
    /// The host is not connected to any network.
    NONE = 2,
    /// The host is behind a captive portal and cannot reach the full Internet.
    PORTAL = 3,
    /// The host is connected to a network, but does not appear to be able to reach the full Internet.
    LIMITED = 4,
    /// The host is connected to a network, and appears to be able to reach the full Internet.
    FULL = 5,
}
*/

// TODO: Check if this is every needed.
/// These methods are described in detail by `man nm-settings`. Grep for `ipv6.method`.
///
/// https://www.networkmanager.dev/docs/libnm/1.44.4/NMSettingIP6Config.html
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Deserialize,
    zvariant::Type,
    zvariant::OwnedValue,
    zvariant::Value,
)]
#[serde(rename_all = "kebab-case")]
enum Ip6ConfigMethod {
    /// IPv6 is not required or is handled by some other mechanism, and NetworkManager should not configure IPv6 for
    /// this connection.
    Ignore,
    /// IPv6 configuration should be automatically determined via a method appropriate for the hardware interface,
    /// ie router advertisements, DHCP, or PPP or some other device-specific manner.
    Auto,
    /// IPv6 configuration should be automatically determined via DHCPv6 only and router advertisements should be
    /// ignored.
    Dhcp,
    /// IPv6 configuration should be automatically configured for link-local-only operation.
    LinkLocal,
    /// All necessary IPv6 configuration (addresses, prefix, DNS, etc) is specified in the setting's properties.
    Manual,
    /// This connection specifies configuration that allows other computers to connect through it to the default
    /// network (usually the Internet).
    Shared,
    /// IPv6 is disabled for the connection.
    ///
    /// Since 1.20
    Disabled,
}

/// Implements functionality to control NetworkManager over DBus.
pub struct NetworkManager {
    connection: Connection,
}

impl NetworkManager {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            connection: crate::get_connection()?,
        })
    }

    pub fn create_wg_tunnel(&self, config: &DeviceConfig) -> Result<WireguardTunnel, Error> {
        self.nm_supports_wireguard()?;
        let tunnel = self.create_wg_tunnel_inner(config)?;
        if let Err(err) = self.wait_until_device_is_ready(&tunnel.device) {
            if let Err(removal_error) = self.remove_tunnel(tunnel) {
                log::error!(
                    "Failed to remove WireGuard tunnel after it not becoming ready fast enough: {}",
                    removal_error
                );
            }
            return Err(err);
        }

        Ok(tunnel)
    }

    pub fn get_interface_name(&self, tunnel: &WireguardTunnel) -> Result<String, Error> {
        tunnel
            .device_proxy(&self.connection)?
            .get_property("Interface")
            .map_err(Error::Dbus)
    }

    pub fn get_device_state(&self, device: &ObjectPath<'_>) -> Result<u32, Error> {
        self.as_device(device)?
            .get_property("State")
            .map_err(Error::Dbus)
    }

    fn create_wg_tunnel_inner(&self, config: &DeviceConfig) -> Result<WireguardTunnel, Error> {
        let connection_config = match self.add_connection_2(config) {
            Ok((path, _config)) => path,
            // TODO: Is this error case really necessary? It might have been for older versions of
            // network manager, but I don't think it's a problem any more. It has existed since
            // NetworkManager 1.20. For context, Debian 11 ship NM 1.30.
            // const DBUS_UNKNOWN_METHOD: &str = "org.freedesktop.DBus.Error.UnknownMethod";
            // Err(Error::Dbus(dbus_error)) if dbus_error.name() == Some(DBUS_UNKNOWN_METHOD) => {
            //     self.add_connection_unsaved(config)?.0
            // }
            Err(err) => {
                log::error!(
                    "Failed to create a new interface via AddConnection2: {}",
                    err
                );
                return Err(err);
            }
        };

        let active_connection = self.activate_connection(
            &connection_config,
            &ObjectPath::try_from("/").unwrap(),
            &ObjectPath::try_from("/").unwrap(),
        )?;

        /// Array of object paths representing devices which are part of this active connection.
        ///
        /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Connection.Active.html#gdbus-property-org-freedesktop-NetworkManager-Connection-Active.Devices
        type Devices = Vec<OwnedObjectPath>;
        let device_paths: Devices = self
            .as_active_connection(&active_connection)?
            .get_property("Devices")
            .map_err(Error::Dbus)?;
        // TODO: Why do we only consider the first device?
        let device_path = device_paths.into_iter().next().ok_or(Error::NoDevice)?;

        Ok(WireguardTunnel {
            config: connection_config,
            connection: active_connection,
            device: device_path,
        })
    }

    pub fn nm_supports_wireguard(&self) -> Result<(), Error> {
        let (major, minor) = self.version()?;
        Self::ensure_nm_is_new_enough_for_wireguard(major, minor)?;
        Self::ensure_nm_is_old_enough_for_dns(major, minor)
    }

    pub fn nm_version_dns_works(&self) -> Result<(), Error> {
        let (major, minor) = self.version()?;
        Self::ensure_nm_is_old_enough_for_dns(major, minor)
    }

    pub fn version_string(&self) -> Result<String, Error> {
        self.as_nm_manager()?
            .get_property("Version")
            .map_err(Error::Dbus)
    }

    fn ensure_nm_is_new_enough_for_wireguard(major: u32, minor: u32) -> Result<(), Error> {
        if major < MINIMUM_SUPPORTED_MAJOR_VERSION
            || (minor < MINIMUM_SUPPORTED_MINOR_VERSION && major == MINIMUM_SUPPORTED_MAJOR_VERSION)
        {
            Err(Error::NMTooOld(major, minor))
        } else {
            Ok(())
        }
    }

    /// TODO: Revisit this, I've seen people complain about it on GitHub.
    fn ensure_nm_is_old_enough_for_dns(
        major_version: u32,
        minor_version: u32,
    ) -> Result<(), Error> {
        if major_version > MAXIMUM_SUPPORTED_MAJOR_VERSION
            || (minor_version > MAXIMUM_SUPPORTED_MINOR_VERSION
                && major_version >= MAXIMUM_SUPPORTED_MAJOR_VERSION)
        {
            Err(Error::NMTooNewFroDns(major_version, minor_version))
        } else {
            Ok(())
        }
    }

    /// Get NetworkMangager version as a tuple of (<major>, <minor>).
    fn version(&self) -> Result<(u32, u32), Error> {
        let version = self.version_string()?;
        Self::parse_nm_version(&version).ok_or(Error::ParseNmVersionError(version))
    }

    fn parse_nm_version(version: &str) -> Option<(u32, u32)> {
        let mut parts = version.split('.').map(|part| part.parse().ok());

        let major_version: u32 = parts.next()??;
        let minor_version: u32 = parts.next()??;
        Some((major_version, minor_version))
    }

    /// org.freedesktop.NetworkManager.Settings.AddConnection2 (since 1.20)
    ///
    /// # Result
    /// A tuple of
    /// - Object path of the new connection that was just added.
    /// - "Output argument, currently no additional results are returned" (lol).
    ///
    /// # Documenation
    ///
    /// https://networkmanager.pages.freedesktop.org/NetworkManager/NetworkManager/gdbus-org.freedesktop.NetworkManager.Settings.html#gdbus-method-org-freedesktop-NetworkManager-Settings.AddConnection2
    fn add_connection_2(
        &self,
        settings_map: &DeviceConfig,
    ) -> Result<(OwnedObjectPath, DeviceConfig), Error> {
        let args: HashMap<String, OwnedValue> = HashMap::default();
        let flags = NM_ADD_CONNECTION_VOLATILE;

        self.as_settings()?
            .call("AddConnection2", &(settings_map, flags, args))
            .map_err(Error::Dbus)

        // Proxy::new(NM_BUS, NM_SETTINGS_PATH, RPC_TIMEOUT, &*self.connection)
        //     .method_call(
        //         NM_SETTINGS_INTERFACE,
        //         "AddConnection2",
        //         (settings_map, flags, args),
        //     )
        //     .map_err(Error::Dbus)
    }

    /// org.freedesktop.NetworkManager.Settings.ActivateConnection
    ///
    /// # Input
    /// - `connection`: The connection to activate.
    ///
    /// If "/" is given, a valid device path must be given, and NetworkManager picks the best connection to activate for
    /// the given device. VPN connections must always pass a valid connection path.
    ///
    /// - `device`: The object path of device to be activated for physical connections.
    ///
    /// This parameter is ignored for VPN connections, because the specific_object (if provided) specifies the device
    /// to use.
    ///
    /// - `specific_object`: The path of a connection-type-specific object this activation should use.
    ///
    /// For VPN connections, pass the object path of an ActiveConnection object that should serve as the "base"
    /// connection (to which the VPN connections lifetime will be tied), or pass "/" and NM will automatically use the
    /// current default device.
    ///
    /// # Result
    ///
    /// Object path of the active connection object representing this active connection.
    ///
    /// # Documenation
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.html#gdbus-method-org-freedesktop-NetworkManager.ActivateConnection
    fn activate_connection(
        &self,
        connection: &ObjectPath<'_>,
        device: &ObjectPath<'_>,
        specific_object: &ObjectPath<'_>,
    ) -> Result<OwnedObjectPath, Error> {
        self.as_nm_manager()?
            .call("ActivateConnection", &(connection, device, specific_object))
            .map_err(Error::Dbus)
    }

    // TODO: I don't think this is needed anymore.
    // fn add_connection_unsaved(
    //     &self,
    //     settings_map: &DeviceConfig,
    // ) -> Result<(dbus::Path<'static>,), Error> {
    //     Proxy::new(NM_BUS, NM_SETTINGS_PATH, RPC_TIMEOUT, &*self.connection)
    //         .method_call(
    //             NM_SETTINGS_INTERFACE,
    //             "AddConnectionUnsaved",
    //             (settings_map,),
    //         )
    //         .map_err(Error::Dbus)
    // }

    // TODO: Do not implement this manually, derive it automatically via zbus.
    // https://z-galaxy.github.io/zbus/client.html#watching-for-changes
    #[allow(unused)]
    fn wait_until_device_is_ready(&self, device: &ObjectPath<'_>) -> Result<(), Error> {
        todo!("Check if we can use signals instead.");
        /*
                const NM_DEVICE_STATE_CHANGED: &str = "StateChanged";
                let device_state = self.get_device_state(device)?;

                if !device_is_ready(device_state) {
                    const DEVICE_READY_TIMEOUT: Duration = Duration::from_secs(15);
                    let deadline = Instant::now() + DEVICE_READY_TIMEOUT;

                    let mut match_rule = MatchRule::new_signal(NM_DEVICE, NM_DEVICE_STATE_CHANGED);

                    match_rule.path = Some(device.clone().into_static());
                    let device_state = Arc::new(AtomicU32::new(device_state));

                    {
                        let shared_device_state = device_state.clone();
                        let device_matcher = self
                            .connection
                            .add_match(
                                match_rule,
                                move |state_change: DeviceStateChange, _connection, _message| {
                                    log::debug!("Received new tunnel state change: {:?}", state_change);
                                    let new_state = state_change.new_state;
                                    shared_device_state.store(new_state, Ordering::Release);
                                    true
                                },
                            )
                            .map_err(Error::Dbus)?;
                        while Instant::now() < deadline
                            && !device_is_ready(device_state.load(Ordering::Acquire))
                        {
                            if let Err(err) = self.connection.process(RPC_TIMEOUT) {
                                log::error!(
                                    "DBus connection failed while waiting for device to be ready: {}",
                                    err
                                );
                            }
                        }

                        if let Err(err) = self.connection.remove_match(device_matcher) {
                            log::error!("Failed to remove match from DBus connection: {}", err);
                        }
                    }

                    let final_device_state = device_state.load(Ordering::Acquire);
                    if !device_is_ready(final_device_state) {
                        return Err(Error::DeviceNotReady(final_device_state));
                    }
                }
                Ok(())
        */
    }

    pub fn remove_tunnel(&self, tunnel: WireguardTunnel) -> Result<(), Error> {
        // TODO: Check if this is still correct
        let deactivation_result: Result<(), Error> = self
            .as_nm_manager()?
            .call("DeactivateConnection", &(&tunnel.connection,))
            .map_err(Error::Dbus);

        // TODO: Check if this is still correct
        let config_result: Result<(), Error> = tunnel
            .config_proxy(&self.connection)?
            .call("Delete", &())
            .map_err(Error::Dbus);
        deactivation_result?;
        config_result?;
        Ok(())
    }

    /// Ensures NetworkManager's connectivity check is disabled and returns the connectivity check
    /// previous state. Returns true only if the connectivity check was enabled and is now
    /// disabled. Disabling the connectivity check should be done before a firewall is applied
    /// due to the fact that blocking DNS requests can make it hang:
    /// <https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/404>
    ///
    /// TODO: The linked above issue has been fixed since ~2022. What does that entail for this
    /// function?
    pub fn ensure_connectivity_check_disabled(&self) -> Result<bool, Error> {
        let nm_manager = self.as_nm_manager()?;
        let connectivity_check = nm_manager.get_property(CONNECTIVITY_CHECK_KEY)?;
        match connectivity_check {
            true => {
                if let Err(err) = self.disable_connectivity_check() {
                    log::error!(
                        "Failed to disable NetworkManager connectivity check: {}",
                        err
                    );
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            false => Ok(false),
        }
    }

    /// Enable NetworkManager's connectivity check.
    ///
    /// Returns `Ok(())` if it succeeded.
    pub fn enable_connectivity_check(&self) -> Result<(), Error> {
        // Turn on the connectivity check by writing `true` to the property.
        self.as_nm_manager()?
            .set_property(CONNECTIVITY_CHECK_KEY, true)
            .map_err(Error::SetProperty)
    }

    /// Disable NetworkManager's connectivity check.
    ///
    /// Returns `Ok(())` if it succeeded.
    fn disable_connectivity_check(&self) -> Result<(), Error> {
        // Turn off the connectivity check by writing `false` to the property.
        self.as_nm_manager()?
            .set_property(CONNECTIVITY_CHECK_KEY, false)
            .map_err(Error::SetProperty)
    }

    /// org.freedesktop.NetworkManager
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager
    fn as_nm_manager(&self) -> Result<Proxy<'_>, Error> {
        Proxy::new(&self.connection, NM_BUS, NM_MANAGER_PATH, NM_MANAGER).map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.Connection.Active
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Connection.Active
    fn as_active_connection<'a>(
        &self,
        active_connection: &ObjectPath<'a>,
    ) -> Result<Proxy<'a>, Error> {
        Proxy::new(
            &self.connection,
            NM_BUS,
            active_connection,
            NM_CONNECTION_ACTIVE,
        )
        .map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.DnsManager
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.DnsManager
    fn as_dns_manager(&self) -> Result<Proxy<'_>, Error> {
        Proxy::new(
            &self.connection,
            NM_BUS,
            NM_DNS_MANAGER_PATH,
            NM_DNS_MANAGER,
        )
        .map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.Device
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device
    fn as_device<'a>(&self, device: &ObjectPath<'a>) -> Result<Proxy<'a>, Error> {
        Proxy::new(&self.connection, NM_BUS, device, NM_DEVICE).map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.IP4Config
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP4Config
    fn as_ip4config<'a>(&self, ip4config: &ObjectPath<'a>) -> Result<Proxy<'a>, Error> {
        Proxy::new(&self.connection, NM_BUS, ip4config, NM_IP4_CONFIG).map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.IP6Config
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP6Config
    fn as_ip6config<'a>(&self, ip4config: &ObjectPath<'a>) -> Result<Proxy<'a>, Error> {
        Proxy::new(&self.connection, NM_BUS, ip4config, NM_IP6_CONFIG).map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.Settings
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Settings
    fn as_settings(&self) -> Result<Proxy<'_>, Error> {
        Proxy::new(
            &self.connection,
            NM_BUS,
            NM_SETTINGS_PATH,
            NM_SETTINGS_INTERFACE,
        )
        .map_err(Error::Proxy)
    }

    pub fn ensure_network_manager_exists(&self) -> Result<(), Error> {
        match self.as_nm_manager()?.get_property::<String>("Version") {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("Failed to read version of NetworkManager {}", err);
                Err(Error::NetworkManagerNotDetected)
            }
        }
    }

    pub fn ensure_can_be_used_to_manage_dns(&self) -> Result<(), Error> {
        self.ensure_resolv_conf_is_managed()?;
        self.ensure_network_manager_exists()?;
        self.nm_version_dns_works()?;
        Ok(())
    }
    pub fn ensure_resolv_conf_is_managed(&self) -> Result<(), Error> {
        // check if NM is set to manage resolv.conf
        let management_mode: String = self
            .as_dns_manager()?
            .get_property(RC_MANAGEMENT_MODE_KEY)
            .map_err(Error::GetProperty)?;

        // TODO: Are these well defined? If so, turn into enum.
        if management_mode == "unmanaged" {
            return Err(Error::NetworkManagerNotManagingDns);
        }

        if management_mode == "systemd-resolved" {
            return match systemd_resolved::SystemdResolved::new() {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::SystemdResolvedNotManagingResolvconf(err)),
            };
        }

        // TODO: Are these well defined? If so, turn into enum.
        let dns_mode: String = self
            .as_dns_manager()?
            .get_property(DNS_MODE_KEY)
            .map_err(Error::GetProperty)?;

        match dns_mode.as_ref() {
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

    /// ffs.
    /// TODO: Refactor.
    pub fn set_dns(
        &mut self,
        interface_name: &str,
        servers: &[IpAddr],
    ) -> Result<DeviceConfig, Error> {
        let device_path = self.fetch_device(interface_name)?;
        self.wait_until_device_is_ready(&device_path)?;

        let device = self.as_device(&device_path)?;
        // Get the currently applied connection on the device.
        // This is a snapshot of the last activated connection on the device,
        // that is the configuration that is currently applied on the device.
        let (mut settings, version_id): (NetworkSettings, u64) = {
            let flags: u32 = 0;
            device
                .call("GetAppliedConnection", &flags)
                .map_err(Error::Dbus)?
        };

        // Keep changed routes.
        // These routes were modified outside NM, likely by RouteManager.
        if let Some(ipv4_settings) = settings.get_mut("ipv4") {
            // Object path of the Ip4Config object describing the configuration of the device.
            // TODO: Only valid when the device is in the NM_DEVICE_STATE_ACTIVATED state.
            let device_ip4_config: ObjectPath<'_> = device
                .get_property("Ip4Config")
                .map_err(Error::GetProperty)?;

            let device_routes: Vec<Vec<u32>> = self
                .as_ip4config(&device_ip4_config)?
                .get_property("Routes")
                .map_err(Error::GetProperty)?;

            let device_route_data: RouteData = self
                .as_ip4config(&device_ip4_config)?
                .get_property("RouteData")
                .map_err(Error::GetProperty)?;

            // All of these unwraps should be fine, since we are not dealing with file descriptors,
            // which the zvariant docs say is the only fallible conversion from Value -> OwnedValue.
            ipv4_settings.insert(
                "route-metric".to_string(),
                Value::new(0u32).try_into_owned().unwrap(),
            );
            ipv4_settings.insert(
                "routes".to_string(),
                Value::new(device_routes).try_into_owned().unwrap(),
            );
            ipv4_settings.insert(
                "route-data".to_string(),
                Value::new(device_route_data).try_into_owned().unwrap(),
            );
        }

        if let Some(ipv6_settings) = settings.get_mut("ipv6") {
            // Object path of the Ip6Config object describing the configuration of the device.
            // TODO: Only valid when the device is in the NM_DEVICE_STATE_ACTIVATED state.
            let device_ip6_config: ObjectPath<'_> = device
                .get_property("Ip6Config")
                .map_err(Error::GetProperty)?;
            let ip6_configuration_set = self.as_ip6config(&device_ip6_config)?;

            /// Array of tuples of IPv6 address/prefix/gateway.
            /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP6Config.html#gdbus-property-org-freedesktop-NetworkManager-IP6Config.Addresses
            // TODO: This property is deprecated according to NM docs. Use `AddressData` or
            // `Gateway` instead.
            type AAYUAY = Vec<(Vec<u8>, u32, Vec<u8>)>;
            let device_addresses6: AAYUAY = ip6_configuration_set
                .get_property("Addresses")
                .map_err(Error::GetProperty)?;

            /// Tuples of IPv6 route/prefix/next-hop/metric.
            /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.IP6Config.html#gdbus-property-org-freedesktop-NetworkManager-IP6Config.Routes
            // TODO: This property is deprecated according to NM docs. Use `RouteData` instead
            // (which we already do ??).
            type AAYUAYU = Vec<(Vec<u8>, u32, Vec<u8>, u32)>;
            let device_routes6: AAYUAYU = ip6_configuration_set
                .get_property("Routes")
                .map_err(Error::GetProperty)?;

            let device_route6_data: RouteData = ip6_configuration_set
                .get_property("RouteData")
                .map_err(Error::GetProperty)?;

            // All of these unwraps should be fine, since we are not dealing with file descriptors,
            // which the zvariant docs say is the only fallible conversion from Value -> OwnedValue.
            ipv6_settings.insert(
                "route-metric".to_string(),
                Value::new(0u32).try_into_owned().unwrap(),
            );
            ipv6_settings.insert(
                "routes".to_string(),
                Value::new(device_routes6).try_into_owned().unwrap(),
            );
            ipv6_settings.insert(
                "route-data".to_string(),
                Value::new(device_route6_data).try_into_owned().unwrap(),
            );

            // * if the link contains link local addresses, addresses shouldn't be reset
            // * if IPv6 isn't enabled, IPv6 method will be set to "ignore", in which case we
            // shouldn't reapply any config for ipv6
            // TODO: We ignore 'disabled', which I think is wrong.
            let should_reset_addresses =
                |method| method != Ip6ConfigMethod::LinkLocal && method != Ip6ConfigMethod::Ignore;
            if let Some(method) = ipv6_settings.get("method")
                && let Ok(method) = Ip6ConfigMethod::try_from(method.clone())
                && should_reset_addresses(method)
            {
                ipv6_settings.insert(
                    "addresses".to_string(),
                    Value::new(device_addresses6).try_into_owned().unwrap(),
                );
            }
        }

        let mut settings_backup = DeviceConfig::new();
        // TODO: Why not just call clone once and be done?
        for (top_key, map) in settings.iter() {
            let mut inner_dict = HashMap::<String, OwnedValue>::new();
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
            Self::update_dns_ip4_config(&mut settings, "ipv4", v4_dns);
        }

        let v6_dns: Vec<Vec<u8>> = servers
            .iter()
            .filter_map(|server| match server {
                IpAddr::V4(_) => None,
                IpAddr::V6(server) => Some(server.octets().to_vec()),
            })
            .collect();
        if !v6_dns.is_empty() {
            Self::update_dns_ip6_config(&mut settings, "ipv6", v6_dns);
        }

        if let Some(wg_config) = settings.get_mut("wireguard")
            && !wg_config.contains_key("fwmark")
        {
            log::error!("WireGuard config doesn't contain the firewall mark");
        }

        self.reapply_settings(&device_path, settings, version_id)?;
        Ok(settings_backup)
    }

    /// Attempts to update the configuration of a device without deactivating it.
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device.html#gdbus-method-org-freedesktop-NetworkManager-Device.Reapply
    pub fn reapply_settings(
        &self,
        device: &ObjectPath<'_>,
        settings: DeviceConfig,
        version_id: u64,
    ) -> Result<(), Error> {
        let flags: u32 = 0;
        let () = self
            .as_device(device)?
            .call("Reapply", &(settings, version_id, flags))?;
        Ok(())
    }

    /// Add [`dns_servers`] to current NetworkManager settings.
    fn update_dns_ip4_config(
        settings: &mut NetworkSettings,
        ip_protocol: &'static str,
        dns_servers: Vec<u32>, // A vec of 4x8 bit octets in Network Endian byte-order (BE).
    ) {
        let settings = match settings.get_mut(ip_protocol) {
            Some(ip_protocol) => ip_protocol,
            None => {
                settings.insert(ip_protocol.to_string(), HashMap::new());
                settings.get_mut(ip_protocol).unwrap()
            }
        };

        // TODO: Document where the f these properties come from
        settings.insert(
            "method".to_string(),
            Value::new("manual".to_string()).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns-priority".to_string(),
            Value::new(DNS_FIRST_PRIORITY).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns".to_string(),
            Value::new(dns_servers).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns-search".to_string(),
            Value::new(vec!["~.".to_string()]).try_into_owned().unwrap(),
        );
    }

    /// Add [`dns_servers`] to current NetworkManager settings.
    fn update_dns_ip6_config(
        settings: &mut NetworkSettings,
        ip_protocol: &'static str,
        dns_servers: Vec<Vec<u8>>, // A vec of 16x8 bit octets in Network Endian byte-order (BE).
    ) {
        let settings = match settings.get_mut(ip_protocol) {
            Some(ip_protocol) => ip_protocol,
            None => {
                settings.insert(ip_protocol.to_string(), HashMap::new());
                settings.get_mut(ip_protocol).unwrap()
            }
        };

        // TODO: Document where the f these properties come from
        settings.insert(
            "method".to_string(),
            Value::new("manual".to_string()).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns-priority".to_string(),
            Value::new(DNS_FIRST_PRIORITY).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns".to_string(),
            Value::new(dns_servers).try_into_owned().unwrap(),
        );
        settings.insert(
            "dns-search".to_string(),
            Value::new(vec!["~.".to_string()]).try_into_owned().unwrap(),
        );
    }

    pub fn fetch_device(&self, interface_name: &str) -> Result<ObjectPath<'_>, Error> {
        /// The list of realized network devices.
        ///
        /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.html#gdbus-property-org-freedesktop-NetworkManager.Devices
        type Devices<'a> = Vec<ObjectPath<'a>>;
        let devices: Devices<'_> = self
            .as_nm_manager()?
            .get_property("Devices")
            .map_err(Error::GetProperty)?;

        for device in devices {
            /// The name of the device's control (and often data) interface.
            ///
            /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device.html#gdbus-property-org-freedesktop-NetworkManager-Device.Interface
            type Interface = String;
            let device_name: Interface = self
                .as_device(&device)?
                .get_property("Interface")
                .map_err(Error::GetProperty)?;

            if device_name != interface_name {
                continue;
            }

            return Ok(device);
        }
        Err(Error::DeviceNotFound)
    }

    pub fn convert_address_to_dbus(address: IpAddr) -> Address {
        let mut result = Address::new();
        result.insert(
            "address".to_string(),
            Value::new(address.to_string()).try_into_owned().unwrap(),
        );
        let prefix: u32 = if address.is_ipv4() { 32 } else { 128 };
        result.insert(
            "prefix".to_string(),
            Value::new(prefix).try_into_owned().unwrap(),
        );
        result
    }
}

/*
#[derive(Debug)]
struct DeviceStateChange {
    new_state: u32,
    _old_state: u32,
    _reason: u32,
}

impl arg::ReadAll for DeviceStateChange {
    fn read(i: &mut arg::Iter<'_>) -> std::result::Result<Self, arg::TypeMismatchError> {
        Ok(DeviceStateChange {
            new_state: i.read()?,
            _old_state: i.read()?,
            _reason: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for DeviceStateChange {
    const NAME: &'static str = NM_DEVICE_STATE_CHANGED;
    const INTERFACE: &'static str = NM_DEVICE;
}
*/

#[derive(Debug)]
/// TODO: Document
pub struct WireguardTunnel {
    /// TODO: Document
    config: OwnedObjectPath,
    /// TODO: Document
    connection: OwnedObjectPath,
    /// TODO: Document
    device: OwnedObjectPath,
}

impl WireguardTunnel {
    /// org.freedesktop.NetworkManager.Device
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device
    fn device_proxy(&self, connection: &Connection) -> Result<Proxy<'_>, Error> {
        Proxy::new(connection, NM_BUS, &self.device, NM_DEVICE).map_err(Error::Proxy)
    }

    /// org.freedesktop.NetworkManager.Settings.Connection
    ///
    /// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Settings.Connection
    fn config_proxy(&self, connection: &Connection) -> Result<Proxy<'_>, Error> {
        Proxy::new(
            connection,
            NM_BUS,
            &self.config,
            NM_SETTINGS_CONNECTION_INTERFACE,
        )
        .map_err(Error::Proxy)
    }
}

pub fn device_is_ready(device_state: u32) -> bool {
    /// Any state above `NM_DEVICE_STATE_IP_CONFIG` is considered to be an OK state to change the
    /// DNS config. For the enums, see https://developer.gnome.org/NetworkManager/stable/nm-dbus-types.html#NMDeviceState
    const READY_STATES: [u32; 3] = [
        NM_DEVICE_STATE_IP_CHECK,
        NM_DEVICE_STATE_SECONDARY,
        NM_DEVICE_STATE_ACTIVATED,
    ];
    READY_STATES.contains(&device_state)
}

// Verify that the contents of /etc/resolv.conf match what NM expects them to be.
fn verify_etc_resolv_conf_contents() -> bool {
    let expected_resolv_conf = "/var/run/NetworkManager/resolv.conf";
    let actual_resolv_conf = "/etc/resolv.conf";
    eq_file_content(&expected_resolv_conf, &actual_resolv_conf)
}

fn eq_file_content<P: AsRef<Path>>(a: &P, b: &P) -> bool {
    let file_a = match File::open(a).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed to open file {}: {}", a.as_ref().display(), e);
            return false;
        }
    };
    let file_b = match File::open(b).map(BufReader::new) {
        Ok(file) => file,
        Err(e) => {
            log::debug!("Failed to open file {}: {}", b.as_ref().display(), e);
            return false;
        }
    };

    !file_a
        .lines()
        .zip(file_b.lines())
        .any(|(a, b)| match (a, b) {
            (Ok(a), Ok(b)) => a != b,
            _ => false,
        })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_versions() {
        NetworkManager::ensure_nm_is_new_enough_for_wireguard(1, 16).unwrap();
        NetworkManager::ensure_nm_is_old_enough_for_dns(1, 26).unwrap();
        assert!(NetworkManager::ensure_nm_is_new_enough_for_wireguard(1, 14).is_err());
        assert!(NetworkManager::ensure_nm_is_old_enough_for_dns(1, 28).is_err());
    }
}
