use super::RESOLV_CONF_PATH;
use crate::linux::iface_index;
use dbus::{
    arg::RefArg,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection},
};
use lazy_static::lazy_static;
use libc::{AF_INET, AF_INET6};
use std::{fs, net::IpAddr, path::Path, sync::Arc, time::Duration};
use talpid_types::ErrorExt as _;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to initialize a connection to D-Bus")]
    ConnectDBus(#[error(source)] dbus::Error),

    #[error(display = "/etc/resolv.conf is not a symlink to Systemd resolved")]
    NotSymlinkedToResolvConf,

    #[error(display = "Static stub file does not point to localhost")]
    StaticStubNotPointingToLocalhost,

    #[error(display = "Systemd resolved not detected")]
    NoSystemdResolved(#[error(source)] dbus::Error),

    #[error(display = "Invalid network interface name")]
    InvalidInterfaceName(#[error(source)] crate::linux::IfaceIndexLookupError),

    #[error(display = "Failed to find link interface in resolved manager")]
    GetLinkError(#[error(source)] Box<Error>),

    #[error(display = "Failed to configure DNS domains")]
    SetDomainsError(#[error(source)] dbus::Error),

    #[error(display = "Failed to revert DNS settings of interface: {}", _0)]
    RevertDnsError(String, #[error(source)] dbus::Error),

    #[error(display = "Failed to perform RPC call on D-Bus")]
    DBusRpcError(#[error(source)] dbus::Error),
}

lazy_static! {
    static ref RESOLVED_STUB_PATHS: Vec<&'static Path> = vec![
        Path::new("/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/run/systemd/resolve/resolv.conf"),
        Path::new("/var/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/var/run/systemd/resolve/resolv.conf"),
    ];
}

const STATIC_STUB_PATH: &str = "/usr/lib/systemd/resolv.conf";

const RESOLVED_BUS: &str = "org.freedesktop.resolve1";
const RPC_TIMEOUT: Duration = Duration::from_secs(1);

const LINK_INTERFACE: &str = "org.freedesktop.resolve1.Link";
const MANAGER_INTERFACE: &str = "org.freedesktop.resolve1.Manager";
const GET_LINK_METHOD: &str = "GetLink";
const SET_DNS_METHOD: &str = "SetDNS";
const SET_DOMAINS_METHOD: &str = "SetDomains";
const REVERT_METHOD: &str = "Revert";

pub struct SystemdResolved {
    pub dbus_connection: Arc<SyncConnection>,
    interface_link: Option<(String, dbus::Path<'static>)>,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_connection = crate::linux::dbus::get_connection().map_err(Error::ConnectDBus)?;

        let systemd_resolved = SystemdResolved {
            dbus_connection,
            interface_link: None,
        };

        systemd_resolved.ensure_resolved_exists()?;
        Self::ensure_resolv_conf_is_resolved_symlink()?;
        Ok(systemd_resolved)
    }

    fn ensure_resolved_exists(&self) -> Result<()> {
        let _: Box<dyn RefArg> = self
            .as_manager_object()
            .get(&MANAGER_INTERFACE, "DNS")
            .map_err(Error::NoSystemdResolved)?;

        Ok(())
    }

    fn ensure_resolv_conf_is_resolved_symlink() -> Result<()> {
        let link_target =
            fs::read_link(RESOLV_CONF_PATH).map_err(|_| Error::NotSymlinkedToResolvConf)?;

        // if /etc/resolv.conf is not symlinked to the stub resolve.conf file , managing DNS
        // through systemd-resolved will not ensure that our resolver is given priority - sometimes
        // this will mean adding 1 and 2 seconds of latency to DNS queries, other times our
        // resolver won't be considered at all. In this case, it's better to fall back to cruder
        // management methods.
        if Self::path_is_resolvconf_stub(&link_target)
            || Self::resolv_conf_is_static_stub(&link_target)?
        {
            Ok(())
        } else {
            Err(Error::NotSymlinkedToResolvConf)
        }
    }

    fn path_is_resolvconf_stub(link_path: &Path) -> bool {
        // if link path is relative to /etc/resolv.conf, resolve the path and compare it.
        if link_path.is_relative() {
            match Path::new("/etc/").join(link_path).canonicalize() {
                Ok(link_destination) => RESOLVED_STUB_PATHS.contains(&link_destination.as_ref()),
                Err(e) => {
                    log::error!(
                        "Failed to canonicalize resolv conf path {} - {}",
                        link_path.display(),
                        e
                    );
                    false
                }
            }
        } else {
            RESOLVED_STUB_PATHS.contains(&link_path)
        }
    }

    /// Checks if path is pointing to the systemd-resolved _static_ resolv.conf file. If it's not,
    /// it returns false, otherwise it checks whether the static stub file points to the local
    /// resolver. If not, the file has been _meddled_ with, so we can't trust it.
    fn resolv_conf_is_static_stub(link_path: &Path) -> Result<bool> {
        if link_path == &STATIC_STUB_PATH.as_ref() {
            let points_to_localhost = fs::read_to_string(link_path)
                .map(|contents| {
                    let parts = contents.trim().split(' ');
                    parts
                        .map(str::parse::<IpAddr>)
                        .map(|maybe_ip| maybe_ip.map(|addr| addr.is_loopback()).unwrap_or(false))
                        .any(|is_loopback| is_loopback)
                })
                .unwrap_or(false);

            if points_to_localhost {
                Ok(true)
            } else {
                Err(Error::StaticStubNotPointingToLocalhost)
            }
        } else {
            Ok(false)
        }
    }


    fn as_manager_object(&self) -> Proxy<'_, &SyncConnection> {
        Proxy::new(
            RESOLVED_BUS,
            "/org/freedesktop/resolve1",
            RPC_TIMEOUT,
            &self.dbus_connection,
        )
    }

    fn as_link_object<'a>(
        &'a self,
        link_object_path: dbus::Path<'a>,
    ) -> Proxy<'a, &'a SyncConnection> {
        Proxy::new(
            RESOLVED_BUS,
            link_object_path,
            RPC_TIMEOUT,
            &self.dbus_connection,
        )
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let link_object_path = self
            .fetch_link(interface_name)
            .map_err(|e| Error::GetLinkError(Box::new(e)))?;
        if let Err(e) = self.reset() {
            log::debug!(
                "Failed to reset previous DNS settings - {}",
                e.display_chain()
            );
        }

        self.set_link_dns(&link_object_path, servers)?;
        self.interface_link = Some((interface_name.to_string(), link_object_path));

        Ok(())
    }

    fn fetch_link(&self, interface_name: &str) -> Result<dbus::Path<'static>> {
        let interface_index = iface_index(interface_name).map_err(Error::InvalidInterfaceName)?;

        self.as_manager_object()
            .method_call(
                MANAGER_INTERFACE,
                GET_LINK_METHOD,
                (interface_index as i32,),
            )
            .map_err(Error::DBusRpcError)
            .map(|result: (dbus::Path<'static>,)| result.0)
    }

    fn set_link_dns<'a, 'b: 'a>(
        &'a self,
        link_object_path: &'b dbus::Path<'static>,
        servers: &[IpAddr],
    ) -> Result<()> {
        let servers = servers
            .iter()
            .map(|addr| (ip_version(addr), ip_to_bytes(addr)))
            .collect::<Vec<_>>();
        self.as_link_object(link_object_path.clone())
            .method_call(LINK_INTERFACE, SET_DNS_METHOD, (servers,))
            .map_err(Error::DBusRpcError)?;

        // set the search domain to catch all DNS requests, forces the link to be the prefered
        // resolver, otherwise systemd-resolved will use other interfaces to do DNS lookups
        let dns_domains: &[_] = &[(&".", true)];

        Proxy::new(
            RESOLVED_BUS,
            link_object_path,
            RPC_TIMEOUT,
            &*self.dbus_connection,
        )
        .method_call(LINK_INTERFACE, SET_DOMAINS_METHOD, (dns_domains,))
        .map_err(Error::SetDomainsError)
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some((interface_name, link_object_path)) = self.interface_link.take() {
            self.revert_link(link_object_path, &interface_name)
                .map_err(|e| Error::RevertDnsError(interface_name.to_owned(), e))
        } else {
            log::trace!("No DNS settings to reset");
            Ok(())
        }
    }

    fn revert_link(
        &mut self,
        link_object_path: dbus::Path<'static>,
        interface_name: &str,
    ) -> std::result::Result<(), dbus::Error> {
        let link = self.as_link_object(link_object_path);

        if let Err(error) = link.method_call::<(), _, _, _>(LINK_INTERFACE, REVERT_METHOD, ()) {
            if error.name() == Some("org.freedesktop.DBus.Error.UnknownObject") {
                log::trace!(
                    "Not resetting DNS of interface {} because it no longer exists",
                    interface_name
                );
                Ok(())
            } else {
                Err(error)
            }
        } else {
            Ok(())
        }
    }
}

fn ip_version(address: &IpAddr) -> i32 {
    match address {
        IpAddr::V4(_) => AF_INET,
        IpAddr::V6(_) => AF_INET6,
    }
}

fn ip_to_bytes(address: &IpAddr) -> Vec<u8> {
    match address {
        IpAddr::V4(v4_address) => v4_address.octets().to_vec(),
        IpAddr::V6(v6_address) => v6_address.octets().to_vec(),
    }
}
