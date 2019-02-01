use super::RESOLV_CONF_PATH;
use crate::linux::iface_index;
use dbus::{
    arg::RefArg, stdintf::*, BusType, Interface, Member, Message, MessageItem, MessageItemArray,
    Signature,
};
use error_chain::ChainedError;
use lazy_static::lazy_static;
use libc::{AF_INET, AF_INET6};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::Path,
};


error_chain! {
    errors {
        NoSystemdResolved {
            description("Systemd resolved not detected")
        }
        InvalidInterfaceName {
            description("Invalid network interface name")
        }
        DbusRpcError {
            description("Failed to perform RPC call on DBus")
        }
        GetLinkError {
            description("Failed to find link interface in resolved manager")
        }
        SetDnsError {
            description("Failed to configure DNS servers")
        }
        SetDomainsError {
            description("Failed to configure DNS domains")
        }
        RevertDnsError {
            description("Failed to revert DNS configuration")
        }
        DBusError {
            description("Failed to initialize a connection to dbus")
        }
    }
}

const DYNAMIC_RESOLV_CONF_PATH: &str = "/run/systemd/resolve/resolv.conf";
const RESOLVED_DNS_SERVER_ADDRESS: [u8; 4] = [127, 0, 0, 53];

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
        let dbus_connection =
            dbus::Connection::get_private(BusType::System).chain_err(|| ErrorKind::DBusError)?;
        let systemd_resolved = SystemdResolved {
            dbus_connection,
            interface_link: None,
        };

        SystemdResolved::ensure_resolved_is_active()?;
        systemd_resolved.ensure_resolved_exists()?;

        Ok(systemd_resolved)
    }

    fn ensure_resolved_exists(&self) -> Result<()> {
        let _: Box<RefArg> = self
            .as_manager_object()
            .get(&MANAGER_INTERFACE, "DNS")
            .chain_err(|| ErrorKind::NoSystemdResolved)?;

        Ok(())
    }

    fn ensure_resolved_is_active() -> Result<()> {
        ensure!(
            Self::resolv_conf_is_resolved_symlink() || Self::resolv_conf_has_resolved_dns()?,
            ErrorKind::NoSystemdResolved
        );

        Ok(())
    }

    fn resolv_conf_is_resolved_symlink() -> bool {
        fs::read_link(RESOLV_CONF_PATH)
            .map(|resolv_conf_target| resolv_conf_target == Path::new(DYNAMIC_RESOLV_CONF_PATH))
            .unwrap_or_else(|_| false)
    }

    fn resolv_conf_has_resolved_dns() -> Result<bool> {
        let resolv_conf_contents =
            fs::read_to_string(RESOLV_CONF_PATH).chain_err(|| ErrorKind::NoSystemdResolved)?;
        let parsed_resolv_conf = resolv_conf::Config::parse(resolv_conf_contents)
            .chain_err(|| ErrorKind::NoSystemdResolved)?;
        let resolved_dns_server =
            resolv_conf::ScopedIp::V4(Ipv4Addr::from(RESOLVED_DNS_SERVER_ADDRESS));

        Ok(parsed_resolv_conf
            .nameservers
            .into_iter()
            .any(|nameserver| nameserver == resolved_dns_server))
    }

    fn as_manager_object(&self) -> dbus::ConnPath<&dbus::Connection> {
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
        let link_object_path = self.fetch_link(interface_name)?;
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
        let interface_index =
            iface_index(interface_name).chain_err(|| ErrorKind::InvalidInterfaceName)?;

        let mut reply = self
            .as_manager_object()
            .method_call_with_args(&MANAGER_INTERFACE, &GET_LINK_METHOD, |message| {
                message.append_items(&[MessageItem::Int32(interface_index as i32)]);
            })
            .chain_err(|| ErrorKind::DbusRpcError)?;

        let result = reply.as_result().chain_err(|| ErrorKind::GetLinkError)?;

        result.read1().chain_err(|| ErrorKind::GetLinkError)
    }

    fn set_link_dns<'a, 'b: 'a>(
        &'a self,
        link_object_path: &'b dbus::Path<'static>,
        servers: &[IpAddr],
    ) -> Result<()> {
        let server_addresses = build_addresses_argument(servers);

        let mut reply = self
            .as_link_object(link_object_path.clone())
            .method_call_with_args(&LINK_INTERFACE, &SET_DNS_METHOD, |message| {
                message.append_items(&[server_addresses]);
            })
            .chain_err(|| ErrorKind::DbusRpcError)?;

        reply
            .as_result()
            .map(|_| ())
            .chain_err(|| ErrorKind::SetDnsError)?;

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

        let mut reply = self
            .dbus_connection
            .send_with_reply_and_block(msg, RPC_TIMEOUT_MS)
            .chain_err(|| ErrorKind::SetDomainsError)?;
        reply
            .as_result()
            .map(|_| ())
            .chain_err(|| ErrorKind::SetDomainsError)
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some((interface_name, link_object_path)) = self.interface_link.take() {
            self.revert_link(link_object_path, &interface_name)
                .chain_err(|| {
                    format!(
                        "Failed to revert DNS settings of interface: {}",
                        interface_name
                    )
                })?;
        } else {
            log::trace!("No DNS settings to reset");
        };
        Ok(())
    }

    fn revert_link(
        &mut self,
        link_object_path: dbus::Path<'static>,
        interface_name: &str,
    ) -> Result<()> {
        let link = self.as_link_object(link_object_path);

        match link.method_call_with_args(&LINK_INTERFACE, &REVERT_METHOD, |_| {}) {
            Ok(mut reply) => reply
                .as_result()
                .map(|_| ())
                .chain_err(|| ErrorKind::RevertDnsError),
            Err(error) => {
                if error.name() == Some("org.freedesktop.DBus.Error.UnknownObject") {
                    log::info!(
                        "Not reseting DNS of interface {} because it no longer exists",
                        interface_name
                    );
                    Ok(())
                } else {
                    Err(error).chain_err(|| ErrorKind::DbusRpcError)
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
