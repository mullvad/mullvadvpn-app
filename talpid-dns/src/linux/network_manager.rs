use std::net::IpAddr;
pub use talpid_dbus::network_manager::Error;
use talpid_dbus::network_manager::{self, DeviceConfig, NetworkManagerConnection as DBus};

pub type Result<T> = std::result::Result<T, Error>;

pub struct NetworkManager {
    pub connection: DBus,
    /// Network manager device (network interface).
    device: Option<String>,
    settings_backup: Option<DeviceConfig>,
}

impl NetworkManager {
    pub async fn new() -> Result<Self> {
        let connection = DBus::new().await?;
        connection.ensure_can_be_used_to_manage_dns().await?;
        let manager = NetworkManager {
            connection,
            device: None,
            settings_backup: None,
        };
        Ok(manager)
    }

    pub async fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let old_settings = self.connection.set_dns(interface_name, servers).await?;
        self.settings_backup = Some(old_settings);
        self.device = Some(interface_name.to_string());
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        let Some(settings_backup) = self.settings_backup.take() else {
            log::trace!("No DNS settings to reset");
            return Ok(());
        };

        let Some(device) = self.device.take() else {
            log::trace!("No NetworkManager device set");
            return Ok(());
        };
        let device_path = match self.connection.fetch_device(&device).await {
            Ok(device_path) => device_path,
            Err(Error::DeviceNotFound) => {
                log::debug!("NetworkManager device ({device}) does not exist");
                return Ok(());
            }
            Err(error) => return Err(error),
        };

        if network_manager::device_is_ready(
            self.connection
                .get_device_state(device_path.clone())
                .await?,
        ) {
            self.connection
                .reapply(device_path, settings_backup)
                .await?;
        }
        Ok(())
    }
}
