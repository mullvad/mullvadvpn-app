use std::collections::HashMap;

use zbus::blocking::Connection;
use zvariant::{OwnedObjectPath, OwnedValue};

// TODO: Inline
type O = OwnedObjectPath;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a DBus connection")]
    Connect(#[source] zbus::Error),
    // A Proxy is a helper to interact with an interface on a remote object.
    #[error("Failed to create proxy for object {0}")]
    Proxy(#[source] zbus::Error),
    // Failed to call some method on a proxy.
    #[error("Failed to call method on proxy {0}")]
    Call(#[source] zbus::Error),
}

#[derive(Clone)]
pub struct NetworkManager {
    pub dbus_connection: Connection,
}

impl NetworkManager {
    pub fn new() -> Result<Self, Error> {
        let dbus_connection = crate::get_connection_zbus().map_err(Error::Connect)?;
        let network_manager = NetworkManager { dbus_connection };
        // TODO: Assert a recent-enough version of network manager is installed. Not even sure if
        // necessary.
        network_manager.ensure_can_be_used_to_manage_dns()?;

        Ok(network_manager)
    }

    /// Check if NetworkManager is capable of managing DNS.
    pub fn ensure_can_be_used_to_manage_dns(&self) -> Result<bool, Error> {
        Ok(true) // TODO: Add DNS capabilities.
    }

    pub fn create_wg_tunnel(&self, wg_config: DeviceConfig) -> Result<WireguardTunnel, Error> {
        WireguardTunnel::new(self.clone(), wg_config)
    }

    fn activate_connection<'a>(
        &self,
        connection: OwnedObjectPath,
    ) -> std::result::Result<ConnectionActiveProxyBlocking<'a>, Error> {
        let device = OwnedObjectPath::try_from("/").unwrap(); // TODO: unwrap
        let specified_object = OwnedObjectPath::try_from("/").unwrap(); // TODO: unwrap
        as_nm(&self.dbus_connection)?
            .activate_connection(connection, device, specified_object)
            .map_err(Error::Call)
    }

    fn add_connection_2(
        &self,
        settings: DeviceConfig,
    ) -> Result<(OwnedObjectPath, DeviceConfig), Error> {
        // Blocks auto-connect on the new profile.
        const NM_ADD_CONNECTION_VOLATILE: u32 = 0x2;
        let flags = NM_ADD_CONNECTION_VOLATILE;
        let args = HashMap::default();
        as_settings(&self.dbus_connection)?
            .add_connection_2(settings, flags, args)
            .map_err(Error::Call)
    }
}

fn as_settings<'a>(conn: &Connection) -> Result<NetworkManagerSettingsProxyBlocking<'a>, Error> {
    NetworkManagerSettingsProxyBlocking::new(conn).map_err(Error::Proxy)
}

