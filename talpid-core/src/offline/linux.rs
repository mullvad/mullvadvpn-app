extern crate iproute2;
extern crate rtnetlink;

use error_chain::ChainedError;
use futures::{future::Either, sync::mpsc::UnboundedSender, Future};
use log::warn;

use self::iproute2::Link;
use self::rtnetlink::LinkLayerType;

use tunnel_state_machine::TunnelCommand;

error_chain! {
    errors {
        GetLinksError {
            description("Failed to get list of IP links")
        }
        NetlinkConnectionError {
            description("Failed to connect to netlink socket")
        }
        NetlinkError {
            description("Error while communicating on the netlink socket")
        }
        NetlinkDisconnected {
            description("Netlink connection has unexpectedly disconnected")
        }
    }
}

pub struct MonitorHandle;

pub fn spawn_monitor(_sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
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

fn link_provides_connectivity(link: &Link) -> bool {
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
