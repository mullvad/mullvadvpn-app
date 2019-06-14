use crate::tunnel_state_machine::TunnelCommand;
use futures::{future::Either, sync::mpsc::UnboundedSender, Future, Stream};
use log::{error, warn};
use netlink_packet::{
    AddressMessage, LinkInfo, LinkInfoKind, LinkLayerType, LinkMessage, LinkNla, NetlinkMessage,
};
use netlink_sys::SocketAddr;
use rtnetlink::{
    constants::{RTMGRP_IPV4_IFADDR, RTMGRP_IPV6_IFADDR, RTMGRP_LINK, RTMGRP_NOTIFY},
    Connection, Handle,
};
use std::{collections::BTreeSet, io, thread};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to get list of IP links")]
    GetLinksError(#[error(cause)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to connect to netlink socket")]
    NetlinkConnectionError(#[error(cause)] io::Error),

    #[error(display = "Failed to start listening on netlink socket")]
    NetlinkBindError(#[error(cause)] io::Error),

    #[error(display = "Error while communicating on the netlink socket")]
    NetlinkError(#[error(cause)] netlink_proto::Error),

    #[error(display = "Error while processing netlink messages")]
    MonitorNetlinkError,

    #[error(display = "Netlink connection has unexpectedly disconnected")]
    NetlinkDisconnected,
}

pub struct MonitorHandle;

pub fn spawn_monitor(sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    let socket = SocketAddr::new(
        0,
        RTMGRP_NOTIFY | RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR,
    );

    let (mut connection, _, messages) = rtnetlink::new_connection_with_messages().unwrap();
    connection
        .socket_mut()
        .bind(&socket)
        .map_err(Error::NetlinkBindError)?;

    let link_monitor = LinkMonitor::new(sender);

    thread::spawn(|| {
        if let Err(error) = monitor_event_loop(connection, messages, link_monitor) {
            error!(
                "{}",
                error.display_chain_with_msg("Error running link monitor event loop")
            );
        }
    });

    Ok(MonitorHandle)
}

pub fn is_offline() -> bool {
    check_if_offline().unwrap_or_else(|error| {
        warn!(
            "{}",
            error.display_chain_with_msg("Failed to check for internet connection")
        );
        false
    })
}

/// Checks if there are no running links or that none of the running links have IP addresses
/// assigned to them.
fn check_if_offline() -> Result<bool> {
    let mut connection = NetlinkConnection::new()?;
    let interfaces = connection.running_interfaces()?;

    if interfaces.is_empty() {
        Ok(true)
    } else {
        // Check if the current IP addresses are not assigned to any one of the running interfaces
        Ok(connection
            .addresses()?
            .into_iter()
            .all(|address| !interfaces.contains(&address.header.index)))
    }
}

struct NetlinkConnection {
    connection: Option<Connection>,
    handle: Handle,
}

impl NetlinkConnection {
    /// Open a connection on the netlink socket.
    pub fn new() -> Result<Self> {
        let (connection, handle) =
            rtnetlink::new_connection().map_err(Error::NetlinkConnectionError)?;

        Ok(NetlinkConnection {
            connection: Some(connection),
            handle,
        })
    }

    /// List all IP addresses assigned to all interfaces.
    pub fn addresses(&mut self) -> Result<Vec<AddressMessage>> {
        self.execute_request(self.handle.address().get().execute().collect())
    }

    /// List all links registered on the system.
    fn links(&mut self) -> Result<Vec<LinkMessage>> {
        self.execute_request(self.handle.link().get().execute().collect())
    }

    /// List all unique interface indices that have a running link.
    pub fn running_interfaces(&mut self) -> Result<BTreeSet<u32>> {
        let links = self.links()?;

        Ok(links
            .into_iter()
            .filter(link_provides_connectivity)
            .map(|link| link.header.index)
            .collect())
    }

    /// Helper function to execute an asynchronous request synchronously.
    fn execute_request<R>(&mut self, request: R) -> Result<R::Item>
    where
        R: Future<Error = rtnetlink::Error>,
    {
        let connection = self.connection.take().ok_or(Error::NetlinkDisconnected)?;

        let (result, connection) = match connection.select2(request).wait() {
            Ok(Either::A(_)) => return Err(Error::NetlinkDisconnected),
            Err(Either::A((error, _))) => return Err(Error::NetlinkError(error)),
            Ok(Either::B((links, connection))) => (Ok(links), connection),
            Err(Either::B((error, connection))) => (
                Err(Error::GetLinksError(failure::Fail::compat(error))),
                connection,
            ),
        };

        self.connection = Some(connection);
        result
    }
}

fn link_provides_connectivity(link: &LinkMessage) -> bool {
    // Some tunnels have the link layer type set to None
    link.header.link_layer_type != LinkLayerType::Loopback
        && link.header.link_layer_type != LinkLayerType::None
        && link.header.link_layer_type != LinkLayerType::Irda
        && link.header.flags.is_running()
        && !is_virtual_interface(link)
}

fn is_virtual_interface(link: &LinkMessage) -> bool {
    for nla in link.nlas.iter() {
        if let LinkNla::LinkInfo(link_info) = nla {
            for info in link_info.iter() {
                // LinkInfo::Kind seems to only be set when the link is actually virtual
                if let LinkInfo::Kind(ref kind) = info {
                    use LinkInfoKind::*;
                    return match kind {
                        Dummy | Bridge | Tun | Nlmon | IpTun => true,
                        _ => false,
                    };
                }
            }
        }
    }
    false
}

fn monitor_event_loop(
    connection: Connection,
    channel: impl Stream<Item = NetlinkMessage, Error = ()>,
    mut link_monitor: LinkMonitor,
) -> Result<()> {
    let monitor = channel
        .for_each(|_message| {
            link_monitor.update();
            Ok(())
        })
        .map_err(|_| Error::MonitorNetlinkError);

    connection
        .map_err(Error::NetlinkError)
        .join(monitor)
        .wait()
        .map(|_| ())
}

struct LinkMonitor {
    is_offline: bool,
    sender: UnboundedSender<TunnelCommand>,
}

impl LinkMonitor {
    pub fn new(sender: UnboundedSender<TunnelCommand>) -> Self {
        let is_offline = is_offline();

        LinkMonitor { is_offline, sender }
    }

    pub fn update(&mut self) {
        self.set_is_offline(is_offline());
    }

    fn set_is_offline(&mut self, is_offline: bool) {
        if self.is_offline != is_offline {
            self.is_offline = is_offline;
            let _ = self
                .sender
                .unbounded_send(TunnelCommand::IsOffline(is_offline));
        }
    }
}
