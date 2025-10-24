use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
    sync::{Arc, Mutex},
};

#[cfg(windows)]
#[path = "windows.rs"]
pub mod network_interface;

pub mod tun_provider;
use futures::{
    SinkExt,
    channel::{
        mpsc::UnboundedSender,
        oneshot::{self, Sender},
    },
};
use talpid_routing::RouteManagerHandle;
use talpid_types::net::AllowedTunnelTraffic;
use tun_provider::TunProvider;

/// Size of IPv4 header in bytes
pub const IPV4_HEADER_SIZE: u16 = 20;
/// Size of IPv6 header in bytes
pub const IPV6_HEADER_SIZE: u16 = 40;
/// WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
pub const WIREGUARD_HEADER_SIZE: u16 = 8 + 32;
/// Size of ICMP header in bytes
pub const ICMP_HEADER_SIZE: u16 = 8;
/// Smallest allowed MTU for IPv4 in bytes
pub const MIN_IPV4_MTU: u16 = 576;
/// Smallest allowed MTU for IPv6 in bytes
pub const MIN_IPV6_MTU: u16 = 1280;

/// Arguments for creating a tunnel.
pub struct TunnelArgs<'a> {
    /// Tokio runtime handle.
    pub runtime: tokio::runtime::Handle,
    /// Resource directory path.
    pub resource_dir: &'a Path,
    /// Callback function called when an event happens.
    pub event_hook: EventHook,
    /// Receiver oneshot channel for closing the tunnel.
    pub tunnel_close_rx: oneshot::Receiver<()>,
    /// Mutex to tunnel provider.
    pub tun_provider: Arc<Mutex<TunProvider>>,
    /// Connection retry attempts.
    pub retry_attempt: u32,
    /// Route manager handle.
    pub route_manager: RouteManagerHandle,
}

#[derive(Clone)]
pub struct EventHook {
    event_tx: UnboundedSender<(TunnelEvent, Sender<()>)>,
}

impl EventHook {
    pub fn new(event_tx: UnboundedSender<(TunnelEvent, Sender<()>)>) -> Self {
        Self { event_tx }
    }

    pub async fn on_event(&mut self, event: TunnelEvent) {
        let (tx, rx) = oneshot::channel::<()>();
        if let Ok(()) = self.event_tx.send((event, tx)).await {
            let _ = rx.await;
        }
    }
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

impl TunnelMetadata {
    /// Return a copy of all gateway addresses
    pub fn gateways(&self) -> Vec<IpAddr> {
        let mut addrs = vec![self.ipv4_gateway.into()];
        if let Some(gateway) = self.ipv6_gateway {
            addrs.push(gateway.into());
        }
        addrs
    }
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
