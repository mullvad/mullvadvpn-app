use std::net::IpAddr;
use talpid_dbus::{
    network_manager::{self, DeviceConfig, NetworkManager as DBus},
    systemd,
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "NetworkManager error")]
    NetworkManager(#[error(source)] talpid_dbus::network_manager::Error),

    #[error(display = "NetworkManager error")]
    Systemd(#[error(source)] systemd::Error),

    #[error(display = "NetworkManager not enabled")]
    NetworkManagerDisabled,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct NetworkManager {
    pub connection: DBus,
    device: Option<String>,
    settings_backup: Option<DeviceConfig>,
}

impl NetworkManager {
    pub fn new() -> Result<Self> {
        let connection = DBus::new()?;
        let sd = systemd::Systemd::new()?;

        if sd.network_manager_will_run()? {
            if !sd.wait_for_network_manager_to_be_active()? {
                log::error!("NetworkManager failed to start after waiting for it");
                return Err(Error::NetworkManagerDisabled);
            }
        } else {
            return Err(Error::NetworkManagerDisabled);
        }
        connection.ensure_resolv_conf_is_managed()?;
        connection.ensure_network_manager_exists()?;
        connection.nm_version_dns_works()?;
        let manager = NetworkManager {
            connection,
            device: None,
            settings_backup: None,
        };
        Ok(manager)
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let old_settings = self.connection.set_dns(interface_name, servers)?;
        self.settings_backup = Some(old_settings);
        self.device = Some(interface_name.to_string());
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(settings_backup) = self.settings_backup.take() {
            let device = match self.device.take() {
                Some(device) => device,
                None => return Ok(()),
            };
            let device_path = match self.connection.fetch_device(&device) {
                Ok(device_path) => device_path,
                Err(network_manager::Error::DeviceNotFound) => return Ok(()),
                Err(error) => return Err(error.into()),
            };

            if network_manager::device_is_ready(self.connection.get_device_state(&device_path)?) {
                self.connection
                    .reapply_settings(&device_path, settings_backup, 0u64)?;
            }
            return Ok(());
        }
        log::trace!("No DNS settings to reset");
        Ok(())
    }
}
