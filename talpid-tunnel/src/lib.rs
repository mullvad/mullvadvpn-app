use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
    sync::{Arc, Mutex},
};

#[cfg(windows)]
#[path = "windows.rs"]
pub mod network_interface;

pub mod tun_provider;
use futures::{channel::oneshot, future::BoxFuture};
use talpid_routing::RouteManagerHandle;
use talpid_types::net::AllowedTunnelTraffic;
use tun_provider::TunProvider;

/// Arguments for creating a tunnel.
pub struct TunnelArgs<'a, L>
where
    L: (Fn(TunnelEvent) -> BoxFuture<'static, ()>) + Send + Clone + Sync + 'static,
{
    /// Toktio runtime handle.
    pub runtime: tokio::runtime::Handle,
    /// Resource directory path.
    pub resource_dir: &'a Path,
    /// Callback function called when an event happens.
    pub on_event: L,
    /// Receiver oneshot channel for closing the tunnel.
    pub tunnel_close_rx: oneshot::Receiver<()>,
    /// Mutex to tunnel provider.
    pub tun_provider: Arc<Mutex<TunProvider>>,
    /// Connection retry attempts.
    pub retry_attempt: u32,
    /// Route manager handle.
    pub route_manager: RouteManagerHandle,
}

/// Information about a VPN tunnel.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TunnelMetadata {
    /// The name of the device which the tunnel is running on.
    pub interface: String,
    /// The local IPs on the tunnel interface.
    pub ips: Vec<IpAddr>,
    /// The IP to the default gateway on the tunnel interface.
    pub ipv4_gateway: Ipv4Addr,
    /// The IP to the IPv6 default gateway on the tunnel interface.
    pub ipv6_gateway: Option<Ipv6Addr>,
}

/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TunnelEvent {
    /// Sent when the tunnel fails to connect due to an authentication error.
    AuthFailed(Option<String>),
    /// Sent when the tunnel interface has been created, before routes are set up.
    InterfaceUp(TunnelMetadata, AllowedTunnelTraffic),
    /// Sent when the tunnel comes up and is ready for traffic.
    Up(TunnelMetadata),
    /// Sent when the tunnel goes down, but before destroying the tunnel device.
    Down,
}
