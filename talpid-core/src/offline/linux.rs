use crate::tunnel_state_machine::TunnelCommand;
use error_chain::ChainedError;
use futures::{future::Either, sync::mpsc::UnboundedSender, Future, Stream};
use iproute2::Link;
use log::{error, trace, warn};
use netlink_socket::{Protocol, SocketAddr, TokioSocket};
use rtnetlink::{
    LinkFlags, LinkHeader, LinkLayerType, LinkMessage, NetlinkCodec, NetlinkFramed, NetlinkMessage,
    RtnlMessage,
};
use std::{collections::BTreeSet, thread};

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
    Ok(list_links_providing_connectivity()?.next().is_none())
}

fn list_links_providing_connectivity() -> Result<impl Iterator<Item = Link>> {
    Ok(list_links()?.into_iter().filter(link_provides_connectivity))
}

fn link_provides_connectivity(link: &impl BasicLinkInfo) -> bool {
    // Some tunnels have the link layer type set to None
    link.link_layer_type() != LinkLayerType::Loopback
        && link.link_layer_type() != LinkLayerType::None
        && link.flags().is_running()
}

fn list_links() -> Result<Vec<Link>> {
    let (connection, connection_handle) =
        iproute2::new_connection().chain_err(|| ErrorKind::NetlinkConnectionError)?;
    let links_request = connection_handle.link().get().execute();

    match connection.select2(links_request).wait() {
        Ok(Either::A(_)) => bail!(ErrorKind::NetlinkDisconnected),
        Ok(Either::B((links, _))) => Ok(links),
        Err(Either::A((error, _))) => Err(Error::with_chain(error, ErrorKind::NetlinkError)),
        Err(Either::B((error, _))) => Err(Error::with_chain(
            failure::Fail::compat(error),
            ErrorKind::GetLinksError,
        )),
    }
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
    running_links: BTreeSet<u32>,
    sender: UnboundedSender<TunnelCommand>,
}

impl LinkMonitor {
    pub fn new(sender: UnboundedSender<TunnelCommand>) -> Result<Self> {
        let links: Vec<Link> = list_links_providing_connectivity()?.collect();
        let is_offline = links.is_empty();
        let running_links = links.into_iter().map(|link| link.index()).collect();

        Ok(LinkMonitor {
            is_offline,
            running_links,
            sender,
        })
    }

    pub fn new_link(&mut self, link_message: LinkMessage) {
        if self.is_offline && link_provides_connectivity(link_message.header()) {
            self.set_is_offline(false);
        }

        if let Ok(link) = Link::from_link_message(link_message) {
            if link_provides_connectivity(&link) {
                self.insert_link(link.index());
            } else {
                self.remove_link(link.index());
            }
        }
    }

    pub fn del_link(&mut self, link_message: LinkMessage) {
        if let Ok(link) = Link::from_link_message(link_message) {
            self.remove_link(link.index());
        }
    }

    fn set_is_offline(&mut self, is_offline: bool) {
        if self.is_offline != is_offline {
            self.is_offline = is_offline;
            let _ = self
                .sender
                .unbounded_send(TunnelCommand::IsOffline(is_offline));
        }
    }

    fn insert_link(&mut self, link_index: u32) {
        self.running_links.insert(link_index);
        self.set_is_offline(false);
    }

    fn remove_link(&mut self, link_index: u32) {
        self.running_links.remove(&link_index);
        if self.running_links.is_empty() {
            self.set_is_offline(true);
        }
    }
}

trait BasicLinkInfo {
    fn flags(&self) -> LinkFlags;
    fn link_layer_type(&self) -> LinkLayerType;
}

impl BasicLinkInfo for Link {
    fn flags(&self) -> LinkFlags {
        self.flags()
    }

    fn link_layer_type(&self) -> LinkLayerType {
        self.link_layer_type()
    }
}

impl BasicLinkInfo for LinkHeader {
    fn flags(&self) -> LinkFlags {
        self.flags()
    }

    fn link_layer_type(&self) -> LinkLayerType {
        self.link_layer_type()
    }
}
