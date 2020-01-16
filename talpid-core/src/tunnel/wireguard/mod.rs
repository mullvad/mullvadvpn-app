#![allow(missing_docs)]

use self::config::Config;
#[cfg(not(windows))]
use super::tun_provider;
use super::{tun_provider::TunProvider, TunnelEvent, TunnelMetadata};
use crate::{ping_monitor, routing};
use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

pub mod config;
mod connectivity_check;
mod stats;
pub mod wireguard_go;

pub use self::wireguard_go::WgGoTunnel;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Wireguard tunnel monitor.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to setup a tunnel device.
    #[cfg(not(windows))]
    #[error(display = "Failed to create tunnel device")]
    SetupTunnelDeviceError(#[error(source)] tun_provider::Error),

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
    #[error(display = "Failed to stop wireguard tunnel - {}", status)]
    StopWireguardError { status: i32 },

    /// Failed to get tunnel config
    #[error(display = "Failed to obtain tunnel config")]
    GetConfigError,

    /// Failed to set ip addresses on tunnel interface.
    #[cfg(target_os = "windows")]
    #[error(display = "Failed to set IP addresses on WireGuard interface")]
    SetIpAddressesError,

    /// Failed to set up routing.
    #[error(display = "Failed to setup routing")]
    SetupRoutingError(#[error(source)] crate::routing::Error),

    /// Failed to move or craete a log file.
    #[error(display = "Failed to setup a logging file")]
    PrepareLogFileError(#[error(source)] io::Error),

    /// Invalid tunnel interface name.
    #[error(display = "Invalid tunnel interface name")]
    InterfaceNameError(#[error(source)] std::ffi::NulError),

    /// Failed to configure Wireguard sockets to bypass the tunnel.
    #[cfg(target_os = "android")]
    #[error(display = "Failed to configure Wireguard sockets to bypass the tunnel")]
    BypassError(#[error(source)] tun_provider::Error),

    /// Failed to duplicate tunnel file descriptor for wireguard-go
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
    #[error(display = "Failed to duplicate tunnel file descriptor for wireguard-go")]
    FdDuplicationError(#[error(source)] nix::Error),

    /// Error whilst trying to read stats
    #[error(display = "Reading tunnel stats failed")]
    StatsError(#[error(source)] stats::Error),

    /// Tunnel handle is invalid
    #[error(display = "Tunnel handle is invalid")]
    InvalidTunnelHandle,

    /// Pinging timed out.
    #[error(display = "Ping timed out")]
    PingError(#[error(source)] ping_monitor::Error),

    /// Tunnel timed out
    #[error(display = "Tunnel timed out")]
    TimeoutError,
}


/// Spawns and monitors a wireguard tunnel
pub struct WireguardMonitor {
    /// Tunnel implementation
    tunnel: Arc<Mutex<Option<Box<dyn Tunnel>>>>,
    /// Route manager
    route_handle: routing::RouteManager,
    /// Callback to signal tunnel events
    event_callback: Box<dyn Fn(TunnelEvent) + Send + Sync + 'static>,
    close_msg_sender: mpsc::Sender<CloseMsg>,
    close_msg_receiver: mpsc::Receiver<CloseMsg>,
    pinger_stop_sender: mpsc::Sender<()>,
}

impl WireguardMonitor {
    pub fn start<F: Fn(TunnelEvent) + Send + Sync + Clone + 'static>(
        config: &Config,
        log_path: Option<&Path>,
        on_event: F,
        tun_provider: &mut TunProvider,
    ) -> Result<WireguardMonitor> {
        let tunnel = Box::new(WgGoTunnel::start_tunnel(
            &config,
            log_path,
            tun_provider,
            Self::get_tunnel_routes(config),
        )?);
        let iface_name = tunnel.get_interface_name().to_string();
        #[cfg_attr(not(windows), allow(unused_mut))]
        let mut route_handle = routing::RouteManager::new(Self::get_routes(&iface_name, &config))
            .map_err(Error::SetupRoutingError)?;

        #[cfg(target_os = "windows")]
        route_handle
            .add_default_route_callback(Some(WgGoTunnel::default_route_changed_callback), ());


        let event_callback = Box::new(on_event.clone());
        let (close_msg_sender, close_msg_receiver) = mpsc::channel();
        let (pinger_tx, pinger_rx) = mpsc::channel();
        let monitor = WireguardMonitor {
            tunnel: Arc::new(Mutex::new(Some(tunnel))),
            route_handle,
            event_callback,
            close_msg_sender,
            close_msg_receiver,
            pinger_stop_sender: pinger_tx,
        };

        let metadata = Self::tunnel_metadata(&iface_name, &config);
        let gateway = config.ipv4_gateway;
        let close_sender = monitor.close_msg_sender.clone();
        let mut connectivity_monitor = connectivity_check::ConnectivityMonitor::new(
            gateway,
            iface_name,
            Arc::downgrade(&monitor.tunnel),
            pinger_rx,
        )?;

        std::thread::spawn(move || {
            match connectivity_monitor.establish_connectivity() {
                Ok(true) => (on_event)(TunnelEvent::Up(metadata)),
                Ok(false) => (on_event)(TunnelEvent::Down),
                Err(err) => {
                    log::error!("ConnectivityMonitor failed: {}", err);
                    (on_event)(TunnelEvent::Down);
                }
            }
            if let Err(err) = connectivity_monitor.wait() {
                log::error!("Connectivity monitor failed - {}", err);
            }

            let _ = close_sender.send(CloseMsg::PingErr);
        });

        Ok(monitor)
    }

    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle {
            chan: self.close_msg_sender.clone(),
        }
    }

    pub fn wait(mut self) -> Result<()> {
        let wait_result = match self.close_msg_receiver.recv() {
            Ok(CloseMsg::PingErr) => Err(Error::TimeoutError),
            Ok(CloseMsg::Stop) => Ok(()),
            Err(_) => Ok(()),
        };

        let _ = self.pinger_stop_sender.send(());

        // Clear routes manually - otherwise there will be some log spam since the tunnel device
        // can be removed before the routes are cleared, which automatically clears some of the
        // routes that were set.
        self.route_handle.stop();

        self.stop_tunnel();

        (self.event_callback)(TunnelEvent::Down);
        wait_result
    }

    fn stop_tunnel(&mut self) {
        match self.tunnel.lock().expect("Tunnel lock poisoned").take() {
            Some(tunnel) => {
                if let Err(e) = tunnel.stop() {
                    log::error!("Failed to stop tunnel - {}", e);
                }
            }
            None => {
                log::debug!("Tunnel already stopped");
            }
        }
    }

    fn get_tunnel_routes(config: &Config) -> impl Iterator<Item = ipnetwork::IpNetwork> + '_ {
        config
            .peers
            .iter()
            .flat_map(|peer| peer.allowed_ips.iter())
            .cloned()
            .flat_map(|allowed_ip| {
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

    fn get_routes(
        iface_name: &str,
        config: &Config,
    ) -> HashMap<ipnetwork::IpNetwork, crate::routing::NetNode> {
        let node = routing::Node::device(iface_name.to_string());
        let mut routes: HashMap<_, _> = Self::get_tunnel_routes(config)
            .map(|network| (network, node.clone().into()))
            .collect();

        // route endpoints with specific routes
        for peer in config.peers.iter() {
            routes.insert(peer.endpoint.ip().into(), routing::NetNode::DefaultNode);
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
}

#[derive(Clone, Debug)]
pub struct CloseHandle {
    chan: mpsc::Sender<CloseMsg>,
}

impl CloseHandle {
    pub fn close(&mut self) {
        if let Err(e) = self.chan.send(CloseMsg::Stop) {
            log::trace!("Failed to send close message to wireguard tunnel - {}", e);
        }
    }
}

pub trait Tunnel: Send {
    fn get_interface_name(&self) -> &str;
    fn stop(self: Box<Self>) -> Result<()>;
    fn get_config(&self) -> Result<stats::Stats>;
}