fn as_nm<'a>(conn: &Connection) -> Result<NetworkManagerProxyBlocking<'a>, Error> {
    NetworkManagerProxyBlocking::new(conn).map_err(Error::Proxy)
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

#[derive(Debug)]
/// TODO: Document
pub struct WireguardTunnel {
    /// TODO: Document
    active: ConnectionActiveProxyBlocking<'static>,
    /// TODO: Document
    nm: NetworkManagerProxyBlocking<'static>,
    /// TODO: Document
    device: DeviceProxyBlocking<'static>,
}

impl WireguardTunnel {
    pub fn new(nm: NetworkManager, wg_config: DeviceConfig) -> Result<Self, Error> {
        let connection = nm
            .add_connection_2(wg_config)
            .inspect_err(|err| {
                log::error!(
                    "Failed to create a new interface via AddConnection2: {}",
                    err
                );
            })?
            .0;

        let active = nm.activate_connection(connection)?;
        let devices = active.devices().map_err(Error::Proxy)?;
        // TODO: Why do we only consider the first device?
        let device = devices.into_iter().next().unwrap();

        let nm = as_nm(&nm.dbus_connection)?;

        Ok(WireguardTunnel { active, nm, device })
    }

    pub fn remove(self) -> Result<(), Error> {
        let device_object = OwnedObjectPath::from(self.device.into_inner().path().clone());
        let deactivation_result = self
            .nm
            .deactivate_connection(device_object)
            .map_err(Error::Call);

        let config_result = self
            .active
            .connection()
            .map_err(Error::Proxy)?
            .delete()
            .map_err(Error::Call);

        deactivation_result?;
        config_result?;
        Ok(())
    }

    pub fn get_interface_name(&self) -> Result<String, Error> {
        self.device.interface().map_err(Error::Call)
    }
}

/// <https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager>
#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
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
    #[zbus(object = "ConnectionActive")]
    fn activate_connection(&self, connection: O, device: O, specified_object: O);

    fn deactivate_connection(&self, active: OwnedObjectPath) -> Result<(), zbus::Error>;

    //  methods
    //
    //  GetDevices              (OUT        ao        devices);
    //  GetAllDevices           (OUT        ao        devices);
    //  GetDeviceByIpIface      (IN         s         iface,
    //                          OUT         o         device);
    //  ActivateConnection      (IN         o         connection,
    //                          IN          o         device,
    //                          IN          o         specific_object,
    //                          OUT         o         active_connection);
    //  AddAndActivateConnection    (IN     a{sa{sv}} connection,
    //                              IN      o         device,
    //                              IN      o         specific_object,
    //                              OUT     o         path,
    //                              OUT     o         active_connection);
    //  DeactivateConnection     (IN        o         active_connection);
    //  Sleep                    (IN        b         sleep);
    //  Enable                   (IN        b         enable);
    //  GetPermissions           (OUT       a{ss}     permissions);
    //  SetLogging               (IN        s         level,
    //                           IN         s         domains);
    //  GetLogging               (OUT       s         level,
    //                           OUT        s         domains);
    //  CheckConnectivity        (OUT       u         connectivity);
    //  state                    (OUT       u         state);
    //
    //  properties
    //
    //  Devices                  readable   ao
    //  AllDevices               readable   ao
    //  NetworkingEnabled        readable   b
    //  WirelessEnabled          readwrite  b
    //  WirelessHardwareEnabled  readable   b
    //  WwanEnabled              readwrite  b
    //  WwanHardwareEnabled      readable   b
    //  WimaxEnabled             readwrite  b
    //  WimaxHardwareEnabled     readable   b
    //  ActiveConnections        readable   ao
    //  PrimaryConnection        readable   o
    //  PrimaryConnectionType    readable   s
    //  Metered                  readable   u
    //  ActivatingConnection     readable   o
    //  Startup                  readable   b
    //  Version                  readable   s
    //  State                    readable   u
    //  Connectivity             readable   u
    //  GlobalDnsConfiguration   readwrite  a{sv}
}

/// https://networkmanager.pages.freedesktop.org/NetworkManager/NetworkManager/gdbus-org.freedesktop.NetworkManager.Settings
#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager.Settings",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait NetworkManagerSettings {
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
        settings: DeviceConfig,
        flags: u32,
        args: HashMap<String, OwnedValue>,
    ) -> Result<(OwnedObjectPath, DeviceConfig), zbus::Error>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager.Connection.Active", // TODO: Check if makes sense.
    default_path = "/org/freedesktop/NetworkManager/Connection/Active"
)]
trait ConnectionActive {
    #[zbus(object = "Connection")]
    fn connection(&self);

    /// Array of object paths representing devices which are part of this active connection.
    ///
    /// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Connection.Active.html#gdbus-property-org-freedesktop-NetworkManager-Connection-Active.Devices
    #[zbus(object = "Device", object_vec)]
    fn devices(&self);
}

/// <https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Settings.Connection>
#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager.Settings.Connection", // TODO: Check if makes sense.
    default_path = "/org/freedesktop/NetworkManager/Settings/Connection"
)]
trait Connection {
    fn delete(&self) -> Result<(), zbus::Error>;
}

/// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.Device
#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager.Device", // TODO: Check if makes sense.
    default_path = "/org/freedesktop/NetworkManager/Device"
)]
trait Device {
    #[zbus(property)]
    fn interface(&self) -> Result<String, zbus::Error>;
}
