use self::config::Config;
#[cfg(not(windows))]
use super::tun_provider;
use super::{tun_provider::TunProvider, TunnelEvent, TunnelMetadata};
use crate::routing::{self, RequiredRoute};
use futures::future::abortable;
#[cfg(target_os = "linux")]
use lazy_static::lazy_static;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(windows)]
use std::io;
use std::{
    collections::HashSet,
    net::SocketAddr,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};
use talpid_types::{net::TransportProtocol, ErrorExt};
use udp_over_tcp::{TcpOptions, Udp2Tcp};

/// WireGuard config data-types
pub mod config;
mod connectivity_check;
mod logging;
mod stats;
mod wireguard_go;
#[cfg(target_os = "linux")]
pub(crate) mod wireguard_kernel;

use self::wireguard_go::WgGoTunnel;

type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Wireguard tunnel monitor.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to set up routing.
    #[error(display = "Failed to setup routing")]
    SetupRoutingError(#[error(source)] crate::routing::Error),

    /// Tunnel timed out
    #[error(display = "Tunnel timed out")]
    TimeoutError,

    /// An interaction with a tunnel failed
    #[error(display = "Tunnel failed")]
    TunnelError(#[error(source)] TunnelError),

    /// Failed to set up Udp2Tcp
    #[error(display = "Failed to start UDP-over-TCP proxy")]
    Udp2TcpError(#[error(source)] udp_over_tcp::udp2tcp::ConnectError),

    /// Failed to obtain the local UDP socket address
    #[error(display = "Failed obtain local address for the UDP socket in Udp2Tcp")]
    GetLocalUdpAddress(#[error(source)] std::io::Error),

    /// Failed to set up connectivity monitor
    #[error(display = "Connectivity monitor failed")]
    ConnectivityMonitorError(#[error(source)] connectivity_check::Error),

    /// Failed to set up IP interfaces.
    #[cfg(windows)]
    #[error(display = "Failed while waiting on IP interfaces")]
    IpInterfacesError(#[error(source)] io::Error),

    /// Failed to set IP addresses on WireGuard interface
    #[cfg(target_os = "windows")]
    #[error(display = "Failed to set IP addresses on WireGuard interface")]
    SetIpAddressesError,
}


/// Spawns and monitors a wireguard tunnel
pub struct WireguardMonitor {
    /// Tunnel implementation
    tunnel: Arc<Mutex<Option<Box<dyn Tunnel>>>>,
    /// Callback to signal tunnel events
    event_callback: Box<dyn Fn(TunnelEvent) + Send + Sync + 'static>,
    close_msg_sender: mpsc::Sender<CloseMsg>,
    close_msg_receiver: mpsc::Receiver<CloseMsg>,
    #[cfg(target_os = "windows")]
    stop_setup_tx: Option<futures::channel::oneshot::Sender<()>>,
    pinger_stop_sender: mpsc::Sender<()>,
    _tcp_proxies: Vec<TcpProxy>,
}

#[cfg(target_os = "linux")]
lazy_static! {
    /// Overrides the preference for the kernel module for WireGuard.
    static ref FORCE_USERSPACE_WIREGUARD: bool = env::var("TALPID_FORCE_USERSPACE_WIREGUARD")
        .map(|v| v != "0")
        .unwrap_or(false);

    static ref FORCE_NM_WIREGUARD: bool = env::var("TALPID_FORCE_NM_WIREGUARD")
        .map(|v| v != "0")
        .unwrap_or(false);
}

struct TcpProxy {
    local_addr: SocketAddr,
    abort_handle: futures::future::AbortHandle,
}

impl TcpProxy {
    pub fn new(runtime: &tokio::runtime::Handle, endpoint: SocketAddr) -> Result<Self> {
        let listen_addr = if endpoint.is_ipv4() {
            SocketAddr::new("127.0.0.1".parse().unwrap(), 0)
        } else {
            SocketAddr::new("::1".parse().unwrap(), 0)
        };

        let udp2tcp = runtime
            .block_on(Udp2Tcp::new(
                listen_addr,
                endpoint,
                Some(&TcpOptions {
                    #[cfg(target_os = "linux")]
                    fwmark: Some(crate::linux::TUNNEL_FW_MARK),
                    ..TcpOptions::default()
                }),
            ))
            .map_err(Error::Udp2TcpError)?;

        let local_addr = udp2tcp
            .local_udp_addr()
            .map_err(Error::GetLocalUdpAddress)?;

        let (udp2tcp_future, abort_handle) = abortable(udp2tcp.run());
        runtime.spawn(udp2tcp_future);

        Ok(Self {
            local_addr,
            abort_handle,
        })
    }

    pub fn local_udp_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for TcpProxy {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

impl WireguardMonitor {
    /// Starts a WireGuard tunnel with the given config
    pub fn start<F: Fn(TunnelEvent) + Send + Sync + Clone + 'static>(
        runtime: tokio::runtime::Handle,
        mut config: Config,
        log_path: Option<&Path>,
        on_event: F,
        tun_provider: &mut TunProvider,
        route_manager: &mut routing::RouteManager,
    ) -> Result<WireguardMonitor> {
        let mut tcp_proxies = vec![];

        for peer in &mut config.peers {
            if peer.protocol == TransportProtocol::Tcp {
                let udp2tcp = TcpProxy::new(&runtime, peer.endpoint.clone())?;

                // Replace remote peer with proxy
                peer.endpoint = udp2tcp.local_udp_addr();

                tcp_proxies.push(udp2tcp);
            }
        }

        let tunnel = Self::open_tunnel(&config, log_path, tun_provider, route_manager)?;
        let iface_name = tunnel.get_interface_name().to_string();
        #[cfg(windows)]
        let iface_luid = tunnel.get_interface_luid();

        let metadata = Self::tunnel_metadata(&iface_name, &config);
        (on_event)(TunnelEvent::InterfaceUp(metadata.clone()));

        #[cfg(target_os = "windows")]
        route_manager
            .add_default_route_callback(Some(WgGoTunnel::default_route_changed_callback), ());

        let event_callback = Box::new(on_event.clone());
        let (close_msg_sender, close_msg_receiver) = mpsc::channel();
        let (pinger_tx, pinger_rx) = mpsc::channel();
        #[cfg(target_os = "windows")]
        let (stop_setup_tx, stop_setup_rx) = futures::channel::oneshot::channel();
        let monitor = WireguardMonitor {
            tunnel: Arc::new(Mutex::new(Some(tunnel))),
            event_callback,
            close_msg_sender,
            close_msg_receiver,
            #[cfg(target_os = "windows")]
            stop_setup_tx: Some(stop_setup_tx),
            pinger_stop_sender: pinger_tx,
            _tcp_proxies: tcp_proxies,
        };

        let gateway = config.ipv4_gateway;
        let close_sender = monitor.close_msg_sender.clone();
        let mut connectivity_monitor = connectivity_check::ConnectivityMonitor::new(
            gateway,
            iface_name.clone(),
            Arc::downgrade(&monitor.tunnel),
            pinger_rx,
        )
        .map_err(Error::ConnectivityMonitorError)?;

        let route_handle = route_manager.handle().map_err(Error::SetupRoutingError)?;
        #[cfg(windows)]
        let runtime = route_manager.runtime_handle();

        std::thread::spawn(move || {
            #[cfg(windows)]
            {
                let iface_close_sender = close_sender.clone();
                let enable_ipv6 = config.ipv6_gateway.is_some();

                let result = runtime.block_on(async move {
                    use futures::future::FutureExt;
                    use winapi::shared::ifdef::NET_LUID;
                    let luid = NET_LUID { Value: iface_luid };
                    let setup_future = super::windows::wait_for_interfaces(luid, true, enable_ipv6);

                    futures::select! {
                        result = setup_future.fuse() => {
                            result.map_err(|error|
                                iface_close_sender.send(CloseMsg::SetupError(
                                    Error::IpInterfacesError(error)
                                ))
                                .unwrap_or(())
                            )
                        }
                        _ = stop_setup_rx.fuse() => Err(()),
                    }
                });

                if result.is_err() {
                    return;
                }
            }

            let setup_iface_routes = move || -> Result<()> {
                #[cfg(target_os = "windows")]
                if !crate::winnet::add_device_ip_addresses(&iface_name, &config.tunnel.addresses) {
                    return Err(Error::SetIpAddressesError);
                }

                #[cfg(target_os = "linux")]
                route_handle
                    .create_routing_rules(config.enable_ipv6)
                    .map_err(Error::SetupRoutingError)?;

                route_handle
                    .add_routes(Self::get_routes(&iface_name, &config))
                    .map_err(Error::SetupRoutingError)
            };

            if let Err(error) = setup_iface_routes() {
                let _ = close_sender.send(CloseMsg::SetupError(error));
                return;
            }

            match connectivity_monitor.establish_connectivity() {
                Ok(true) => {
                    (on_event)(TunnelEvent::Up(metadata));

                    if let Err(error) = connectivity_monitor.run() {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Connectivity monitor failed")
                        );
                    }
                }
                Ok(false) => log::warn!("Timeout while checking tunnel connection"),
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to check tunnel connection")
                    );
                }
            }

            let _ = close_sender.send(CloseMsg::PingErr);
        });

        Ok(monitor)
    }

    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))]
    fn open_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: &mut TunProvider,
        route_manager: &mut routing::RouteManager,
    ) -> Result<Box<dyn Tunnel>> {
        #[cfg(target_os = "linux")]
        if !*FORCE_USERSPACE_WIREGUARD {
            if crate::dns::will_use_nm() {
                match wireguard_kernel::NetworkManagerTunnel::new(config) {
                    Ok(tunnel) => {
                        log::debug!("Using NetworkManager to use kernel WireGuard implementation");
                        return Ok(Box::new(tunnel));
                    }
                    Err(err) => {
                        log::error!(
                            "{}",
                            err.display_chain_with_msg(
                                "Failed to initialize WireGuard tunnel via NetworkManager"
                            )
                        );
                    }
                };
            } else {
                match wireguard_kernel::NetlinkTunnel::new(route_manager.runtime_handle(), config) {
                    Ok(tunnel) => {
                        log::debug!("Using kernel WireGuard implementation");
                        return Ok(Box::new(tunnel));
                    }
                    Err(error) => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg(
                                "Failed to setup kernel WireGuard device, falling back to the userspace implementation"
                            )
                        );
                    }
                };
            }
        }

        #[cfg(target_os = "linux")]
        log::debug!("Using userspace WireGuard implementation");
        Ok(Box::new(
            WgGoTunnel::start_tunnel(
                &config,
                log_path,
                tun_provider,
                Self::get_tunnel_routes(config),
            )
            .map_err(Error::TunnelError)?,
        ))
    }

    /// Returns a close handle for the tunnel
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle {
            chan: self.close_msg_sender.clone(),
        }
    }

    /// Blocks the current thread until tunnel disconnects
    pub fn wait(mut self) -> Result<()> {
        let wait_result = match self.close_msg_receiver.recv() {
            Ok(CloseMsg::PingErr) => Err(Error::TimeoutError),
            Ok(CloseMsg::Stop) => Ok(()),
            Ok(CloseMsg::SetupError(error)) => Err(error),
            Err(_) => Ok(()),
        };

        #[cfg(windows)]
        if let Some(stop_tx) = self.stop_setup_tx.take() {
            let _ = stop_tx.send(());
        }
        let _ = self.pinger_stop_sender.send(());

        self.stop_tunnel();

        (self.event_callback)(TunnelEvent::Down);
        wait_result
    }

    fn stop_tunnel(&mut self) {
        match self.tunnel.lock().expect("Tunnel lock poisoned").take() {
            Some(tunnel) => {
                if let Err(e) = tunnel.stop() {
                    log::error!("{}", e.display_chain_with_msg("Failed to stop tunnel"));
                }
            }
            None => {
                log::debug!("Tunnel already stopped");
            }
        }
    }

    fn get_tunnel_routes(config: &Config) -> impl Iterator<Item = ipnetwork::IpNetwork> + '_ {
        let routes = config
            .peers
            .iter()
            .flat_map(|peer| peer.allowed_ips.iter())
            .cloned();
        #[cfg(target_os = "linux")]
        {
            routes
        }
        #[cfg(not(target_os = "linux"))]
        {
            routes.flat_map(|allowed_ip| {
                if allowed_ip.prefix() == 0 {
                    if allowed_ip.is_ipv4() {
                        vec!["0.0.0.0/1".parse().unwrap(), "128.0.0.0/1".parse().unwrap()]
                    } else {
                        vec!["8000::/1".parse().unwrap(), "::/1".parse().unwrap()]
                    }
                } else {
                    vec![allowed_ip]
                }
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn get_routes(iface_name: &str, config: &Config) -> HashSet<RequiredRoute> {
        let mut routes: HashSet<RequiredRoute> = {
            let node_v4 =
                routing::Node::new(config.ipv4_gateway.clone().into(), iface_name.to_string());
            let node_v6 = if let Some(ipv6_gateway) = config.ipv6_gateway.as_ref() {
                routing::Node::new(ipv6_gateway.clone().into(), iface_name.to_string())
            } else {
                routing::Node::device(iface_name.to_string())
            };
            Self::get_tunnel_routes(config)
                .map(|network| {
                    if network.is_ipv4() {
                        RequiredRoute::new(network, node_v4.clone())
                    } else {
                        RequiredRoute::new(network, node_v6.clone())
                    }
                })
                .collect()
        };

        // route endpoints with specific routes
        for peer in config.peers.iter() {
            routes.insert(RequiredRoute::new(
                peer.endpoint.ip().into(),
                routing::NetNode::DefaultNode,
            ));
        }

        routes
    }

    #[cfg(target_os = "linux")]
    fn get_routes(iface_name: &str, config: &Config) -> HashSet<RequiredRoute> {
        use netlink_packet_route::rtnl::constants::RT_TABLE_MAIN;

        let node = routing::Node::device(iface_name.to_string());
        let mut routes: HashSet<RequiredRoute> = Self::get_tunnel_routes(config)
            .map(|network| {
                if network.prefix() == 0 {
                    RequiredRoute::new(network, node.clone())
                } else {
                    RequiredRoute::new(network, node.clone()).table(u32::from(RT_TABLE_MAIN))
                }
            })
            .collect();

        // add routes for the gateway so that DNS requests can be made in the tunnel
        // using `mullvad-exclude`
        routes.insert(
            RequiredRoute::new(
                ipnetwork::Ipv4Network::from(config.ipv4_gateway).into(),
                node.clone(),
            )
            .table(u32::from(RT_TABLE_MAIN)),
        );

        if let Some(gateway) = config.ipv6_gateway {
            routes.insert(
                RequiredRoute::new(ipnetwork::Ipv6Network::from(gateway).into(), node.clone())
                    .table(u32::from(RT_TABLE_MAIN)),
            );
        }

        routes
    }

    #[cfg(all(not(target_os = "linux"), not(windows)))]
    fn get_routes(iface_name: &str, config: &Config) -> HashSet<RequiredRoute> {
        let node = routing::Node::device(iface_name.to_string());
        let mut routes: HashSet<RequiredRoute> = Self::get_tunnel_routes(config)
            .map(|network| RequiredRoute::new(network, node.clone()))
            .collect();

        // route endpoints with specific routes
        for peer in config.peers.iter() {
            routes.insert(RequiredRoute::new(
                peer.endpoint.ip().into(),
                routing::NetNode::DefaultNode,
            ));
        }

        routes
    }

    fn tunnel_metadata(interface_name: &str, config: &Config) -> TunnelMetadata {
        TunnelMetadata {
            interface: interface_name.to_string(),
            ips: config.tunnel.addresses.clone(),
            ipv4_gateway: config.ipv4_gateway,
            ipv6_gateway: config.ipv6_gateway,
        }
    }
}

enum CloseMsg {
    Stop,
    PingErr,
    SetupError(Error),
}

/// Close handle for a WireGuard tunnel.
#[derive(Clone, Debug)]
pub struct CloseHandle {
    chan: mpsc::Sender<CloseMsg>,
}

impl CloseHandle {
    /// Closes a WireGuard tunnel
    pub fn close(&mut self) {
        if let Err(e) = self.chan.send(CloseMsg::Stop) {
            log::trace!("Failed to send close message to wireguard tunnel: {}", e);
        }
    }
}

pub(crate) trait Tunnel: Send {
    fn get_interface_name(&self) -> String;
    #[cfg(target_os = "windows")]
    fn get_interface_luid(&self) -> u64;
    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError>;
    fn get_tunnel_stats(&self) -> std::result::Result<stats::Stats, TunnelError>;
    #[cfg(target_os = "linux")]
    fn slow_stats_refresh_rate(&self) {}
}

/// Errors to be returned from WireGuard implementations, namely implementers of the Tunnel trait
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum TunnelError {
    /// A recoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by wireguard-go that indicates that trying to establish the
    /// tunnel again should work normally. The error encountered is known to be sporadic.
    #[error(display = "Recoverable error while starting wireguard tunnel")]
    RecoverableStartWireguardError,

    /// An unrecoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by wireguard-go that indicates that trying to establish the
    /// tunnel again will likely fail with the same error. An error was encountered during tunnel
    /// configuration which can't be dealt with gracefully.
    #[error(display = "Failed to start wireguard tunnel")]
    FatalStartWireguardError,

    /// Failed to tear down wireguard tunnel.
    #[error(display = "Failed to stop wireguard tunnel. Status: {}", status)]
    StopWireguardError {
        /// Returned error code
        status: i32,
    },

    /// Error whilst trying to parse the WireGuard config to read the stats
    #[error(display = "Reading tunnel stats failed")]
    StatsError(#[error(source)] stats::Error),

    /// Error whilst trying to retrieve config of a WireGuard tunnel
    #[error(display = "Failed to get config of WireGuard tunnel")]
    GetConfigError,

    /// Failed to duplicate tunnel file descriptor for wireguard-go
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
    #[error(display = "Failed to duplicate tunnel file descriptor for wireguard-go")]
    FdDuplicationError(#[error(source)] nix::Error),

    /// Failed to setup a tunnel device.
    #[cfg(not(windows))]
    #[error(display = "Failed to create tunnel device")]
    SetupTunnelDeviceError(#[error(source)] tun_provider::Error),

    /// Failed to configure Wireguard sockets to bypass the tunnel.
    #[cfg(target_os = "android")]
    #[error(display = "Failed to configure Wireguard sockets to bypass the tunnel")]
    BypassError(#[error(source)] tun_provider::Error),

    /// Invalid tunnel interface name.
    #[error(display = "Invalid tunnel interface name")]
    InterfaceNameError(#[error(source)] std::ffi::NulError),

    /// Failed to convert adapter alias to UTF-8.
    #[cfg(target_os = "windows")]
    #[error(display = "Failed to convert adapter alias")]
    InvalidAlias,

    /// Failure to set up logging
    #[error(display = "Failed to set up logging")]
    LoggingError(#[error(source)] logging::Error),
}
