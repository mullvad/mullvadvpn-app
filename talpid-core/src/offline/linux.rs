use crate::tunnel_state_machine::TunnelCommand;
use error_chain::ChainedError;
use futures::{future::Either, sync::mpsc::UnboundedSender, Future, Stream};
use iproute2::{Address, Connection, ConnectionHandle, Link, NetlinkIpError};
use log::{error, trace, warn};
use netlink_socket::{Protocol, SocketAddr, TokioSocket};
use rtnetlink::{
    AddressMessage, LinkLayerType, LinkMessage, NetlinkCodec, NetlinkFramed, NetlinkMessage,
    RtnlMessage,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    net::IpAddr,
    thread,
};

error_chain! {
    errors {
        GetLinksError {
            description("Failed to get list of IP links")
        }
        NetlinkConnectionError {
            description("Failed to connect to netlink socket")
        }
        NetlinkBindError {
            description("Failed to start listening on netlink socket")
        }
        NetlinkError {
            description("Error while communicating on the netlink socket")
        }
        NetlinkDisconnected {
            description("Netlink connection has unexpectedly disconnected")
        }
    }
}

const RTMGRP_NOTIFY: u32 = 1;
const RTMGRP_LINK: u32 = 2;
const RTMGRP_IPV4_IFADDR: u32 = 0x10;
const RTMGRP_IPV6_IFADDR: u32 = 0x100;

pub struct MonitorHandle;

pub fn spawn_monitor(sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    let mut socket =
        TokioSocket::new(Protocol::Route).chain_err(|| ErrorKind::NetlinkConnectionError)?;
    socket
        .bind(&SocketAddr::new(
            0,
            RTMGRP_NOTIFY | RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR,
        ))
        .chain_err(|| ErrorKind::NetlinkBindError)?;

    let channel = NetlinkFramed::new(socket, NetlinkCodec::<NetlinkMessage>::new());
    let link_monitor = LinkMonitor::new(sender)?;

    thread::spawn(|| {
        if let Err(error) = monitor_event_loop(channel, link_monitor) {
            let chained_error = error.chain_err(|| "Error running link monitor event loop");
            error!("{}", chained_error.display_chain());
        }
    });

    Ok(MonitorHandle)
}

pub fn is_offline() -> bool {
    check_if_offline().unwrap_or_else(|error| {
        let chained_error = error.chain_err(|| "Failed to check for internet connection");
        warn!("{}", chained_error.display_chain());
        false
    })
}

fn check_if_offline() -> Result<bool> {
    let mut connection = NetlinkConnection::new()?;
    let interfaces = connection.running_interfaces()?;

    if interfaces.is_empty() {
        Ok(true)
    } else {
        Ok(connection
            .addresses()?
            .into_iter()
            .all(|address| !interfaces.contains(&address.index())))
    }
}

pub struct NetlinkConnection {
    connection: Option<Connection>,
    connection_handle: ConnectionHandle,
}

impl NetlinkConnection {
    pub fn new() -> Result<Self> {
        let (connection, connection_handle) =
            iproute2::new_connection().chain_err(|| ErrorKind::NetlinkConnectionError)?;

        Ok(NetlinkConnection {
            connection: Some(connection),
            connection_handle,
        })
    }

    pub fn addresses(&mut self) -> Result<Vec<Address>> {
        self.execute_request(self.connection_handle.address().get().execute())
    }

    pub fn links(&mut self) -> Result<Vec<Link>> {
        self.execute_request(self.connection_handle.link().get().execute())
    }

    pub fn running_interfaces(&mut self) -> Result<BTreeSet<u32>> {
        let links = self.links()?;

        Ok(links
            .into_iter()
            .filter(link_provides_connectivity)
            .map(|link| link.index())
            .collect())
    }

    fn execute_request<R>(&mut self, request: R) -> Result<R::Item>
    where
        R: Future<Error = NetlinkIpError>,
    {
        let connection = self
            .connection
            .take()
            .ok_or(ErrorKind::NetlinkDisconnected)?;

        let (result, connection) = match connection.select2(request).wait() {
            Ok(Either::A(_)) => bail!(ErrorKind::NetlinkDisconnected),
            Err(Either::A((error, _))) => bail!(Error::with_chain(error, ErrorKind::NetlinkError)),
            Ok(Either::B((links, connection))) => (Ok(links), connection),
            Err(Either::B((error, connection))) => (
                Err(Error::with_chain(
                    failure::Fail::compat(error),
                    ErrorKind::GetLinksError,
                )),
                connection,
            ),
        };

        self.connection = Some(connection);
        result
    }
}

