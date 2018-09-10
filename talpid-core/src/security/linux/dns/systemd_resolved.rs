extern crate dbus;

use std::net::IpAddr;

use libc::{AF_INET6, AF_INET};

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
        SetDnsRpcError {
            description("Failed to perform RPC call to configure DNS servers")
        }
        SetDnsError {
            description("Failed to configure DNS servers")
        }
    }

    foreign_links {
        DbusError(dbus::Error);
    }
}

const MANAGER_INTERFACE: &str = "org.freedesktop.resolve1.Manager";
const RPC_TIMEOUT_MS: i32 = 5000;

pub struct SystemdResolved {
    dbus_connection: dbus::Connection,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_connection = dbus::Connection::get_private(BusType::System)?;
        let systemd_resolved = SystemdResolved { dbus_connection };

        systemd_resolved.ensure_resolved_exists()?;

        Ok(systemd_resolved)
    }

    fn ensure_resolved_exists(&self) -> Result<()> {
        let _: Box<RefArg> = self
            .resolved_path()
            .get(MANAGER_INTERFACE, "DNS")
            .chain_err(|| ErrorKind::NoSystemdResolved)?;

        Ok(())
    }

    fn resolved_path<'a>(&'a self) -> dbus::ConnPath<'a, &'a dbus::Connection> {
        self.dbus_connection.with_path(
            "org.freedesktop.resolve1",
            "/org/freedesktop/resolve1",
            RPC_TIMEOUT_MS,
        )
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: Vec<IpAddr>) -> Result<()> {
        let interface = Interface::from_slice(MANAGER_INTERFACE.as_bytes()).unwrap();
        let method = Member::from_slice("SetLinkDNS".as_bytes()).unwrap();

        let interface_index =
            iface_index(interface_name).chain_err(|| ErrorKind::InvalidInterface)? as i32;
        let server_addresses = build_addresses_argument(servers);

        let mut reply = self
            .resolved_path()
            .method_call_with_args(&interface, &method, |message| {
                message.append_items(&[MessageItem::Int32(interface_index), server_addresses]);
            }).chain_err(|| ErrorKind::SetDnsRpcError)?;

        reply
            .as_result()
            .map(|_| ())
            .chain_err(|| ErrorKind::SetDnsError)
    }

    pub fn reset(&mut self) -> Result<()> {
        // Tunnel interface is removed by the time the security policy is reset, so there's no need
        // to revert the changes done.
        Ok(())
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
