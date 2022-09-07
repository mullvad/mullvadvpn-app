use crate::{
    linux::{iface_index, IfaceIndexLookupError},
    routing::RouteManagerHandle,
};
use std::net::IpAddr;
use talpid_dbus::{
    systemd,
    systemd_resolved::{AsyncHandle, SystemdResolved as DbusInterface},
};
use talpid_types::ErrorExt;

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "systemd-resolved operation failed")]
    SystemdResolved(#[error(source)] SystemdDbusError),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceName(#[error(source)] IfaceIndexLookupError),

    #[error(display = "Systemd DBus error")]
    Systemd(#[error(source)] systemd::Error),

    #[error(display = "systemd-resolved is disabled")]
    SystemdResolvedDisabled,
}

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    tunnel_index: u32,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();
        let sd = systemd::Systemd::new()?;
        if sd.systemd_resolved_will_run()? {
            if !sd.wait_for_systemd_resolved_to_be_active()? {
                log::error!("systemd-resolved failed to start after waiting for it");
                return Err(Error::SystemdResolvedDisabled);
            }
        } else {
            return Err(Error::SystemdResolvedDisabled);
        }

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            tunnel_index: 0,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(
        &mut self,
        _route_manager: RouteManagerHandle,
        interface_name: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        let tunnel_index = iface_index(interface_name)?;
        self.tunnel_index = tunnel_index;

        if let Err(error) = self.dbus_interface.disable_dot(self.tunnel_index).await {
            log::error!("Failed to disable DoT: {}", error.display_chain());
        }

        if let Err(error) = self
            .dbus_interface
            .set_domains(tunnel_index, &[(".", true)])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, servers.to_vec())
            .await?;

        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        if let Err(error) = self
            .dbus_interface
            .set_domains(self.tunnel_index, &[])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, vec![])
            .await?;

        Ok(())
    }
}
