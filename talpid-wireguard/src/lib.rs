//! Manage WireGuard tunnels.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

use self::config::Config;
use futures::future::{abortable, AbortHandle as FutureAbortHandle, BoxFuture, Future};
#[cfg(windows)]
use futures::{channel::mpsc, StreamExt};
#[cfg(target_os = "linux")]
use lazy_static::lazy_static;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(windows)]
use std::io;
use std::{
    borrow::Cow,
    convert::Infallible,
    net::IpAddr,
    path::Path,
    pin::Pin,
    sync::{mpsc as sync_mpsc, Arc, Mutex},
    time::Duration,
};
use talpid_routing as routing;
use talpid_routing::{self, RequiredRoute};
#[cfg(not(windows))]
use talpid_tunnel::tun_provider;
use talpid_tunnel::{tun_provider::TunProvider, TunnelArgs, TunnelEvent, TunnelMetadata};

#[cfg(windows)]
use talpid_types::BoxedError;
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig, wireguard::PublicKey, AllowedTunnelTraffic, Endpoint,
        TransportProtocol,
    },
    ErrorExt,
};
use tokio::sync::Mutex as AsyncMutex;
use tunnel_obfuscation::{
    create_obfuscator, Error as ObfuscationError, Settings as ObfuscationSettings, Udp2TcpSettings,
};

/// WireGuard config data-types
pub mod config;
mod connectivity_check;
mod logging;
mod ping_monitor;
mod stats;
mod wireguard_go;
#[cfg(target_os = "linux")]
pub(crate) mod wireguard_kernel;
#[cfg(windows)]
mod wireguard_nt;

use self::wireguard_go::WgGoTunnel;

