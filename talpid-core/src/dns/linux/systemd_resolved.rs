use super::RESOLV_CONF_PATH;
use crate::linux::iface_index;
use dbus::{
    arg::RefArg, stdintf::*, BusType, Interface, Member, Message, MessageItem, MessageItemArray,
    Signature,
};
use lazy_static::lazy_static;
use libc::{AF_INET, AF_INET6};
use std::{fs, io, net::IpAddr, path::Path};
use talpid_types::ErrorExt as _;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to initialize a connection to D-Bus")]
    ConnectDBus(#[error(source)] dbus::Error),

    #[error(display = "/etc/resolv.conf is not a symlink to Systemd resolved")]
    NotSymlinkedToResolvConf,

    #[error(display = "Systemd resolved not detected")]
    NoSystemdResolved(#[error(source)] dbus::Error),

    #[error(display = "Failed to read Systemd resolved's resolv.conf")]
    ReadResolvConfFailed(#[error(source)] io::Error),

    #[error(display = "Failed to parse Systemd resolved's resolv.conf")]
    ParseResolvConfFailed(#[error(source)] resolv_conf::ParseError),

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

    #[error(display = "Failed to match the returned D-Bus object with expected type")]
    MatchDBusTypeError(#[error(source)] dbus::arg::TypeMismatchError),
}

lazy_static! {
    static ref RESOLVED_STUB_PATHS: Vec<&'static Path> = vec![
        Path::new("/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/run/systemd/resolve/resolv.conf"),
        Path::new("/var/run/systemd/resolve/stub-resolv.conf"),
        Path::new("/var/run/systemd/resolve/resolv.conf"),
    ];
}


const RESOLVED_BUS: &str = "org.freedesktop.resolve1";
const RPC_TIMEOUT_MS: i32 = 1000;

lazy_static! {
    static ref LINK_INTERFACE: Interface<'static> =
        Interface::from_slice(b"org.freedesktop.resolve1.Link").unwrap();
    static ref MANAGER_INTERFACE: Interface<'static> =
        Interface::from_slice(b"org.freedesktop.resolve1.Manager").unwrap();
    static ref GET_LINK_METHOD: Member<'static> = Member::from_slice(b"GetLink").unwrap();
    static ref SET_DNS_METHOD: Member<'static> = Member::from_slice(b"SetDNS").unwrap();
    static ref SET_DOMAINS_METHOD: Member<'static> = Member::from_slice(b"SetDomains").unwrap();
    static ref REVERT_METHOD: Member<'static> = Member::from_slice(b"Revert").unwrap();
}

pub struct SystemdResolved {
    dbus_connection: dbus::Connection,
    interface_link: Option<(String, dbus::Path<'static>)>,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let result = (|| {
            let dbus_connection =
                dbus::Connection::get_private(BusType::System).map_err(Error::ConnectDBus)?;
            let systemd_resolved = SystemdResolved {
                dbus_connection,
                interface_link: None,
            };

            systemd_resolved.ensure_resolved_exists()?;
            Self::ensure_resolv_conf_is_resolved_symlink()?;
            Ok(systemd_resolved)
        })();

        if let Err(err) = &result {
            log::error!("systemd-resolved is not being used because: {}", err);
        }


        result
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
        if Self::path_is_resolvconf_stub(&link_target) {
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

    fn as_manager_object(&self) -> dbus::ConnPath<'_, &dbus::Connection> {
        self.dbus_connection
            .with_path(RESOLVED_BUS, "/org/freedesktop/resolve1", RPC_TIMEOUT_MS)
    }

    fn as_link_object<'a>(
        &'a self,
        link_object_path: dbus::Path<'a>,
    ) -> dbus::ConnPath<'a, &'a dbus::Connection> {
        self.dbus_connection
            .with_path(RESOLVED_BUS, link_object_path, RPC_TIMEOUT_MS)
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

        let mut reply = self
            .as_manager_object()
            .method_call_with_args(&MANAGER_INTERFACE, &GET_LINK_METHOD, |message| {
                message.append_items(&[MessageItem::Int32(interface_index as i32)]);
            })
            .map_err(Error::DBusRpcError)?;
        reply
            .as_result()
            .map_err(Error::DBusRpcError)?
            .read1()
            .map_err(Error::MatchDBusTypeError)
    }

    fn set_link_dns<'a, 'b: 'a>(
        &'a self,
        link_object_path: &'b dbus::Path<'static>,
        servers: &[IpAddr],
    ) -> Result<()> {
        let server_addresses = build_addresses_argument(servers);
        self.as_link_object(link_object_path.clone())
            .method_call_with_args(&LINK_INTERFACE, &SET_DNS_METHOD, |message| {
                message.append_items(&[server_addresses]);
            })
            .and_then(|mut reply| reply.as_result().map(|_| ()))
            .map_err(Error::DBusRpcError)?;

        // set the search domain to catch all DNS requests, forces the link to be the prefered
        // resolver, otherwise systemd-resolved will use other interfaces to do DNS lookups
        let dns_domains: &[_] = &[(&".", true)];

        let msg = Message::new_method_call(
            RESOLVED_BUS,
            link_object_path as &str,
            &LINK_INTERFACE as &str,
            &SET_DOMAINS_METHOD as &str,
        )
        .expect("failed to construct a new dbus message")
        .append1(dns_domains);

        self.dbus_connection
            .send_with_reply_and_block(msg, RPC_TIMEOUT_MS)
            .and_then(|mut reply| reply.as_result().map(|_| ()))
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

        match link.method_call_with_args(&LINK_INTERFACE, &REVERT_METHOD, |_| {}) {
            Ok(mut reply) => reply.as_result().map(|_| ()),
            Err(error) => {
                if error.name() == Some("org.freedesktop.DBus.Error.UnknownObject") {
                    log::info!(
                        "Not reseting DNS of interface {} because it no longer exists",
                        interface_name
                    );
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }
}

fn build_addresses_argument(addresses: &[IpAddr]) -> MessageItem {
    let addresses = addresses.iter().map(ip_address_to_message_item).collect();

    MessageItem::Array(
        MessageItemArray::new(addresses, Signature::make::<Vec<(i32, Vec<u8>)>>())
            .expect("Invalid construction of DBus array of IP addresses argument"),
    )
}

fn ip_address_to_message_item(address: &IpAddr) -> MessageItem {
    let (protocol, octets) = match address {
        IpAddr::V4(ipv4_address) => (AF_INET, bytes_to_message_item_array(&ipv4_address.octets())),
        IpAddr::V6(ipv6_address) => (
            AF_INET6,
            bytes_to_message_item_array(&ipv6_address.octets()),
        ),
    };

    MessageItem::Struct(vec![
        MessageItem::Int32(protocol),
        MessageItem::Array(octets),
    ])
}

fn bytes_to_message_item_array(bytes: &[u8]) -> MessageItemArray {
    MessageItemArray::new(
        bytes.iter().cloned().map(MessageItem::Byte).collect(),
        Signature::make::<Vec<u8>>(),
    )
    .expect("Invalid construction of DBus array of bytes argument")
}
