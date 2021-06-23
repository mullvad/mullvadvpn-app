use crate::{
    linux::{iface_index, IfaceIndexLookupError},
    routing::{self, RouteManagerHandle},
};
use futures::{
    channel::mpsc::UnboundedSender,
    stream::{abortable, AbortHandle},
    StreamExt,
};
use rtnetlink::IpVersion;
use std::{
    collections::BTreeMap,
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

const PUBLIC_INTERNET_ADDRESS_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6));
const PUBLIC_INTERNET_ADDRESS_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "The route manager returned an error")]
    RouteManagerError(#[error(source)] routing::Error),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceNameError(#[error(source)] IfaceIndexLookupError),
}

pub struct DnsRouteMonitor {
    abort_handle: AbortHandle,
}

impl Drop for DnsRouteMonitor {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DnsConfig {
    pub interface: u32,
    pub resolvers: Vec<IpAddr>,
}

impl fmt::Display for DnsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "interface index {}, resolvers:", self.interface)?;
        for server in &self.resolvers {
            write!(f, " {}", server)?;
        }
        Ok(())
    }
}

pub async fn spawn_monitor(
    route_manager: RouteManagerHandle,
    destinations: Vec<IpAddr>,
    update_tx: UnboundedSender<BTreeMap<u32, DnsConfig>>,
) -> Result<(DnsRouteMonitor, BTreeMap<u32, DnsConfig>)> {
    let listener = route_manager
        .change_listener()
        .await
        .map_err(Error::RouteManagerError)?;
    let (mut listener, abort_handle) = abortable(listener);

    let monitor = DnsRouteMonitor { abort_handle };

    let mut last_config = setup_configurations(&route_manager, &destinations).await?;
    let initial_config = last_config.clone();

    tokio::spawn(async move {
        while let Some(_event) = listener.next().await {
            match setup_configurations(&route_manager, &destinations).await {
                Ok(new_config) => {
                    if last_config != new_config {
                        last_config = new_config.clone();
                        if update_tx.unbounded_send(new_config).is_err() {
                            log::trace!("Stopping DNS monitor: channel is closed");
                            break;
                        }
                    }
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to determine new DNS interface settings"
                        )
                    );
                }
            }
        }
    });

    Ok((monitor, initial_config))
}

async fn setup_configurations(
    handle: &RouteManagerHandle,
    destinations: &[IpAddr],
) -> Result<BTreeMap<u32, DnsConfig>> {
    let mut interface_to_destinations = BTreeMap::<u32, DnsConfig>::new();
    for destination in destinations {
        let interface = if destination.is_loopback() {
            get_default_route_interface(handle, get_ip_version(destination), true).await?
        } else {
            if crate::firewall::is_local_address(destination) {
                get_destination_interface(handle, *destination, true).await?
            } else {
                get_default_route_interface(handle, get_ip_version(destination), false).await?
            }
        };
        match interface {
            Some(iface) => {
                if let Some(config) = interface_to_destinations.get_mut(&iface) {
                    config.resolvers.push(*destination);
                } else {
                    interface_to_destinations.insert(
                        iface,
                        DnsConfig {
                            interface: iface,
                            resolvers: vec![*destination],
                        },
                    );
                }
            }
            None => {
                log::trace!(
                    "Ignoring DNS server that did not match to any interface: {}",
                    destination
                );
            }
        }
    }

    Ok(interface_to_destinations)
}

async fn get_default_route_interface(
    handle: &RouteManagerHandle,
    ip_version: IpVersion,
    set_mark: bool,
) -> Result<Option<u32>> {
    match ip_version {
        IpVersion::V4 => {
            get_destination_interface(handle, PUBLIC_INTERNET_ADDRESS_V4, set_mark).await
        }
        IpVersion::V6 => {
            get_destination_interface(handle, PUBLIC_INTERNET_ADDRESS_V6, set_mark).await
        }
    }
}

async fn get_destination_interface(
    handle: &RouteManagerHandle,
    destination: IpAddr,
    set_mark: bool,
) -> Result<Option<u32>> {
    let route = handle
        .get_destination_route(destination, set_mark)
        .await
        .map_err(Error::RouteManagerError)?;
    route
        .map(|route| route.get_node().get_device().map(iface_index))
        .flatten()
        .transpose()
        .map_err(Error::InterfaceNameError)
}

fn get_ip_version(addr: &IpAddr) -> IpVersion {
    if addr.is_ipv4() {
        IpVersion::V4
    } else {
        IpVersion::V6
    }
}