type Result<T> = std::result::Result<T, Error>;
type EventCallback = Box<dyn (Fn(TunnelEvent) -> BoxFuture<'static, ()>) + Send + Sync + 'static>;

/// Errors that can happen in the Wireguard tunnel monitor.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to set up routing.
    #[error(display = "Failed to setup routing")]
    SetupRoutingError(#[error(source)] talpid_routing::Error),

    /// Tunnel timed out
    #[error(display = "Tunnel timed out")]
    TimeoutError,

    /// An interaction with a tunnel failed
    #[error(display = "Tunnel failed")]
    TunnelError(#[error(source)] TunnelError),

    /// Failed to create tunnel obfuscator
    #[error(display = "Failed to create tunnel obfuscator")]
    CreateObfuscatorError(#[error(source)] ObfuscationError),

    /// Failed to run tunnel obfuscator
    #[error(display = "Tunnel obfuscator failed")]
    ObfuscatorError(#[error(source)] ObfuscationError),

    /// Failed to set up connectivity monitor
    #[error(display = "Connectivity monitor failed")]
    ConnectivityMonitorError(#[error(source)] connectivity_check::Error),

    /// Failed to negotiate PQ PSK
    #[error(display = "Failed to negotiate PQ PSK")]
    PskNegotiationError(#[error(source)] talpid_tunnel_config_client::Error),

    /// Failed to set up IP interfaces.
    #[cfg(windows)]
    #[error(display = "Failed to set up IP interfaces")]
    IpInterfacesError,

    /// Failed to set IP addresses on WireGuard interface
    #[cfg(target_os = "windows")]
    #[error(display = "Failed to set IP addresses on WireGuard interface")]
    SetIpAddressesError(#[error(source)] talpid_windows_net::Error),
}

/// Spawns and monitors a wireguard tunnel
pub struct WireguardMonitor {
    runtime: tokio::runtime::Handle,
    /// Tunnel implementation
    tunnel: Arc<Mutex<Option<Box<dyn Tunnel>>>>,
    /// Callback to signal tunnel events
    event_callback: EventCallback,
    close_msg_receiver: sync_mpsc::Receiver<CloseMsg>,
    pinger_stop_sender: sync_mpsc::Sender<()>,
    obfuscator: Arc<AsyncMutex<Option<ObfuscatorHandle>>>,
}

const INITIAL_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(15);
const PSK_EXCHANGE_TIMEOUT_MULTIPLIER: u32 = 2;

/// Simple wrapper that automatically cancels the future which runs an obfuscator.
struct ObfuscatorHandle {
    abort_handle: FutureAbortHandle,
    #[cfg(target_os = "android")]
    remote_socket_fd: std::os::unix::io::RawFd,
}

impl ObfuscatorHandle {
    pub fn new(
        abort_handle: FutureAbortHandle,
        #[cfg(target_os = "android")] remote_socket_fd: std::os::unix::io::RawFd,
    ) -> Self {
        Self {
            abort_handle,
            #[cfg(target_os = "android")]
            remote_socket_fd,
        }
    }

    #[cfg(target_os = "android")]
    pub fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        self.remote_socket_fd
    }

    pub fn abort(&self) {
        self.abort_handle.abort();
    }
}

impl Drop for ObfuscatorHandle {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
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

async fn maybe_create_obfuscator(
    config: &mut Config,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
) -> Result<Option<ObfuscatorHandle>> {
    // There are one or two peers.
    // The first one is always the entry relay.
    let mut first_peer = config.peers.get_mut(0).expect("missing peer");

    if let Some(ref obfuscator_config) = config.obfuscator_config {
        match obfuscator_config {
            ObfuscatorConfig::Udp2Tcp { endpoint } => {
                log::trace!("Connecting to Udp2Tcp endpoint {:?}", *endpoint);
                let settings = Udp2TcpSettings {
                    peer: *endpoint,
                    #[cfg(target_os = "linux")]
                    fwmark: config.fwmark,
                };
                let obfuscator = create_obfuscator(&ObfuscationSettings::Udp2Tcp(settings))
                    .await
                    .map_err(Error::CreateObfuscatorError)?;
                let endpoint = obfuscator.endpoint();

                log::trace!("Patching first WireGuard peer to become {:?}", endpoint);
                first_peer.endpoint = endpoint;

                #[cfg(target_os = "android")]
                let remote_socket_fd = obfuscator.remote_socket_fd();

                let (runner, abort_handle) = abortable(async move {
                    match obfuscator.run().await {
                        Ok(_) => {
                            let _ = close_msg_sender.send(CloseMsg::ObfuscatorExpired);
                        }
                        Err(error) => {
                            log::error!(
                                "{}",
                                error.display_chain_with_msg("Obfuscation controller failed")
                            );
                            let _ = close_msg_sender
                                .send(CloseMsg::ObfuscatorFailed(Error::ObfuscatorError(error)));
                        }
                    }
                });
                tokio::spawn(runner);
                return Ok(Some(ObfuscatorHandle::new(
                    abort_handle,
                    #[cfg(target_os = "android")]
                    remote_socket_fd,
                )));
            }
        }
    }
    Ok(None)
}

impl WireguardMonitor {
    /// Starts a WireGuard tunnel with the given config
    pub fn start<
        F: (Fn(TunnelEvent) -> Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + Clone
            + 'static,
    >(
        mut config: Config,
        psk_negotiation: Option<PublicKey>,
        log_path: Option<&Path>,
        args: TunnelArgs<'_, F>,
    ) -> Result<WireguardMonitor> {
        let on_event = args.on_event;

        let endpoint_addrs: Vec<IpAddr> =
            config.peers.iter().map(|peer| peer.endpoint.ip()).collect();
        let (close_msg_sender, close_msg_receiver) = sync_mpsc::channel();

        let obfuscator = args.runtime.block_on(maybe_create_obfuscator(
            &mut config,
            close_msg_sender.clone(),
        ))?;

        #[cfg(target_os = "windows")]
        let (setup_done_tx, setup_done_rx) = mpsc::channel(0);

        let tunnel = Self::open_tunnel(
            args.runtime.clone(),
            &Self::patch_allowed_ips(&config, psk_negotiation.is_some()),
            log_path,
            args.resource_dir,
            args.tun_provider.clone(),
            #[cfg(target_os = "windows")]
            args.route_manager.clone(),
            #[cfg(target_os = "windows")]
            setup_done_tx,
        )?;
        let iface_name = tunnel.get_interface_name();

        #[cfg(target_os = "android")]
        if let Some(remote_socket_fd) = obfuscator.as_ref().map(|obfs| obfs.remote_socket_fd()) {
            // Exclude remote obfuscation socket or bridge
            log::debug!("Excluding remote socket fd from the tunnel");
            if let Err(error) = args.tun_provider.lock().unwrap().bypass(remote_socket_fd) {
                log::error!("Failed to exclude remote socket fd: {error}");
            }
        }

        let event_callback = Box::new(on_event.clone());
        let (pinger_tx, pinger_rx) = sync_mpsc::channel();
        let monitor = WireguardMonitor {
            runtime: args.runtime.clone(),
            tunnel: Arc::new(Mutex::new(Some(tunnel))),
            event_callback,
            close_msg_receiver,
            pinger_stop_sender: pinger_tx,
            obfuscator: Arc::new(AsyncMutex::new(obfuscator)),
        };

        let gateway = config.ipv4_gateway;
        let mut connectivity_monitor = connectivity_check::ConnectivityMonitor::new(
            gateway,
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            iface_name.clone(),
            Arc::downgrade(&monitor.tunnel),
            pinger_rx,
        )
        .map_err(Error::ConnectivityMonitorError)?;

        let metadata = Self::tunnel_metadata(&iface_name, &config);
        let tunnel = monitor.tunnel.clone();
        let obfs_handle = monitor.obfuscator.clone();
        let obfs_close_sender = close_msg_sender.clone();

        let tunnel_fut = async move {
            #[cfg(windows)]
            Self::add_device_ip_addresses(&iface_name, &config.tunnel.addresses, setup_done_rx)
                .await?;

            let allowed_traffic = if psk_negotiation.is_some() {
                AllowedTunnelTraffic::Only(Endpoint::new(
                    config.ipv4_gateway,
                    talpid_tunnel_config_client::CONFIG_SERVICE_PORT,
                    TransportProtocol::Tcp,
                ))
            } else {
                AllowedTunnelTraffic::All
            };
            (on_event)(TunnelEvent::InterfaceUp(metadata.clone(), allowed_traffic)).await;

            // Add non-default routes before establishing the tunnel.
            #[cfg(target_os = "linux")]
            args.route_manager
                .create_routing_rules(config.enable_ipv6)
                .await
                .map_err(Error::SetupRoutingError)
                .map_err(CloseMsg::SetupError)?;

            let routes = Self::get_pre_tunnel_routes(&iface_name, &config)
                .chain(Self::get_endpoint_routes(&endpoint_addrs))
                .collect();
            args.route_manager
                .add_routes(routes)
                .await
                .map_err(Error::SetupRoutingError)
                .map_err(CloseMsg::SetupError)?;

            if let Some(pubkey) = psk_negotiation {
                Self::perform_psk_negotiation(
                    tunnel,
                    obfs_handle,
                    obfs_close_sender,
                    args.retry_attempt,
                    pubkey,
                    &mut config,
                )
                .await?;
                (on_event)(TunnelEvent::InterfaceUp(
                    metadata.clone(),
                    AllowedTunnelTraffic::All,
                ))
                .await;
            }

            let mut connectivity_monitor = tokio::task::spawn_blocking(move || {
                match connectivity_monitor.establish_connectivity(args.retry_attempt) {
                    Ok(true) => Ok(connectivity_monitor),
                    Ok(false) => {
                        log::warn!("Timeout while checking tunnel connection");
                        Err(CloseMsg::PingErr)
                    }
                    Err(error) => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to check tunnel connection")
                        );
                        Err(CloseMsg::PingErr)
                    }
                }
            })
            .await
            .unwrap()?;

            // Add any default route(s) that may exist.
            args.route_manager
                .add_routes(Self::get_post_tunnel_routes(&iface_name, &config).collect())
                .await
                .map_err(Error::SetupRoutingError)
                .map_err(CloseMsg::SetupError)?;

            (on_event)(TunnelEvent::Up(metadata)).await;

            tokio::task::spawn_blocking(move || {
                if let Err(error) = connectivity_monitor.run() {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Connectivity monitor failed")
                    );
                }
            })
            .await
            .unwrap();

            Err::<Infallible, CloseMsg>(CloseMsg::PingErr)
        };

        let close_sender = close_msg_sender.clone();
        let monitor_handle = tokio::spawn(async move {
            // This is safe to unwrap because the future resolves to `Result<Infallible, E>`.
            let close_msg = tunnel_fut.await.unwrap_err();
            let _ = close_sender.send(close_msg);
        });

        tokio::spawn(async move {
            if args.tunnel_close_rx.await.is_ok() {
                monitor_handle.abort();
                let _ = close_msg_sender.send(CloseMsg::Stop);
            }
        });

        Ok(monitor)
    }

    /// Replace `0.0.0.0/0`/`::/0` with the gateway IPs when `gateway_only` is true.
    /// Used to block traffic to other destinations while connecting on Android.
    fn patch_allowed_ips(config: &Config, gateway_only: bool) -> Cow<'_, Config> {
        if gateway_only {
            let mut patched_config = config.clone();
            let gateway_net_v4 = ipnetwork::IpNetwork::from(IpAddr::from(config.ipv4_gateway));
            let gateway_net_v6 = config
                .ipv6_gateway
                .map(|net| ipnetwork::IpNetwork::from(IpAddr::from(net)));
            for peer in &mut patched_config.peers {
                peer.allowed_ips = peer
                    .allowed_ips
                    .iter()
                    .cloned()
                    .filter_map(|mut allowed_ip| {
                        if allowed_ip.prefix() == 0 {
                            if allowed_ip.is_ipv4() {
                                allowed_ip = gateway_net_v4;
                            } else if let Some(net) = gateway_net_v6 {
                                allowed_ip = net;
                            } else {
                                return None;
                            }
                        }
                        Some(allowed_ip)
                    })
                    .collect();
            }
            Cow::Owned(patched_config)
        } else {
            Cow::Borrowed(config)
        }
    }

    #[cfg(windows)]
    async fn add_device_ip_addresses(
        iface_name: &str,
        addresses: &[IpAddr],
        mut setup_done_rx: mpsc::Receiver<std::result::Result<(), BoxedError>>,
    ) -> std::result::Result<(), CloseMsg> {
        setup_done_rx
            .next()
            .await
            .ok_or_else(|| {
                // Tunnel was shut down early
                CloseMsg::SetupError(Error::IpInterfacesError)
            })?
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to configure tunnel interface")
                );
                CloseMsg::SetupError(Error::IpInterfacesError)
            })?;

        // TODO: The LUID can be obtained directly.
        let luid = talpid_windows_net::luid_from_alias(iface_name).map_err(|error| {
            log::error!("Failed to obtain tunnel interface LUID: {}", error);
            CloseMsg::SetupError(Error::IpInterfacesError)
        })?;
        for address in addresses {
            talpid_windows_net::add_ip_address_for_interface(luid, *address)
                .map_err(|error| CloseMsg::SetupError(Error::SetIpAddressesError(error)))?;
        }
        Ok(())
    }

    async fn perform_psk_negotiation(
        tunnel: Arc<Mutex<Option<Box<dyn Tunnel>>>>,
        obfuscation_handle: Arc<AsyncMutex<Option<ObfuscatorHandle>>>,
        obfs_close_sender: sync_mpsc::Sender<CloseMsg>,
        retry_attempt: u32,
        current_pubkey: PublicKey,
        config: &mut Config,
    ) -> std::result::Result<(), CloseMsg> {
        log::debug!("Performing PQ-safe PSK exchange");

        let timeout = std::cmp::min(
            MAX_PSK_EXCHANGE_TIMEOUT,
            INITIAL_PSK_EXCHANGE_TIMEOUT
                .saturating_mul(PSK_EXCHANGE_TIMEOUT_MULTIPLIER.saturating_pow(retry_attempt)),
        );

        let (private_key, psk) = tokio::time::timeout(
            timeout,
            talpid_tunnel_config_client::push_pq_key(
                IpAddr::V4(config.ipv4_gateway),
                config.tunnel.private_key.public_key(),
            ),
        )
        .await
        .map_err(|_timeout_err| {
            log::warn!("Timeout while negotiating PSK");
            CloseMsg::PskNegotiationTimeout
        })?
        .map_err(Error::PskNegotiationError)
        .map_err(CloseMsg::SetupError)?;

        config.tunnel.private_key = private_key;

        for peer in &mut config.peers {
            if current_pubkey == peer.public_key {
                peer.psk = Some(psk);
                break;
            }
        }

        log::trace!(
            "Ephemeral pubkey: {}",
            config.tunnel.private_key.public_key()
        );

        // Restart the obfuscation server
        let mut obfs_guard = obfuscation_handle.lock().await;
        if let Some(obfs_abort_handle) = obfs_guard.take() {
            obfs_abort_handle.abort();
            *obfs_guard = maybe_create_obfuscator(config, obfs_close_sender)
                .await
                .map_err(CloseMsg::SetupError)?;
        }

        let set_config_future = tunnel
            .lock()
            .unwrap()
            .as_ref()
            .map(|tunnel| tunnel.set_config(config.clone()));
        if let Some(f) = set_config_future {
            f.await
                .map_err(Error::TunnelError)
                .map_err(CloseMsg::SetupError)?;
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
        tun_provider: Arc<Mutex<TunProvider>>,
        #[cfg(windows)] route_manager_handle: crate::routing::RouteManagerHandle,
        #[cfg(windows)] setup_done_tx: mpsc::Sender<std::result::Result<(), BoxedError>>,
    ) -> Result<Box<dyn Tunnel>> {
        #[cfg(target_os = "linux")]
        if !*FORCE_USERSPACE_WIREGUARD {
            if will_nm_manage_dns() {
                match wireguard_kernel::NetworkManagerTunnel::new(runtime, config) {
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
                match wireguard_kernel::NetlinkTunnel::new(runtime, config) {
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

        #[cfg(target_os = "windows")]
        if config.use_wireguard_nt {
            log::debug!("Using WireGuardNT");
            return wireguard_nt::WgNtTunnel::start_tunnel(
                config,
                log_path,
                resource_dir,
                setup_done_tx,
            )
            .map(|tun| Box::new(tun) as Box<dyn Tunnel + 'static>)
            .map_err(Error::TunnelError);
        }

        #[cfg(any(target_os = "linux", windows))]
        log::debug!("Using userspace WireGuard implementation");
        Ok(Box::new(
            WgGoTunnel::start_tunnel(
                config,
                log_path,
                #[cfg(not(windows))]
                tun_provider,
                #[cfg(not(windows))]
                Self::get_tunnel_destinations(config).flat_map(Self::replace_default_prefixes),
                #[cfg(windows)]
                route_manager_handle,
                #[cfg(windows)]
                setup_done_tx,
                #[cfg(windows)]
                &runtime,
            )
            .map_err(Error::TunnelError)?,
        ))
    }

    /// Blocks the current thread until tunnel disconnects
    pub fn wait(mut self) -> Result<()> {
        let wait_result = match self.close_msg_receiver.recv() {
            Ok(CloseMsg::PskNegotiationTimeout) | Ok(CloseMsg::PingErr) => Err(Error::TimeoutError),
            Ok(CloseMsg::Stop) | Ok(CloseMsg::ObfuscatorExpired) => Ok(()),
            Ok(CloseMsg::SetupError(error)) => Err(error),
            Ok(CloseMsg::ObfuscatorFailed(error)) => Err(error),
            Err(_) => Ok(()),
        };

        let _ = self.pinger_stop_sender.send(());

        self.runtime
            .block_on((self.event_callback)(TunnelEvent::Down));

        self.stop_tunnel();

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

    /// Returns routes to the peer endpoints (through the physical interface).
    #[cfg_attr(target_os = "linux", allow(unused_variables))]
    fn get_endpoint_routes(endpoints: &[IpAddr]) -> impl Iterator<Item = RequiredRoute> + '_ {
        #[cfg(target_os = "linux")]
        {
            // No need due to policy based routing.
            std::iter::empty::<RequiredRoute>()
        }
        #[cfg(not(target_os = "linux"))]
        endpoints.iter().map(|ip| {
            RequiredRoute::new(
                ipnetwork::IpNetwork::from(*ip),
                routing::NetNode::DefaultNode,
            )
        })
    }

    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    fn get_tunnel_nodes(iface_name: &str, config: &Config) -> (routing::Node, routing::Node) {
        #[cfg(windows)]
        {
            let v4 = routing::Node::new(config.ipv4_gateway.clone().into(), iface_name.to_string());
            let v6 = if let Some(ipv6_gateway) = config.ipv6_gateway.as_ref() {
                routing::Node::new(ipv6_gateway.clone().into(), iface_name.to_string())
            } else {
                routing::Node::device(iface_name.to_string())
            };
            (v4, v6)
        }

        #[cfg(not(windows))]
        {
            let node = routing::Node::device(iface_name.to_string());
            (node.clone(), node)
        }
    }

    /// Return routes for all allowed IPs, as well as the gateway, except 0.0.0.0/0.
    fn get_pre_tunnel_routes<'a>(
        iface_name: &str,
        config: &'a Config,
    ) -> impl Iterator<Item = RequiredRoute> + 'a {
        let gateway_node = routing::Node::device(iface_name.to_string());
        let gateway_routes = std::iter::once(RequiredRoute::new(
            ipnetwork::Ipv4Network::from(config.ipv4_gateway).into(),
            gateway_node.clone(),
        ))
        .chain(config.ipv6_gateway.map(|gateway| {
            RequiredRoute::new(ipnetwork::Ipv6Network::from(gateway).into(), gateway_node)
        }));

        let (node_v4, node_v6) = Self::get_tunnel_nodes(iface_name, config);

        let routes = gateway_routes.chain(
            Self::get_tunnel_destinations(config)
                .filter(|allowed_ip| allowed_ip.prefix() != 0)
                .map(move |allowed_ip| {
                    if allowed_ip.is_ipv4() {
                        RequiredRoute::new(allowed_ip, node_v4.clone())
                    } else {
                        RequiredRoute::new(allowed_ip, node_v6.clone())
                    }
                }),
        );

        routes
    }

    /// Return any 0.0.0.0/0 routes specified by the allowed IPs.
    fn get_post_tunnel_routes<'a>(
        iface_name: &str,
        config: &'a Config,
    ) -> impl Iterator<Item = RequiredRoute> + 'a {
        let (node_v4, node_v6) = Self::get_tunnel_nodes(iface_name, config);
        let iter = Self::get_tunnel_destinations(config)
            .filter(|allowed_ip| allowed_ip.prefix() == 0)
            .flat_map(Self::replace_default_prefixes)
            .map(move |allowed_ip| {
                if allowed_ip.is_ipv4() {
                    RequiredRoute::new(allowed_ip, node_v4.clone())
                } else {
                    RequiredRoute::new(allowed_ip, node_v6.clone())
                }
            });
        #[cfg(not(target_os = "linux"))]
        return iter;

        #[cfg(target_os = "linux")]
        iter.map(|route| route.use_main_table(false))
    }

    /// Return routes for all allowed IPs.
    fn get_tunnel_destinations(config: &Config) -> impl Iterator<Item = ipnetwork::IpNetwork> + '_ {
        config
            .peers
            .iter()
            .flat_map(|peer| peer.allowed_ips.iter())
            .cloned()
    }

    /// Replace default (0-prefix) routes with more specific routes.
    fn replace_default_prefixes(network: ipnetwork::IpNetwork) -> Vec<ipnetwork::IpNetwork> {
        #[cfg(not(target_os = "linux"))]
        if network.prefix() == 0 {
            if network.is_ipv4() {
                vec!["0.0.0.0/1".parse().unwrap(), "128.0.0.0/1".parse().unwrap()]
            } else {
                vec!["8000::/1".parse().unwrap(), "::/1".parse().unwrap()]
            }
        } else {
            vec![network]
        }

        #[cfg(target_os = "linux")]
        vec![network]
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
    PskNegotiationTimeout,
    PingErr,
    SetupError(Error),
    ObfuscatorExpired,
    ObfuscatorFailed(Error),
}

pub(crate) trait Tunnel: Send {
    fn get_interface_name(&self) -> String;
    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError>;
    fn get_tunnel_stats(&self) -> std::result::Result<stats::StatsMap, TunnelError>;
    fn set_config(
        &self,
        _config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send>>;
}

/// Errors to be returned from WireGuard implementations, namely implementers of the Tunnel trait
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum TunnelError {
    /// A recoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by the implementation that indicates that trying to establish the
    /// tunnel again should work normally. The error encountered is known to be sporadic.
    #[error(display = "Recoverable error while starting wireguard tunnel")]
    RecoverableStartWireguardError,

    /// An unrecoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by the implementation that indicates that trying to establish the
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

    /// Failed to set WireGuard tunnel config on device
    #[error(display = "Failed to set config of WireGuard tunnel")]
    SetConfigError,

    /// Failed to duplicate tunnel file descriptor for wireguard-go
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
    #[error(display = "Failed to duplicate tunnel file descriptor for wireguard-go")]
    FdDuplicationError(#[error(source)] nix::Error),

    /// Failed to setup a tunnel device.
    #[cfg(not(windows))]
    #[error(display = "Failed to create tunnel device")]
    SetupTunnelDeviceError(#[error(source)] tun_provider::Error),

    /// Failed to setup a tunnel device.
    #[cfg(windows)]
    #[error(display = "Failed to config IP interfaces on tunnel device")]
    SetupIpInterfaces(#[error(source)] io::Error),

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

#[cfg(target_os = "linux")]
fn will_nm_manage_dns() -> bool {
    use talpid_dbus::network_manager::NetworkManager;

    if talpid_dbus::systemd_resolved::SystemdResolved::new().is_ok() {
        return false;
    }

    NetworkManager::new()
        .and_then(|nm| {
            nm.ensure_can_be_used_to_manage_dns()?;
            Ok(true)
        })
        .unwrap_or(false)
}
