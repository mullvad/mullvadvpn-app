use crate::tunnel_state_machine::TunnelCommand;
use futures::{channel::mpsc::UnboundedSender, StreamExt, TryStreamExt};
use netlink_packet_route::{
    constants::{ARPHRD_LOOPBACK, ARPHRD_NONE, IFF_LOWER_UP, IFF_UP},
    rtnl::link::nlas::{Info as LinkInfo, InfoKind, Nla as LinkNla},
    LinkMessage,
};
use netlink_sys::SocketAddr;
use rtnetlink::{
    constants::{RTMGRP_IPV4_IFADDR, RTMGRP_IPV6_IFADDR, RTMGRP_LINK, RTMGRP_NOTIFY},
    Handle,
};
use std::{collections::BTreeSet, io, sync::Weak};

pub type Result<T> = std::result::Result<T, Error>;

const EVENT_LOOP_THREAD_NAME: &str = "mullvad-offline-detection-event-loop";

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
    runtime: tokio02::runtime::Runtime,
}

impl MonitorHandle {
    pub fn is_offline(&mut self) -> bool {
        match self.runtime.block_on(check_offline_state(&self.handle)) {
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

pub fn spawn_monitor(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<MonitorHandle> {
    let mut runtime = tokio02::runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(1)
        .enable_all()
        .thread_name(EVENT_LOOP_THREAD_NAME)
        .build()
        .map_err(Error::EventLoopError)?;

    let (connection, handle, mut messages) = runtime.block_on(async move {
        let (mut connection, handle, messages) =
            rtnetlink::new_connection().map_err(Error::NetlinkConnectionError)?;

        let mgroup_flags = RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);

        connection
            .socket_mut()
            .bind(&addr)
            .map_err(Error::BindError)?;

        Ok((connection, handle, messages))
    })?;

    // Connection will be closed once the runtime is dropped
    let _ = runtime.spawn(connection);
    let mut is_offline = runtime.block_on(check_offline_state(&handle))?;

    let monitor_handle = MonitorHandle {
        handle: handle.clone(),
        runtime,
    };


    let _ = monitor_handle.runtime.spawn(async move {
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
