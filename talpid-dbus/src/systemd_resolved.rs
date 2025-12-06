use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::Path;
use std::sync::LazyLock;

use serde::Serialize;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::ObjectPath;
use zvariant::OwnedObjectPath;

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

    #[error("Systemd resolved not detected")]
    NoSystemdResolved(#[source] zbus::Error),

    #[error("Failed to find link interface in resolved manager")]
    GetLinkError(#[source] zbus::Error),

    #[error("Failed to configure DNS domains")]
    SetDomainsError(#[source] zbus::Error),

    // A Proxy is a helper to interact with an interface on a remote object.
    #[error("Failed to create proxy for object {0}")]
    Proxy(#[source] zbus::Error),

    #[error("Failed to revert DNS settings of interface: {0}")]
    RevertDnsError(String, #[source] dbus::Error),

    #[error("Failed to replace DNS settings")]
    ReplaceDnsError,

    #[error("Failed to perform RPC call on D-Bus")]
    DBusRpcError(#[source] zbus::Error),

    #[error("Failed to add a match to listen for DNS config updates")]
    DnsUpdateMatchError(#[source] dbus::Error),

    #[error("Failed to remove a match for DNS config updates")]
    DnsUpdateRemoveMatchError(#[source] dbus::Error),

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
        let dbus_connection = crate::get_connection_zbus().map_err(Error::ConnectDBus)?;

        let systemd_resolved = SystemdResolved { dbus_connection };

        systemd_resolved.ensure_resolved_exists()?;
        Self::ensure_resolv_conf_is_resolved_symlink()?;
        Ok(systemd_resolved)
    }

    pub fn new_connection() -> Result<Self, Error> {
        let dbus_connection = Connection::system().map_err(Error::ConnectDBus)?;
        let systemd_resolved = SystemdResolved { dbus_connection };

        systemd_resolved.ensure_resolved_exists()?;
        Self::ensure_resolv_conf_is_resolved_symlink()?;
        Ok(systemd_resolved)
    }

    /// Try to look up the DNS property from SystemdResolved.
    /// If it is set, this function returns Ok(()).
    pub fn ensure_resolved_exists(&self) -> Result<DnsServer, Error> {
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

    pub fn get_dns(&self, interface_index: u32) -> Result<DnsState, Error> {
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

    pub fn set_dns(&self, interface_index: u32, servers: Vec<IpAddr>) -> Result<DnsState, Error> {
        let servers: Vec<_> = servers.into_iter().map(LinkIpAddr).collect();
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

    pub fn get_domains(&self, interface_index: u32) -> Result<Vec<(String, bool)>, Error> {
        let link_object_path = self.fetch_link(interface_index)?;
        self.get_link_dns_domains(&link_object_path)
    }

    pub fn set_domains(&self, interface_index: u32, domains: &[(&str, bool)]) -> Result<(), Error> {
        let link_object_path = self.fetch_link(interface_index)?;
        self.set_link_dns_domains(&link_object_path, domains)
    }

    /// Returns the object path to the `org.freedesktop.resolve1.Link` object corresponding to the
    /// network interface index.
    //fn fetch_link(&self, interface_index: u32) -> Result<OwnedObjectPath, Error> {
    fn fetch_link(&self, interface_index: u32) -> Result<OwnedObjectPath, Error> {
        // grep `GetLink`:
        // https://manpages.debian.org/bullseye/systemd/org.freedesktop.resolve1.5.en.html
        self.as_manager_object()?
            .call("GetLink", &interface_index)
            .map_err(Error::GetLinkError)
    }

    fn get_link_dns(&self, link_object_path: &ObjectPath<'_>) -> Result<Vec<LinkIpAddr>, Error> {
        const DNS_SERVERS: &str = "DNS";
        self.as_link_object(link_object_path)?
            .get_property(DNS_SERVERS)
            .map_err(Error::GetLinkError)
    }

    fn set_link_dns(
        &self,
        link_object_path: &ObjectPath<'_>,
        servers: &[LinkIpAddr],
    ) -> Result<(), Error> {
        const SET_DNS_METHOD: &str = "SetDNS";
        let link_object = self.as_link_object(link_object_path)?;
        // TODO: Check if workaround on main is relevant anymore.
        link_object
            .call(SET_DNS_METHOD, &servers)
            .map_err(Error::DBusRpcError)
    }

    fn link_disable_dns_over_tls(&self, interface_index: u32) -> Result<(), Error> {
        const SET_DNS_OVER_TLS_METHOD: &str = "SetDNSOverTLS";
        // TODO: Can these two calls be consolidated?
        // Yes, most likely. Proxy exposes a `path` function, which would encapsulate
        // link_object_path.
        let link_object_path = self.fetch_link(interface_index)?;
        self.as_link_object(&link_object_path)?
            // TODO: Handle "org.freedesktop.DBus.Error.UnknownMethod" gracefully.
            .call(SET_DNS_OVER_TLS_METHOD, &"no")
            .map_err(Error::DBusRpcError)
    }

    fn get_link_dns_domains(
        &self,
        link_object_path: &ObjectPath<'_>,
    ) -> Result<Vec<(String, bool)>, Error> {
        const DNS_DOMAINS: &str = "Domains";
        let domains: Vec<(String, bool)> = self
            .as_link_object(link_object_path)?
            .get_property(DNS_DOMAINS)
            .map_err(Error::DBusRpcError)?;
        Ok(domains)
    }

    pub fn revert_link(&mut self, dns_state: &DnsState) -> std::result::Result<(), Error> {
        const REVERT_METHOD: &str = "Revert";
        self.as_link_object(&dns_state.interface_path)?
            // TODO: Handle "org.freedesktop.DBus.Error.UnknownObject" gracefully.
            .call(REVERT_METHOD, &())
            .map_err(Error::DBusRpcError)
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

    pub async fn get_dns(&self, interface_index: u32) -> Result<DnsState, Error> {
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
        interface_index: u32,
        servers: Vec<IpAddr>,
    ) -> Result<DnsState, Error> {
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.set_dns(interface_index, servers))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn disable_dot(&self, interface_index: u32) -> Result<(), Error> {
        let interface = self.dbus_interface.clone();
        tokio::task::spawn_blocking(move || interface.link_disable_dns_over_tls(interface_index))
            .await
            .map_err(Error::AsyncTaskError)?
    }

    pub async fn set_domains(
        &self,
        interface_index: u32,
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
    pub interface_index: u32,
    pub set_servers: Vec<LinkIpAddr>,
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
#[derive(Debug, zvariant::Type, zvariant::OwnedValue)]
pub struct DnsServer {
    pub iface_index: i32,
    pub address_family: i32, // TODO: c_int
    pub address: LinkIpAddr,
}

/// TODO: Document me.
#[derive(Debug, Clone, PartialEq, zvariant::Type, Serialize)]
pub struct LinkIpAddr(IpAddr);

impl From<LinkIpAddr> for zvariant::Value<'_> {
    fn from(value: LinkIpAddr) -> Self {
        let value = match value.0 {
            IpAddr::V4(address) => zvariant::Array::from(address.octets().as_slice()),
            IpAddr::V6(address) => zvariant::Array::from(address.octets().as_slice()),
        };
        zvariant::Value::Array(value)
    }
}

impl TryFrom<zvariant::Value<'_>> for LinkIpAddr {
    type Error = zvariant::Error;
    fn try_from(value: zvariant::Value<'_>) -> Result<Self, Self::Error> {
        let zvariant::Value::Array(bytes) = value else {
            return Err(zvariant::Error::IncorrectType);
        };

        let ctxt = zvariant::serialized::Context::new_dbus(zvariant::NATIVE_ENDIAN, 0);
        match bytes.len() {
            4 => {
                let mut ipv4_bytes = [0u8; 4];
                let bytes = zvariant::to_bytes(ctxt, &bytes).unwrap();
                ipv4_bytes.copy_from_slice(&bytes);
                let ipv4 = IpAddr::from(ipv4_bytes);
                Ok(LinkIpAddr(ipv4))
            }
            16 => {
                let mut ipv6_bytes = [0u8; 16];
                let bytes = zvariant::to_bytes(ctxt, &bytes).unwrap();
                ipv6_bytes.copy_from_slice(&bytes);
                let ipv6 = IpAddr::from(ipv6_bytes);
                Ok(LinkIpAddr(ipv6))
            }
            _ => Err(zvariant::Error::OutOfBounds),
        }
    }
}
