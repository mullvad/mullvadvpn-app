use crate::tunnel_state_machine::TunnelCommand;
use futures::{
    channel::{mpsc::UnboundedSender, oneshot},
    FutureExt, StreamExt, TryStreamExt,
};
use netlink_packet_route::{
    constants::{ARPHRD_LOOPBACK, ARPHRD_NONE, IFF_LOWER_UP, IFF_UP},
    rtnl::link::nlas::{Info as LinkInfo, InfoKind, Nla as LinkNla},
    LinkMessage,
};
use rtnetlink::{
    constants::{RTMGRP_IPV4_IFADDR, RTMGRP_IPV6_IFADDR, RTMGRP_LINK, RTMGRP_NOTIFY},
    sys::SocketAddr,
    Handle,
};
use std::{collections::BTreeSet, io, sync::Weak};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to get list of IP links")]
    GetLinksError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to get list of IP addresses")]
    GetAddressesError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to connect to netlink socket")]
    NetlinkConnectionError(#[error(source)] io::Error),

    #[error(display = "Failed to connect to bind to netlink socket")]
    BindError(#[error(source)] io::Error),

    #[error(display = "Failed to start listening on netlink socket")]
    NetlinkBindError(#[error(source)] io::Error),

    #[error(display = "Error while processing netlink messages")]
    MonitorNetlinkError,

    #[error(display = "Netlink connection has unexpectedly disconnected")]
    NetlinkDisconnected,

    #[error(display = "Failed to initialize event loop")]
    EventLoopError(#[error(source)] io::Error),
}

pub struct MonitorHandle {
    handle: rtnetlink::Handle,
    _stop_connection_tx: oneshot::Sender<()>,
}

impl MonitorHandle {
    pub async fn is_offline(&mut self) -> bool {
        match check_offline_state(&self.handle).await {
            Ok(is_offline) => is_offline,
            Err(err) => {
                log::error!(
                    "Failed to verify offline state: {}. Presuming connectivity",
                    err
                );
                false
            }
        }
    }
}

pub async fn spawn_monitor(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<MonitorHandle> {
    let (mut connection, handle, mut messages) =
        rtnetlink::new_connection().map_err(Error::NetlinkConnectionError)?;

    let mgroup_flags = RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR | RTMGRP_LINK | RTMGRP_NOTIFY;
    let addr = SocketAddr::new(0, mgroup_flags);

    connection
        .socket_mut()
        .bind(&addr)
        .map_err(Error::BindError)?;

    let (stop_connection_tx, stop_rx) = oneshot::channel();

    // Connection will be closed once the channel is dropped
    tokio::spawn(async {
        futures::select! {
            _ = connection.fuse() => (),
            _ = stop_rx.fuse() => (),
        }
    });
    let mut is_offline = check_offline_state(&handle).await?;

    let monitor_handle = MonitorHandle {
        handle: handle.clone(),
        _stop_connection_tx: stop_connection_tx,
    };


    tokio::spawn(async move {
        while let Some(_new_message) = messages.next().await {
            match sender.upgrade() {
                Some(sender) => {
                    let new_offline_state = check_offline_state(&handle).await.unwrap_or(false);
                    if new_offline_state != is_offline {
                        is_offline = new_offline_state;
                        let _ = sender.unbounded_send(TunnelCommand::IsOffline(is_offline));
                    }
                }
                None => return,
            }
        }
    });


    Ok(monitor_handle)
}

async fn check_offline_state(handle: &Handle) -> Result<bool> {
    let mut link_request = handle.link().get().execute();
    let mut links = BTreeSet::new();
    while let Some(link) = link_request
        .try_next()
        .await
        .map_err(failure::Fail::compat)
        .map_err(Error::GetLinksError)?
    {
        if link_provides_connectivity(&link) {
            links.insert(link.header.index);
        }
    }

    if links.is_empty() {
        return Ok(true);
    }

    let mut address_request = handle.address().get().execute();

    while let Some(address) = address_request
        .try_next()
        .await
        .map_err(failure::Fail::compat)
        .map_err(Error::GetAddressesError)?
    {
        if links.contains(&address.header.index) {
            return Ok(false);
        }
    }
    Ok(true)
}


// TODO: Improve by allowing bridge links to provide connectivity, will require route checking.
fn link_provides_connectivity(link: &LinkMessage) -> bool {
    // Some tunnels have the link layer type set to None
    link.header.link_layer_type != ARPHRD_NONE
        && link.header.link_layer_type != ARPHRD_LOOPBACK
        && (link.header.flags & IFF_UP > 0 || link.header.flags & IFF_LOWER_UP > 0)
        && !is_virtual_interface(link)
}

fn is_virtual_interface(link: &LinkMessage) -> bool {
    for nla in link.nlas.iter() {
        if let LinkNla::Info(info_nlas) = nla {
            for info in info_nlas.iter() {
                // LinkInfo::Kind seems to only be set when the link is actually virtual
                if let LinkInfo::Kind(ref kind) = info {
                    use InfoKind::*;
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
