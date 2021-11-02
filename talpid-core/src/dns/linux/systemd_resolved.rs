use crate::{
    linux::{iface_index, IfaceIndexLookupError},
    routing::RouteManagerHandle,
};
use std::net::{IpAddr, Ipv4Addr};
use talpid_dbus::systemd_resolved::{AsyncHandle, SystemdResolved as DbusInterface};
use talpid_types::ErrorExt;

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "systemd-resolved operation failed")]
    SystemdResolvedError(#[error(source)] SystemdDbusError),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceNameError(#[error(source)] IfaceIndexLookupError),
}

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    tunnel_index: u32,
}

const RELAY_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 64, 0, 1));
const RELAY_IP_2: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 64, 0, 2));


impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

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

        let non_local_servers = servers
            .iter()
            .cloned()
            .filter(|ip| {
                !crate::firewall::is_local_address(ip) || *ip == RELAY_IP || *ip == RELAY_IP_2
            })
            .collect::<Vec<_>>();

        if non_local_servers.is_empty() {
            return Ok(());
        }

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
            .set_dns(self.tunnel_index, non_local_servers)
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
