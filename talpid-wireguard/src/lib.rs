//! Manage WireGuard tunnels.

#![deny(missing_docs)]

use self::config::Config;
use futures::future::{abortable, AbortHandle as FutureAbortHandle, BoxFuture, Future};
#[cfg(windows)]
use futures::{channel::mpsc, StreamExt};
#[cfg(target_os = "linux")]
use once_cell::sync::Lazy;
#[cfg(target_os = "android")]
use std::borrow::Cow;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(windows)]
use std::io;
use std::{
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

use ipnetwork::IpNetwork;
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig,
        wireguard::{PresharedKey, PrivateKey, PublicKey},
        AllowedTunnelTraffic, Endpoint, TransportProtocol,
    },
    BoxedError, ErrorExt,
};
use tokio::sync::Mutex as AsyncMutex;
use tunnel_obfuscation::{
    create_obfuscator, Error as ObfuscationError, Settings as ObfuscationSettings, Udp2TcpSettings,
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use talpid_tunnel::{IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, WIREGUARD_HEADER_SIZE};

/// WireGuard config data-types
pub mod config;
mod connectivity_check;
mod logging;
mod ping_monitor;
mod stats;
#[cfg(target_os = "linux")]
mod unix;
#[cfg(wireguard_go)]
mod wireguard_go;
#[cfg(target_os = "linux")]
pub(crate) mod wireguard_kernel;
#[cfg(windows)]
mod wireguard_nt;

#[cfg(wireguard_go)]
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

    /// Failed to set MTU
    #[error(display = "Failed to detect MTU because every ping was dropped.")]
    MtuDetectionAllDropped,

    /// Failed to set MTU
    #[error(display = "Failed to detect MTU because of unexpected ping error.")]
    MtuDetectionPingError(#[error(source)] surge_ping::SurgeError),

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
    SetIpAddressesError(#[error(source)] talpid_windows::net::Error),
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::CreateObfuscatorError(_) => true,
            Error::ObfuscatorError(_) => true,
            Error::PskNegotiationError(_) => true,
            Error::TunnelError(TunnelError::RecoverableStartWireguardError) => true,

            Error::SetupRoutingError(error) => error.is_recoverable(),

            #[cfg(target_os = "android")]
            Error::TunnelError(TunnelError::BypassError(_)) => true,

            #[cfg(windows)]
            _ => self.get_tunnel_device_error().is_some(),

            #[cfg(not(windows))]
            _ => false,
        }
    }

    /// Get the inner tunnel device error, if there is one
    #[cfg(windows)]
    pub fn get_tunnel_device_error(&self) -> Option<&io::Error> {
        match self {
            Error::TunnelError(TunnelError::SetupTunnelDevice(error)) => Some(error),
            _ => None,
        }
    }
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

const INITIAL_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(48);
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
/// Overrides the preference for the kernel module for WireGuard.
static FORCE_USERSPACE_WIREGUARD: Lazy<bool> = Lazy::new(|| {
    env::var("TALPID_FORCE_USERSPACE_WIREGUARD")
        .map(|v| v != "0")
        .unwrap_or(false)
});

async fn maybe_create_obfuscator(
    config: &mut Config,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
) -> Result<Option<ObfuscatorHandle>> {
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
                config.entry_peer.endpoint = endpoint;

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
        psk_negotiation: bool,
        log_path: Option<&Path>,
        args: TunnelArgs<'_, F>,
    ) -> Result<WireguardMonitor> {
        let on_event = args.on_event.clone();

        let endpoint_addrs: Vec<IpAddr> = config.peers().map(|peer| peer.endpoint.ip()).collect();

        let (close_obfs_sender, close_obfs_listener) = sync_mpsc::channel();
        let obfuscator = args.runtime.block_on(maybe_create_obfuscator(
            &mut config,
            close_obfs_sender.clone(),
        ))?;

        #[cfg(target_os = "windows")]
        let (setup_done_tx, setup_done_rx) = mpsc::channel(0);
        let tunnel = Self::open_tunnel(
            args.runtime.clone(),
            &config,
            log_path,
            args.resource_dir,
            args.tun_provider.clone(),
            #[cfg(target_os = "windows")]
            args.route_manager.clone(),
            #[cfg(target_os = "windows")]
            setup_done_tx,
            #[cfg(target_os = "android")]
            psk_negotiation,
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

        let obfuscator = Arc::new(AsyncMutex::new(obfuscator));

        let event_callback = Box::new(on_event.clone());
        let (pinger_tx, pinger_rx) = sync_mpsc::channel();
        let monitor = WireguardMonitor {
            runtime: args.runtime.clone(),
            tunnel: Arc::new(Mutex::new(Some(tunnel))),
            event_callback,
            close_msg_receiver: close_obfs_listener,
            pinger_stop_sender: pinger_tx,
            obfuscator,
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

        let moved_tunnel = monitor.tunnel.clone();
        let moved_close_obfs_sender = close_obfs_sender.clone();
        let moved_obfuscator = monitor.obfuscator.clone();
        let tunnel_fut = async move {
            let tunnel = moved_tunnel;
            let close_obfs_sender: sync_mpsc::Sender<CloseMsg> = moved_close_obfs_sender;
            let obfuscator = moved_obfuscator;
            #[cfg(windows)]
            Self::add_device_ip_addresses(&iface_name, &config.tunnel.addresses, setup_done_rx)
                .await?;

            let metadata = Self::tunnel_metadata(&iface_name, &config);
            let allowed_traffic = if psk_negotiation {
                AllowedTunnelTraffic::One(Endpoint::new(
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

            let psk_obfs_sender = close_obfs_sender.clone();
            if psk_negotiation {
                Self::psk_negotiation(
                    &tunnel,
                    &mut config,
                    args.retry_attempt,
                    args.on_event.clone(),
                    &iface_name,
                    obfuscator.clone(),
                    psk_obfs_sender,
                    #[cfg(target_os = "android")]
                    args.tun_provider,
                )
                .await?;
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

            let metadata = Self::tunnel_metadata(&iface_name, &config);
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

        let close_sender = close_obfs_sender.clone();
        let monitor_handle = tokio::spawn(async move {
            // This is safe to unwrap because the future resolves to `Result<Infallible, E>`.
            let close_msg = tunnel_fut.await.unwrap_err();
            let _ = close_sender.send(close_msg);
        });

        tokio::spawn(async move {
            if args.tunnel_close_rx.await.is_ok() {
                monitor_handle.abort();
                let _ = close_obfs_sender.send(CloseMsg::Stop);
            }
        });

        Ok(monitor)
    }

    #[allow(clippy::too_many_arguments)]
    async fn psk_negotiation<F>(
        tunnel: &Arc<Mutex<Option<Box<dyn Tunnel>>>>,
        config: &mut Config,
        retry_attempt: u32,
        on_event: F,
        iface_name: &str,
        obfuscator: Arc<AsyncMutex<Option<ObfuscatorHandle>>>,
        close_obfs_sender: sync_mpsc::Sender<CloseMsg>,
        #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
    ) -> std::result::Result<(), CloseMsg>
    where
        F: (Fn(TunnelEvent) -> Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + Clone
            + 'static,
    {
        let wg_psk_privkey = PrivateKey::new_from_random();
        let close_obfs_sender = close_obfs_sender.clone();

        let allowed_traffic = Endpoint::new(
            config.ipv4_gateway,
            talpid_tunnel_config_client::CONFIG_SERVICE_PORT,
            TransportProtocol::Tcp,
        );
        let allowed_traffic = if config.is_multihop() {
            // NOTE: We need to let traffic meant for the exit IP through the firewall. This
            // should not allow any non-PQ traffic to leak since you can only reach the
            // exit peer with these rules and not the broader internet.
            AllowedTunnelTraffic::Two(
                allowed_traffic,
                Endpoint::from_socket_address(
                    config.exit_peer_mut().endpoint,
                    TransportProtocol::Udp,
                ),
            )
        } else {
            AllowedTunnelTraffic::One(allowed_traffic)
        };
        let metadata = Self::tunnel_metadata(iface_name, config);
        (on_event)(TunnelEvent::InterfaceUp(metadata, allowed_traffic.clone())).await;

        let exit_psk =
            Self::perform_psk_negotiation(retry_attempt, config, wg_psk_privkey.public_key())
                .await?;

        log::debug!("Successfully exchanged PSK with exit peer");

        if config.is_multihop() {
            // Set up tunnel to lead to entry
            let mut entry_tun_config = config.clone();
            entry_tun_config
                .entry_peer
                .allowed_ips
                .push(IpNetwork::new(IpAddr::V4(config.ipv4_gateway), 32).unwrap());

            let close_obfs_sender = close_obfs_sender.clone();
            let entry_config = Self::reconfigure_tunnel(
                tunnel,
                entry_tun_config,
                obfuscator.clone(),
                close_obfs_sender,
                #[cfg(target_os = "android")]
                &tun_provider,
            )
            .await?;
            let entry_psk = Some(
                Self::perform_psk_negotiation(
                    retry_attempt,
                    &entry_config,
                    wg_psk_privkey.public_key(),
                )
                .await?,
            );
            log::debug!("Successfully exchanged PSK with entry peer");

            config.entry_peer.psk = entry_psk;
        }

        config.exit_peer_mut().psk = Some(exit_psk);

        config.tunnel.private_key = wg_psk_privkey;

        *config = Self::reconfigure_tunnel(
            tunnel,
            config.clone(),
            obfuscator,
            close_obfs_sender,
            #[cfg(target_os = "android")]
            &tun_provider,
        )
        .await?;
        let metadata = Self::tunnel_metadata(iface_name, config);
        (on_event)(TunnelEvent::InterfaceUp(
            metadata,
            AllowedTunnelTraffic::All,
        ))
        .await;

        Ok(())
    }

    /// Reconfigures the tunnel to use the provided config while potentially modifying the config
    /// and restarting the obfuscation provider. Returns the new config used by the new tunnel.
    async fn reconfigure_tunnel(
        tunnel: &Arc<Mutex<Option<Box<dyn Tunnel>>>>,
        mut config: Config,
        obfuscator: Arc<AsyncMutex<Option<ObfuscatorHandle>>>,
        close_obfs_sender: sync_mpsc::Sender<CloseMsg>,
        #[cfg(target_os = "android")] tun_provider: &Arc<Mutex<TunProvider>>,
    ) -> std::result::Result<Config, CloseMsg> {
        let mut obfs_guard = obfuscator.lock().await;
        if let Some(obfuscator_handle) = obfs_guard.take() {
            obfuscator_handle.abort();
            *obfs_guard = maybe_create_obfuscator(&mut config, close_obfs_sender)
                .await
                .map_err(CloseMsg::ObfuscatorFailed)?;

            // Exclude new remote obfuscation socket or bridge
            #[cfg(target_os = "android")]
            if let Some(obfuscator_handle) = &*obfs_guard {
                let remote_socket_fd = obfuscator_handle.remote_socket_fd();
                log::debug!("Excluding remote socket fd from the tunnel");
                if let Err(error) = tun_provider.lock().unwrap().bypass(remote_socket_fd) {
                    log::error!("Failed to exclude remote socket fd: {error}");
                }
            }
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

        Ok(config)
    }

    /// Replace `0.0.0.0/0`/`::/0` with the gateway IPs when `gateway_only` is true.
    /// Used to block traffic to other destinations while connecting on Android.
    #[cfg(target_os = "android")]
    fn patch_allowed_ips(config: &Config, gateway_only: bool) -> Cow<'_, Config> {
        if gateway_only {
            let mut patched_config = config.clone();
            let gateway_net_v4 = ipnetwork::IpNetwork::from(IpAddr::from(config.ipv4_gateway));
            let gateway_net_v6 = config
                .ipv6_gateway
                .map(|net| ipnetwork::IpNetwork::from(IpAddr::from(net)));
            for peer in patched_config.peers_mut() {
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
        let luid = talpid_windows::net::luid_from_alias(iface_name).map_err(|error| {
            log::error!("Failed to obtain tunnel interface LUID: {}", error);
            CloseMsg::SetupError(Error::IpInterfacesError)
        })?;
        for address in addresses {
            talpid_windows::net::add_ip_address_for_interface(luid, *address)
                .map_err(|error| CloseMsg::SetupError(Error::SetIpAddressesError(error)))?;
        }
        Ok(())
    }

    async fn perform_psk_negotiation(
        retry_attempt: u32,
        config: &Config,
        wg_psk_pubkey: PublicKey,
    ) -> std::result::Result<PresharedKey, CloseMsg> {
        log::debug!("Performing PQ-safe PSK exchange");

        let timeout = std::cmp::min(
            MAX_PSK_EXCHANGE_TIMEOUT,
            INITIAL_PSK_EXCHANGE_TIMEOUT
                .saturating_mul(PSK_EXCHANGE_TIMEOUT_MULTIPLIER.saturating_pow(retry_attempt)),
        );

        let psk = tokio::time::timeout(
            timeout,
            talpid_tunnel_config_client::push_pq_key(
                IpAddr::from(config.ipv4_gateway),
                config.tunnel.private_key.public_key(),
                wg_psk_pubkey,
            ),
        )
        .await
        .map_err(|_timeout_err| {
            log::warn!("Timeout while negotiating PSK");
            CloseMsg::PskNegotiationTimeout
        })?
        .map_err(Error::PskNegotiationError)
        .map_err(CloseMsg::SetupError)?;

        Ok(psk)
    }

    #[allow(unused_variables)]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
        tun_provider: Arc<Mutex<TunProvider>>,
        #[cfg(target_os = "android")] psk_negotiation: bool,
        #[cfg(windows)] route_manager_handle: crate::routing::RouteManagerHandle,
        #[cfg(windows)] setup_done_tx: mpsc::Sender<std::result::Result<(), BoxedError>>,
    ) -> Result<Box<dyn Tunnel>> {
        log::debug!("Tunnel MTU: {}", config.mtu);

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
        {
            wireguard_nt::WgNtTunnel::start_tunnel(config, log_path, resource_dir, setup_done_tx)
                .map(|tun| Box::new(tun) as Box<dyn Tunnel + 'static>)
                .map_err(Error::TunnelError)
        }

        #[cfg(wireguard_go)]
        {
            let routes =
                Self::get_tunnel_destinations(config).flat_map(Self::replace_default_prefixes);

            #[cfg(target_os = "android")]
            let config = Self::patch_allowed_ips(config, psk_negotiation);

            #[cfg(target_os = "linux")]
            log::debug!("Using userspace WireGuard implementation");
            Ok(Box::new(
                WgGoTunnel::start_tunnel(
                    #[allow(clippy::needless_borrow)]
                    &config,
                    log_path,
                    tun_provider,
                    routes,
                )
                .map_err(Error::TunnelError)?,
            ))
        }
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
            let v4 = routing::Node::new(config.ipv4_gateway.into(), iface_name.to_string());
            let v6 = if let Some(ipv6_gateway) = config.ipv6_gateway.as_ref() {
                routing::Node::new((*ipv6_gateway).into(), iface_name.to_string())
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

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let gateway_routes =
            gateway_routes.map(|route| Self::apply_route_mtu_for_multihop(route, config));

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
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        return iter;

        #[cfg(target_os = "linux")]
        return iter
            .map(|route| route.use_main_table(false))
            .map(|route| Self::apply_route_mtu_for_multihop(route, config));

        #[cfg(target_os = "macos")]
        iter.map(|route| Self::apply_route_mtu_for_multihop(route, config))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn apply_route_mtu_for_multihop(route: RequiredRoute, config: &Config) -> RequiredRoute {
        if !config.is_multihop() {
            route
        } else {
            // Set route MTU by subtracting the WireGuard overhead from the tunnel MTU. Plus
            // some margin to make room for padding bytes.
            let ip_overhead = match route.prefix.is_ipv4() {
                true => IPV4_HEADER_SIZE,
                false => IPV6_HEADER_SIZE,
            };
            const PADDING_BYTES_MARGIN: u16 = 15;
            let mtu = config.mtu - ip_overhead - WIREGUARD_HEADER_SIZE - PADDING_BYTES_MARGIN;

            route.mtu(mtu)
        }
    }

    /// Return routes for all allowed IPs.
    fn get_tunnel_destinations(config: &Config) -> impl Iterator<Item = ipnetwork::IpNetwork> + '_ {
        config
            .peers()
            .flat_map(|peer| peer.allowed_ips.iter())
            .cloned()
    }

    /// Replace default (0-prefix) routes with more specific routes.
    fn replace_default_prefixes(network: ipnetwork::IpNetwork) -> Vec<ipnetwork::IpNetwork> {
        #[cfg(windows)]
        if network.prefix() == 0 {
            if network.is_ipv4() {
                vec!["0.0.0.0/1".parse().unwrap(), "128.0.0.0/1".parse().unwrap()]
            } else {
                vec!["8000::/1".parse().unwrap(), "::/1".parse().unwrap()]
            }
        } else {
            vec![network]
        }

        #[cfg(not(windows))]
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

/// Detects the maximum MTU that does not cause dropped packets.
///
/// The detection works by sending evenly spread out range of pings between 576 and the given
/// current tunnel MTU, and returning the maximum packet size that was returned within a timeout.
#[cfg(target_os = "linux")]
async fn auto_mtu_detection(
    gateway: std::net::Ipv4Addr,
    #[cfg(any(target_os = "macos", target_os = "linux"))] iface_name: String,
    current_mtu: u16,
) -> Result<u16> {
    use futures::{future, stream::FuturesUnordered, TryStreamExt};
    use surge_ping::{Client, Config, PingIdentifier, PingSequence, SurgeError};
    use talpid_tunnel::{ICMP_HEADER_SIZE, MIN_IPV4_MTU};
    use tokio_stream::StreamExt;

    /// Max time to wait for any ping, when this expires, we give up and throw an error.
    const PING_TIMEOUT: Duration = Duration::from_secs(10);
    /// Max time to wait after the first ping arrives. Every ping after this timeout is considered
    /// dropped, so we return the largest collected packet size.
    const PING_OFFSET_TIMEOUT: Duration = Duration::from_secs(2);

    let config_builder = Config::builder().kind(surge_ping::ICMP::V4);
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let config_builder = config_builder.interface(&iface_name);
    let client = Client::new(&config_builder.build()).unwrap();

    let step_size = 20;
    let linspace = mtu_spacing(MIN_IPV4_MTU, current_mtu, step_size);

    let payload_buf = vec![0; current_mtu as usize];

    let mut ping_stream = linspace
        .iter()
        .enumerate()
        .map(|(i, &mtu)| {
            let client = client.clone();
            let payload_size = (mtu - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE) as usize;
            let payload = &payload_buf[0..payload_size];
            async move {
                log::trace!("Sending ICMP ping of total size {mtu}");
                client
                    .pinger(IpAddr::V4(gateway), PingIdentifier(0))
                    .await
                    .timeout(PING_TIMEOUT)
                    .ping(PingSequence(i as u16), payload)
                    .await
            }
        })
        .collect::<FuturesUnordered<_>>()
        .map_ok(|(packet, _rtt)| {
            let surge_ping::IcmpPacket::V4(packet) = packet else {
                unreachable!("ICMP ping response was not of IPv4 type");
            };
            let size = packet.get_size() as u16 + IPV4_HEADER_SIZE;
            log::trace!("Got ICMP ping response of total size {size}");
            debug_assert_eq!(size, linspace[packet.get_sequence().0 as usize]);
            size
        });

    let first_ping_size = ping_stream
        .next()
        .await
        .expect("At least one pings should be sent")
        // Short-circuit and return on error
        .map_err(|e| match e {
            // If the first ping we get back timed out, then all of them did
            SurgeError::Timeout { .. } => Error::MtuDetectionAllDropped,
            // Unexpected error type
            e => Error::MtuDetectionPingError(e),
        })?;

    ping_stream
        .timeout(PING_OFFSET_TIMEOUT) // Start a new, shorter, timeout
        .map_while(|res| res.ok()) // Stop waiting for pings after this timeout
        .try_fold(first_ping_size, |acc, mtu| future::ready(Ok(acc.max(mtu)))) // Get largest ping
        .await
        .map_err(Error::MtuDetectionPingError)
}

/// Creates a linear spacing of MTU values with the given step size. Always includes the given end
/// points.
#[cfg(target_os = "linux")]
fn mtu_spacing(mtu_min: u16, mtu_max: u16, step_size: u16) -> Vec<u16> {
    if mtu_min > mtu_max {
        panic!("Invalid MTU detection range: `mtu_min`={mtu_min}, `mtu_max`={mtu_max}.");
    }
    let second_mtu = mtu_min.next_multiple_of(step_size);
    let in_between = (second_mtu..mtu_max).step_by(step_size as usize);
    let mut ret = Vec::with_capacity(((mtu_max - second_mtu).div_ceil(step_size) + 2) as usize);
    ret.push(mtu_min);
    ret.extend(in_between);
    ret.push(mtu_max);
    ret
}

#[derive(Debug)]
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
    StatsError(#[error(source)] BoxedError),

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
    SetupTunnelDevice(#[error(source)] tun_provider::Error),

    /// Failed to set up a tunnel device
    #[cfg(windows)]
    #[error(display = "Failed to create tunnel device")]
    SetupTunnelDevice(#[error(source)] io::Error),

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
