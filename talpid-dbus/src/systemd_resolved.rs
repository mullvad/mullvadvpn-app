use std::fs;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::sync::LazyLock;

use libc::AF_INET;
use libc::AF_INET6;
use serde::{Deserialize, Serialize};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::ObjectPath;
use zvariant::OwnedObjectPath;

// TODO: newtype `interface_index`

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize a connection to D-Bus")]
    ConnectDBus(#[source] zbus::Error),

    #[error("Failed to read /etc/resolv.conf: {0}")]
    ReadResolvConfError(#[source] io::Error),

    #[error("/etc/resolv.conf contents do not match systemd-resolved resolv.conf")]
    ResolvConfDiffers,

    #[error("/etc/resolv.conf is not a symlink to Systemd resolved")]
    NotSymlinkedToResolvConf,

    #[error("Static stub file does not point to localhost")]
    StaticStubNotPointingToLocalhost,

    #[error("systemd-resolved not detected")]
    NoSystemdResolved(#[source] zbus::Error),

    #[error("Failed to find link interface in resolved manager")]
    GetLinkError(#[source] zbus::Error),

    #[error("Failed to configure DNS domains")]
    SetDomainsError(#[source] zbus::Error),

    // A Proxy is a helper to interact with an interface on a remote object.
    #[error("Failed to create proxy for object {0}")]
    Proxy(#[source] zbus::Error),

    #[error("Failed to revert DNS settings of interface: {0}")]
    RevertDnsError(String, #[source] zbus::Error),

    #[error("Failed to replace DNS settings")]
    ReplaceDnsError,

    #[error("Failed to perform RPC call on D-Bus")]
    DBusRpcError(#[source] zbus::Error),

    #[error("Async D-Bus task failed")]
    AsyncTaskError(#[source] tokio::task::JoinError),
}

static RESOLVED_STUB_PATHS: LazyLock<Vec<&'static Path>> = LazyLock::new(|| {
    vec![
        Path::new("/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/run/systemd/resolve/resolv.conf"),
        Path::new("/var/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/var/run/systemd/resolve/resolv.conf"),
    ]
});

// TODO: Link to relevant documentation for all of these constants.
const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";
const RESOLVED_BUS: &str = "org.freedesktop.resolve1";

#[derive(Clone)]
pub struct SystemdResolved {
    pub dbus_connection: Connection,
}

#[derive(Clone)]
// TODO: Make proper async
pub struct AsyncHandle {
    dbus_interface: SystemdResolved,
}

impl SystemdResolved {
    pub fn new() -> Result<Self, Error> {
        let dbus_connection = crate::get_connection().map_err(Error::ConnectDBus)?;

        let systemd_resolved = SystemdResolved { dbus_connection };

        systemd_resolved.ensure_resolved_exists()?;
        Self::ensure_resolv_conf_is_resolved_symlink()?;
        Ok(systemd_resolved)
    }

    /// Try to look up the DNS property from SystemdResolved.
    /// If it is set, this function returns Ok(()).
    pub fn ensure_resolved_exists(&self) -> Result<Vec<DnsServer>, Error> {
        self.as_manager_object()?
            .get_property("DNS")
            .map_err(Error::NoSystemdResolved)
    }

    pub fn ensure_resolv_conf_is_resolved_symlink() -> Result<(), Error> {
        match fs::read_link(RESOLV_CONF_PATH) {
            Ok(link_target) => {
                // if /etc/resolv.conf is not symlinked to the stub resolve.conf file , managing DNS
                // through systemd-resolved will not ensure that our resolver is given priority -
                // sometimes this will mean adding 1 and 2 seconds of latency to DNS
                // queries, other times our resolver won't be considered at all. In
                // this case, it's better to fall back to cruder management methods.
                if Self::path_is_resolvconf_stub(&link_target)
                    || Self::resolv_conf_is_static_stub(&link_target)?
                    || Self::ensure_resolvconf_contents().is_ok()
                {
                    Ok(())
                } else {
                    Err(Error::NotSymlinkedToResolvConf)
                }
            }
            // etc/resolv.conf is not a symlink
            Err(err) if err.kind() == io::ErrorKind::InvalidInput => {
                Self::ensure_resolvconf_contents()
            }
            Err(err) => {
                log::trace!("Failed to read /etc/resolv.conf symlink: {}", err);
                Err(Error::NotSymlinkedToResolvConf)
            }
        }
    }

    fn ensure_resolvconf_contents() -> Result<(), Error> {
        let resolv_conf =
            fs::read_to_string(RESOLV_CONF_PATH).map_err(Error::ReadResolvConfError)?;
        if RESOLVED_STUB_PATHS
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .any(|link_contents| link_contents == resolv_conf)
        {
            Ok(())
        } else {
            Err(Error::ResolvConfDiffers)
        }
    }

    fn path_is_resolvconf_stub(link_path: &Path) -> bool {
        // if link path is relative to /etc/resolv.conf, resolve the path and compare it.
        if link_path.is_relative() {
            match Path::new("/etc/").join(link_path).canonicalize() {
                Ok(link_destination) => RESOLVED_STUB_PATHS.contains(&link_destination.as_ref()),
                Err(e) => {
                    log::error!(
                        "Failed to canonicalize resolv conf path {}: {}",
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
    fn resolv_conf_is_static_stub(link_path: &Path) -> Result<bool, Error> {
        const STATIC_STUB_PATH: &str = "/usr/lib/systemd/resolv.conf";
        if link_path == AsRef::<Path>::as_ref(STATIC_STUB_PATH) {
            let points_to_localhost = fs::read_to_string(link_path)
                .map(|contents| {
                    let parts = contents.trim().split(' ');
                    parts
                        .map(str::parse::<IpAddr>)
                        .any(|maybe_ip| maybe_ip.map(|addr| addr.is_loopback()).unwrap_or(false))
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

    /// TODO: Document mee
    fn as_manager_object(&self) -> Result<Proxy<'_>, Error> {
        const RESOLVED_MANAGER_PATH: &str = "/org/freedesktop/resolve1";
        const MANAGER_INTERFACE: &str = "org.freedesktop.resolve1.Manager";
        Proxy::new(
            &self.dbus_connection,
            RESOLVED_BUS,
            RESOLVED_MANAGER_PATH,
            MANAGER_INTERFACE, // TODO: Inline
        )
        .map_err(Error::Proxy)
    }

    /// TODO: Document mee
    fn as_link_object<'a>(&self, link_object_path: &ObjectPath<'a>) -> Result<Proxy<'a>, Error> {
        const LINK_INTERFACE: &str = "org.freedesktop.resolve1.Link";
        Proxy::new(
            &self.dbus_connection,
            RESOLVED_BUS,
            link_object_path,
            LINK_INTERFACE,
        )
        .map_err(Error::Proxy)
    }

    pub fn get_dns(&self, interface_index: i32) -> Result<DnsState, Error> {
        let link_object_path = self.fetch_link(interface_index)?;
        let set_servers = self.get_link_dns(&link_object_path)?;

        Ok(DnsState {
            interface_path: link_object_path,
            interface_index,
            set_servers,
        })
    }

    pub fn set_dns_state(&self, state: DnsState) -> Result<(), Error> {
        self.set_link_dns(&state.interface_path, &state.set_servers)
    }

    pub fn set_dns(&self, interface_index: i32, servers: Vec<IpAddr>) -> Result<DnsState, Error> {
        let servers: Vec<_> = servers.into_iter().map(LinkIpAddr::from).collect();
        let link_object_path = self.fetch_link(interface_index)?;
        self.set_link_dns(&link_object_path, &servers)?;
        // TODO: can this be automatically derived somehow? I.e. return it from `set_link_dns`?
        let dns_state = DnsState {
            interface_path: link_object_path,
            interface_index,
            set_servers: servers,
        };
        Ok(dns_state)
    }

    pub fn get_domains(&self, interface_index: i32) -> Result<Vec<(String, bool)>, Error> {
        let link_object_path = self.fetch_link(interface_index)?;
        self.get_link_dns_domains(&link_object_path)
    }

    pub fn set_domains(&self, interface_index: i32, domains: &[(&str, bool)]) -> Result<(), Error> {
        let link_object_path = self.fetch_link(interface_index)?;
        self.set_link_dns_domains(&link_object_path, domains)
    }

    /// Returns the object path to the `org.freedesktop.resolve1.Link` object corresponding to the
    /// network interface index.
    fn fetch_link(&self, interface_index: i32) -> Result<OwnedObjectPath, Error> {
        // grep `GetLink`:
        // https://manpages.debian.org/bullseye/systemd/org.freedesktop.resolve1.5.en.html
        self.as_manager_object()?
            .call("GetLink", &interface_index)
            .map_err(Error::GetLinkError)
    }

    fn get_link_dns(&self, link_object_path: &ObjectPath<'_>) -> Result<Vec<LinkIpAddr>, Error> {
        self.as_link_object(link_object_path)?
            .get_property("DNS")
            .map_err(Error::DBusRpcError)
    }

    fn set_link_dns(
        &self,
        link_object_path: &ObjectPath<'_>,
        servers: &[LinkIpAddr],
    ) -> Result<(), Error> {
        // TODO: Check if workaround on main is relevant anymore.
        self.as_link_object(link_object_path)?
            .call("SetDNS", &servers)
            .map_err(Error::DBusRpcError)
    }

    fn link_disable_dns_over_tls(&self, interface_index: i32) -> Result<(), Error> {
        // TODO: Can these two calls be consolidated?
        // Yes, most likely. Proxy exposes a `path` function, which would encapsulate
        // link_object_path.
        let link_object_path = self.fetch_link(interface_index)?;
        self.as_link_object(&link_object_path)?
            // TODO: Handle "org.freedesktop.DBus.Error.UnknownMethod" gracefully.
            .call("SetDNSOverTLS", &"no")
            .map_err(Error::DBusRpcError)
    }

    fn get_link_dns_domains(
        &self,
        link_object_path: &ObjectPath<'_>,
    ) -> Result<Vec<(String, bool)>, Error> {
        let domains: Vec<(String, bool)> = self
            .as_link_object(link_object_path)?
            .get_property("Domains")
            .map_err(Error::DBusRpcError)?;
        Ok(domains)
    }

    pub fn revert_link(&mut self, dns_state: &DnsState) -> std::result::Result<(), Error> {
        const REVERT_METHOD: &str = "Revert";
        match self
            .as_link_object(&dns_state.interface_path)?
            .call(REVERT_METHOD, &())
        {
            Ok(()) => Ok(()),
            Err(zbus::Error::FDO(fdo)) => match *fdo {
                zbus::fdo::Error::UnknownObject(_) => todo!(),
                _ => todo!(),
            },
            Err(err) => Err(Error::DBusRpcError(err)),
        }
    }

    /// TODO: Document mee
    fn set_link_dns_domains(
        &self,
        link_object_path: &ObjectPath<'_>,
        domains: &[(&str, bool)],
    ) -> Result<(), Error> {
        const SET_DOMAINS_METHOD: &str = "SetDomains";
        self.as_link_object(link_object_path)?
            .call(SET_DOMAINS_METHOD, &domains)
            .map_err(Error::SetDomainsError)
    }

    pub fn async_handle(&self) -> AsyncHandle {
        AsyncHandle::new(self.clone())
    }
}

// TODO: nuke when the whole module is converted to async-first.
impl AsyncHandle {
    fn new(dbus_interface: SystemdResolved) -> Self {
        Self { dbus_interface }
    }

    pub async fn get_dns(&self, interface_index: i32) -> Result<DnsState, Error> {
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.get_dns(interface_index))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn set_dns_state(&self, state: DnsState) -> Result<(), Error> {
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.set_dns_state(state))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn set_dns(
        &self,
        interface_index: i32,
        servers: Vec<IpAddr>,
    ) -> Result<DnsState, Error> {
        log::info!("Updating DNS: {servers:#?}");
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.set_dns(interface_index, servers))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn disable_dot(&self, interface_index: i32) -> Result<(), Error> {
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.link_disable_dns_over_tls(interface_index))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn set_domains(
        &self,
        interface_index: i32,
        domains: &[(&'static str, bool)],
    ) -> Result<(), Error> {
        let interface = self.dbus_interface.clone();
        let domains = domains.to_vec();
        tokio::task::spawn_blocking(move || interface.set_domains(interface_index, &domains))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn revert_link(&self, state: DnsState) -> Result<(), Error> {
        let mut interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.revert_link(&state))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub fn handle(&self) -> &SystemdResolved {
        &self.dbus_interface
    }
}

#[derive(Debug, Clone, PartialEq, zvariant::Type)]
pub struct DnsState {
    pub interface_path: OwnedObjectPath,
    pub interface_index: i32,
    pub set_servers: Vec<LinkIpAddr>,
}

/// The IP address representation that systemd-resolved communicates back and forth over DBUS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, zvariant::Type, zvariant::Value)]
pub struct LinkIpAddr((i32, Vec<u8>));

impl From<IpAddr> for LinkIpAddr {
    fn from(address: IpAddr) -> LinkIpAddr {
        match address {
            IpAddr::V4(address) => {
                let octects = address.octets();
                LinkIpAddr((AF_INET, octects.to_vec()))
            }
            IpAddr::V6(address) => {
                let octects = address.octets();
                LinkIpAddr((AF_INET6, octects.to_vec()))
            }
        }
    }
}

/// https://manpages.debian.org/bullseye/systemd/org.freedesktop.resolve1.5.en.html:
///
/// DNS contain arrays of all DNS servers currently used by systemd-resolved. DNS contains information similar
/// to the DNS server data in /run/systemd/resolve/resolv.conf.
///
/// Each structure in the array consists of
/// - a numeric network interface index
/// - an address family
/// - a byte array containing the DNS server address (either 4 bytes in length for IPv4 or 16 bytes in lengths for IPv6)
#[derive(Debug, zvariant::Type, zvariant::OwnedValue, zvariant::Value)]
pub struct DnsServer {
    pub iface_index: i32,
    pub address_family: i32, // TODO: c_int
    pub address: DnsAddress,
}

#[derive(Debug, zvariant::Type, zvariant::Value)]
pub struct DnsAddress(Vec<u8>);

impl TryFrom<DnsAddress> for IpAddr {
    type Error = zvariant::Error;
    fn try_from(value: DnsAddress) -> Result<Self, Self::Error> {
        let bytes = value.0;
        // chop of the leading i32. systemd-resolved be like that.
        let Some((_, octets)) = bytes.as_slice().split_first_chunk::<4>() else {
            return Err(zvariant::Error::IncorrectType);
        };
        match octets.len() {
            4 => {
                let octets = octets.first_chunk::<4>().unwrap();
                Ok(IpAddr::V4(Ipv4Addr::from(*octets)))
            }
            16 => {
                let octets = octets.first_chunk::<16>().unwrap();
                Ok(IpAddr::V6(Ipv6Addr::from(*octets)))
            }
            _ => Err(zvariant::Error::OutOfBounds),
        }
    }
}