fn link_provides_connectivity(link: &Link) -> bool {
    // Some tunnels have the link layer type set to None
    link.link_layer_type() != LinkLayerType::Loopback
        && link.link_layer_type() != LinkLayerType::None
        && link.flags().is_running()
}

fn monitor_event_loop(
    channel: NetlinkFramed<NetlinkCodec<NetlinkMessage>>,
    mut link_monitor: LinkMonitor,
) -> Result<()> {
    channel
        .for_each(|(message, _address)| {
            let (_header, payload) = message.into_parts();

            match payload {
                RtnlMessage::NewLink(link_message) => link_monitor.new_link(link_message),
                RtnlMessage::DelLink(link_message) => link_monitor.del_link(link_message),
                RtnlMessage::NewAddress(address_message) => {
                    link_monitor.new_address(address_message)
                }
                RtnlMessage::DelAddress(address_message) => {
                    link_monitor.del_address(address_message)
                }
                _ => trace!("Ignoring unknown link message"),
            }

            Ok(())
        })
        .wait()
        .map_err(|error| {
            Error::with_chain(failure::Fail::compat(error), ErrorKind::NetlinkError)
        })?;

    Ok(())
}

struct LinkMonitor {
    is_offline: bool,
    running_interfaces: BTreeSet<u32>,
    interface_addresses: BTreeMap<u32, HashSet<(Option<IpAddr>, Option<IpAddr>)>>,
    sender: UnboundedSender<TunnelCommand>,
}

impl LinkMonitor {
    pub fn new(sender: UnboundedSender<TunnelCommand>) -> Result<Self> {
        let mut connection = NetlinkConnection::new()?;
        let running_interfaces = connection.running_interfaces()?;
        let addresses = connection.addresses()?;
        let mut interface_addresses = BTreeMap::new();

        for address in addresses {
            interface_addresses
                .entry(address.index())
                .or_insert_with(HashSet::new)
                .insert((address.ifa_address(), address.ifa_local()));
        }

        let mut monitor = LinkMonitor {
            is_offline: false,
            running_interfaces,
            interface_addresses,
            sender,
        };

        monitor.is_offline = monitor.check_if_offline();

        Ok(monitor)
    }

    pub fn new_link(&mut self, link_message: LinkMessage) {
        if let Ok(link) = Link::from_link_message(link_message) {
            let interface = link.index();

            if link_provides_connectivity(&link) {
                self.insert_interface(interface);
            } else {
                self.remove_interface(interface);
            }
        }
    }

    pub fn del_link(&mut self, link_message: LinkMessage) {
        if let Ok(link) = Link::from_link_message(link_message) {
            self.remove_interface(link.index());
        }
    }

    pub fn new_address(&mut self, address_message: AddressMessage) {
        if let Ok(address) = Address::from_address_message(address_message) {
            let interface = address.index();
            let address = (address.ifa_address(), address.ifa_local());

            self.interface_addresses
                .entry(interface)
                .or_insert_with(HashSet::new)
                .insert(address);

            if self.is_offline && self.running_interfaces.contains(&interface) {
                self.set_is_offline(false);
            }
        }
    }

    pub fn del_address(&mut self, address_message: AddressMessage) {
        if let Ok(address) = Address::from_address_message(address_message) {
            let interface = address.index();
            let address = (address.ifa_address(), address.ifa_local());

            if let Some(addresses) = self.interface_addresses.get_mut(&interface) {
                if !self.is_offline && addresses.is_empty() {
                    self.set_is_offline(self.check_if_offline());
                }
            }
        }
    }

    fn check_if_offline(&self) -> bool {
        self.running_interfaces.is_empty()
            || self
                .interface_addresses
                .iter()
                .filter(|(interface, _)| self.running_interfaces.contains(interface))
                .all(|(_, addresses)| addresses.is_empty())
    }

    fn set_is_offline(&mut self, is_offline: bool) {
        if self.is_offline != is_offline {
            self.is_offline = is_offline;
            let _ = self
                .sender
                .unbounded_send(TunnelCommand::IsOffline(is_offline));
        }
    }

    fn insert_interface(&mut self, interface_index: u32) {
        self.running_interfaces.insert(interface_index);

        if let Some(addresses) = self.interface_addresses.get(&interface_index) {
            if !addresses.is_empty() {
                self.set_is_offline(false);
            }
        }
    }

    fn remove_interface(&mut self, interface_index: u32) {
        self.running_interfaces.remove(&interface_index);

        if self.check_if_offline() {
            self.set_is_offline(true);
        }
    }
}
