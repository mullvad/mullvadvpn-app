use super::super::routing;
use talpid_types::net::{TunnelOptions, WireguardEndpointData};

use super::{TunnelEvent, TunnelMetadata};
use std::{net::IpAddr, path::Path};

pub mod config;
mod ping_monitor;
pub mod wireguard_go;

use self::config::Config;
pub use self::wireguard_go::WgGoTunnel;

// amount of seconds to run `ping` until it returns.
const PING_TIMEOUT: u64 = 5;

error_chain! {
    errors {
        /// Failed to setup a tunnel device
        SetupTunnelDeviceError {
            description("Failed to create tunnel device")
        }

        /// Failed to setup wireguard tunnel
        StartWireguardError(status: i32) {
            display("Failed to start wireguard tunnel - {}", status)
        }
        /// Failed to tear down wireguard tunnel
        StopWireguardError(status: i32) {
            display("Failed to stop wireguard tunnel - {}", status)
        }
        /// Failed to set up routing
        SetupRoutingError {
            display("Failed to setup routing")
        }

        /// Failed to move or craete a log file
        PrepareLogFileError {
            display("Failed to setup a logging file")
        }

        /// Failed to run
        PingError {
            display("Failed to run ping")
        }

        /// Pinging timed out
        PingTimeoutError {
            display("Ping timed out")
        }
    }
}

pub trait CloseHandle: Sync + Send {
    fn close(&mut self);
    fn close_with_error(&mut self, err: Error);
}

pub trait Tunnel: Send {
    fn get_interface_name(&self) -> &str;
    fn close_handle(&self) -> Box<dyn CloseHandle>;
    fn wait(self: Box<Self>) -> Result<()>;
}

/// Spawns and monitors a wireguard tunnel
pub struct WireguardMonitor {
    /// Tunnel implementation
    tunnel: Box<Tunnel>,
    /// Route manager
    router: routing::RouteManager,
    /// Callback to signal tunnel events
    event_callback: Box<Fn(TunnelEvent) + Send + Sync + 'static>,
}

impl WireguardMonitor {
    pub fn start<F: Fn(TunnelEvent) + Send + Sync + 'static>(
        address: IpAddr,
        data: WireguardEndpointData,
        options: &TunnelOptions,
        log_path: Option<&Path>,
        on_event: F,
    ) -> Result<WireguardMonitor> {
        let config = Config::from_data(address, data.clone(), options);
        let tunnel = Box::new(WgGoTunnel::start_tunnel(&config, log_path)?);
        let router = routing::RouteManager::new().chain_err(|| ErrorKind::SetupRoutingError)?;
        let event_callback = Box::new(on_event);

        let mut monitor = WireguardMonitor {
            tunnel,
            router,
            event_callback,
        };
        monitor.setup_routing(&config)?;
        monitor.start_pinger(&config);
        monitor.tunnel_up(data);

        Ok(monitor)
    }

    pub fn close_handle(&self) -> Box<CloseHandle> {
        self.tunnel.close_handle()
    }

    pub fn wait(self) -> Result<()> {
        let result = self.tunnel.wait();
        (self.event_callback)(TunnelEvent::Down);
        result
    }

    #[allow(unused_mut)]
    fn setup_routing(&mut self, config: &Config) -> Result<()> {
        let iface_name = self.tunnel.get_interface_name();
        let mut routes: Vec<_> = config
            .interface
            .peers
            .iter()
            .flat_map(|peer| peer.allowed_ips.iter())
            .cloned()
            .map(|allowed_ip| {
                routing::Route::new(allowed_ip, routing::NetNode::Device(iface_name.to_string()))
            })
            .collect();

        #[cfg(target_os = "macos")]
        {
            // To survive network roaming on osx, we should listen for new routes and reapply them
            // here - probably would need RouteManager be extended. Or maybe RouteManager can deal
            // with it on it's own
            let default_node = self
                .router
                .get_default_route_node()
                .chain_err(|| ErrorKind::SetupRoutingError)?;
            // route endpoints with specific routes
            for peer in config.interface.peers.iter() {
                let default_route = routing::Route::new(
                    peer.endpoint.ip().clone().into(),
                    routing::NetNode::Address(default_node.clone()),
                );
                routes.push(default_route);
            }
        }

        let required_routes = routing::RequiredRoutes {
            routes,
            fwmark: Some(config.interface.fwmark.to_string()),
        };
        self.router
            .add_routes(required_routes)
            .chain_err(|| ErrorKind::SetupRoutingError)
    }

    fn start_pinger(&self, config: &Config) {
        let pinger = ping_monitor::PingMonitor::new(
            config.pingable_address,
            self.tunnel.get_interface_name().to_string(),
            self.tunnel.close_handle(),
        );
        pinger.monitor_ping(PING_TIMEOUT);
    }

    fn tunnel_up(&self, data: WireguardEndpointData) {
        let interface_name = self.tunnel.get_interface_name();
        let metadata = TunnelMetadata {
            interface: interface_name.to_string(),
            ip: data.addresses,
            gateway: data.gateway,
        };
        (self.event_callback)(TunnelEvent::Up(metadata));
    }
}
