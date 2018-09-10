extern crate dbus;

use std::net::IpAddr;

use libc::{AF_INET, AF_INET6};

use self::dbus::arg::RefArg;
use self::dbus::stdintf::*;
use self::dbus::{BusType, Interface, Member, MessageItem, MessageItemArray, Signature};

use super::super::iface_index;

error_chain! {
    errors {
        NoSystemdResolved {
            description("Systemd resolved not detected")
        }
        InvalidInterface {
            description("Invalid interface to configure DNS settings")
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
        RevertDnsError {
            description("Failed to revert DNS configuration")
        }
    }

    foreign_links {
        DbusError(dbus::Error);
    }
}

const RESOLVED_BUS: &str = "org.freedesktop.resolve1";
const LINK_INTERFACE: &str = "org.freedesktop.resolve1.Link";
const MANAGER_INTERFACE: &str = "org.freedesktop.resolve1.Manager";
const RPC_TIMEOUT_MS: i32 = 1000;

pub struct SystemdResolved {
    dbus_connection: dbus::Connection,
    interface_link: Option<dbus::Path<'static>>,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_connection = dbus::Connection::get_private(BusType::System)?;
        let systemd_resolved = SystemdResolved {
            dbus_connection,
            interface_link: None,
        };

        systemd_resolved.ensure_resolved_exists()?;

        Ok(systemd_resolved)
    }

    fn ensure_resolved_exists(&self) -> Result<()> {
        let _: Box<RefArg> = self
            .as_manager_object()
            .get(MANAGER_INTERFACE, "DNS")
            .chain_err(|| ErrorKind::NoSystemdResolved)?;

        Ok(())
    }

    fn as_manager_object<'a>(&'a self) -> dbus::ConnPath<'a, &'a dbus::Connection> {
        self.dbus_connection
            .with_path(RESOLVED_BUS, "/org/freedesktop/resolve1", RPC_TIMEOUT_MS)
    }

    fn as_link_object<'a>(&'a self) -> Option<dbus::ConnPath<'a, &'a dbus::Connection>> {
        self.interface_link.as_ref().map(|link_object_path| {
            self.dbus_connection
                .with_path(RESOLVED_BUS, link_object_path.clone(), RPC_TIMEOUT_MS)
        })
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: Vec<IpAddr>) -> Result<()> {
        self.fetch_link(interface_name)?;

        let interface = Interface::from_slice(LINK_INTERFACE.as_bytes()).unwrap();
        let method = Member::from_slice("SetDNS".as_bytes()).unwrap();

        let server_addresses = build_addresses_argument(servers);

        let mut reply = self
            .as_link_object()
            .expect("fetch_link succeeded without configuring link object path")
            .method_call_with_args(&interface, &method, |message| {
                message.append_items(&[server_addresses]);
            }).chain_err(|| ErrorKind::DbusRpcError)?;

        reply
            .as_result()
            .map(|_| ())
            .chain_err(|| ErrorKind::SetDnsError)
    }

    fn fetch_link(&mut self, interface_name: &str) -> Result<()> {
        let interface = Interface::from_slice(MANAGER_INTERFACE.as_bytes()).unwrap();
        let method = Member::from_slice("GetLink".as_bytes()).unwrap();

        let interface_index =
            iface_index(interface_name).chain_err(|| ErrorKind::InvalidInterface)?;

        let mut reply = self
            .as_manager_object()
            .method_call_with_args(&interface, &method, |message| {
                message.append_items(&[MessageItem::Int32(interface_index as i32)]);
            }).chain_err(|| ErrorKind::DbusRpcError)?;

        let result = reply.as_result().chain_err(|| ErrorKind::GetLinkError)?;

        self.interface_link = Some(result.read1().chain_err(|| ErrorKind::GetLinkError)?);

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(link) = self.as_link_object() {
            let interface = Interface::from_slice(LINK_INTERFACE.as_bytes()).unwrap();
            let method = Member::from_slice("Revert".as_bytes()).unwrap();

            match link.method_call_with_args(&interface, &method, |_| {}) {
                Ok(mut reply) => reply
                    .as_result()
                    .map(|_| ())
                    .chain_err(|| ErrorKind::RevertDnsError),
                Err(error) => if error.name() == Some("org.freedesktop.DBus.Error.UnknownObject") {
                    warn!("Not reseting DNS because interface link no longer exists");
                    Ok(())
                } else {
                    Err(error).chain_err(|| ErrorKind::RevertDnsError)
                },
            }
        } else {
            Ok(())
        }
    }
}

fn build_addresses_argument(addresses: Vec<IpAddr>) -> MessageItem {
    let addresses = addresses
        .into_iter()
        .map(ip_address_to_message_item)
        .collect();

    MessageItem::Array(
        MessageItemArray::new(addresses, Signature::make::<Vec<(i32, Vec<u8>)>>())
            .expect("Invalid construction of DBus array of IP addresses argument"),
    )
}

fn ip_address_to_message_item(address: IpAddr) -> MessageItem {
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
        bytes.into_iter().cloned().map(MessageItem::Byte).collect(),
        Signature::make::<Vec<u8>>(),
    ).expect("Invalid construction of DBus array of bytes argument")
}
