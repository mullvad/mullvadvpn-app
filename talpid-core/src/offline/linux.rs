use crate::tunnel_state_machine::TunnelCommand;
use error_chain::ChainedError;
use futures::{future::Either, sync::mpsc::UnboundedSender, Future, Stream};
use iproute2::{Connection, ConnectionHandle, Link, NetlinkIpError};
use log::{error, warn};
use netlink_socket::{Protocol, SocketAddr, TokioSocket};
use rtnetlink::{LinkLayerType, NetlinkCodec, NetlinkFramed, NetlinkMessage};
use std::thread;

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

pub struct MonitorHandle;

pub fn spawn_monitor(sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    let mut socket =
        TokioSocket::new(Protocol::Route).chain_err(|| ErrorKind::NetlinkConnectionError)?;
    socket
        .bind(&SocketAddr::new(0, RTMGRP_NOTIFY | RTMGRP_LINK))
        .chain_err(|| ErrorKind::NetlinkBindError)?;

    let channel = NetlinkFramed::new(socket, NetlinkCodec::<NetlinkMessage>::new());
    let link_monitor = LinkMonitor::new(sender);

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
    Ok(list_links_providing_connectivity()?.next().is_none())
}

fn list_links_providing_connectivity() -> Result<impl Iterator<Item = Link>> {
    let mut connection = NetlinkConnection::new()?;

    Ok(connection
        .links()?
        .into_iter()
        .filter(link_provides_connectivity))
}

fn link_provides_connectivity(link: &Link) -> bool {
    // Some tunnels have the link layer type set to None
    link.link_layer_type() != LinkLayerType::Loopback
        && link.link_layer_type() != LinkLayerType::None
        && link.flags().is_running()
}

struct NetlinkConnection {
    connection: Option<Connection>,
    connection_handle: ConnectionHandle,
}

impl NetlinkConnection {
    /// Open a connection on the netlink socket.
    pub fn new() -> Result<Self> {
        let (connection, connection_handle) =
            iproute2::new_connection().chain_err(|| ErrorKind::NetlinkConnectionError)?;

        Ok(NetlinkConnection {
            connection: Some(connection),
            connection_handle,
        })
    }

    /// List all links registered on the system.
    pub fn links(&mut self) -> Result<Vec<Link>> {
        self.execute_request(self.connection_handle.link().get().execute())
    }

    /// Helper function to execute an asynchronous request synchronously.
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

fn monitor_event_loop(
    channel: NetlinkFramed<NetlinkCodec<NetlinkMessage>>,
    mut link_monitor: LinkMonitor,
) -> Result<()> {
    channel
        .for_each(|(_message, _address)| {
            link_monitor.update();
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
