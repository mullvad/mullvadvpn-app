//! Manage WireGuard tunnels.

#![deny(missing_docs)]

use self::config::Config;
#[cfg(windows)]
use futures::channel::mpsc;
use futures::future::Future;
use obfuscation::ObfuscatorHandle;
#[cfg(target_os = "android")]
use std::borrow::Cow;
#[cfg(windows)]
use std::io;
use std::{
    convert::Infallible,
    net::IpAddr,
    path::Path,
    pin::Pin,
    sync::{mpsc as sync_mpsc, Arc},
};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::{env, sync::LazyLock};
#[cfg(not(target_os = "android"))]
use talpid_routing::{self, RequiredRoute};
use talpid_tunnel::{tun_provider, EventHook, TunnelArgs, TunnelEvent, TunnelMetadata};

#[cfg(target_os = "android")]
use talpid_routing::RouteManagerHandle;
#[cfg(daita)]
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::{
    net::{wireguard::TunnelParameters, AllowedTunnelTraffic, Endpoint, TransportProtocol},
    BoxedError, ErrorExt,
};
use tokio::sync::Mutex as AsyncMutex;

#[cfg(feature = "boringtun")]
mod boringtun;

#[cfg(not(feature = "boringtun"))]
mod wireguard_go;

/// WireGuard config data-types
pub mod config;
mod connectivity;
mod ephemeral;
mod logging;
mod obfuscation;
mod stats;
#[cfg(target_os = "linux")]
pub(crate) mod wireguard_kernel;
#[cfg(windows)]
mod wireguard_nt;

#[cfg(not(target_os = "android"))]
mod mtu_detection;

type TunnelType = Box<dyn Tunnel>;

type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Wireguard tunnel monitor.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to set up routing.
    #[error("Failed to setup routing")]
    SetupRoutingError(#[source] talpid_routing::Error),

    /// Tunnel timed out
    #[error("Tunnel timed out")]
    TimeoutError,

    /// Invalid WireGuard configuration
    #[error("Invalid WireGuard configuration")]
    WireguardConfigError(#[from] crate::config::Error),

    /// An interaction with a tunnel failed
    #[error("Tunnel failed")]
    TunnelError(#[source] TunnelError),

    /// Failed to run tunnel obfuscation
    #[error("Tunnel obfuscation failed")]
    ObfuscationError(#[source] tunnel_obfuscation::Error),

    /// Failed to set up connectivity monitor
    #[error("Connectivity monitor failed")]
    ConnectivityMonitorError(#[source] connectivity::Error),

    /// Failed while negotiating ephemeral peer
    #[error("Failed while negotiating ephemeral peer")]
    EphemeralPeerNegotiationError(#[source] talpid_tunnel_config_client::Error),

    /// Failed to set up IP interfaces.
    #[cfg(windows)]
    #[error("Failed to set up IP interfaces")]
    IpInterfacesError,

    /// Failed to set IP addresses on WireGuard interface
    #[cfg(target_os = "windows")]
    #[error("Failed to set IP addresses on WireGuard interface")]
    SetIpAddressesError(#[source] talpid_windows::net::Error),
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::ObfuscationError(_) => true,
            Error::EphemeralPeerNegotiationError(_) => true,
            Error::TunnelError(TunnelError::RecoverableStartWireguardError(..)) => true,

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
    tunnel: Arc<AsyncMutex<Option<TunnelType>>>,
    /// Callback to signal tunnel events
    event_hook: EventHook,
    close_msg_receiver: sync_mpsc::Receiver<CloseMsg>,
    pinger_stop_sender: connectivity::CancelToken,
    obfuscator: Arc<AsyncMutex<Option<ObfuscatorHandle>>>,
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
/// Overrides the preference for the kernel module for WireGuard.
static FORCE_USERSPACE_WIREGUARD: LazyLock<bool> = LazyLock::new(|| {
    env::var("TALPID_FORCE_USERSPACE_WIREGUARD")
        .map(|v| v != "0")
        .unwrap_or(false)
});

impl WireguardMonitor {
    /// Starts a WireGuard tunnel with the given config
    #[cfg(not(target_os = "android"))]
    pub fn start(
        params: &TunnelParameters,
        log_path: Option<&Path>,
        args: TunnelArgs<'_>,
    ) -> Result<WireguardMonitor> {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let desired_mtu = args
            .runtime
            .block_on(get_desired_mtu(params, &args.route_manager));
        #[cfg(target_os = "macos")]
        let desired_mtu = get_desired_mtu(params);
        let mut config = crate::config::Config::from_parameters(params, desired_mtu)
            .map_err(Error::WireguardConfigError)?;

        let endpoint_addrs = [params.get_next_hop_endpoint().address.ip()];

        let (close_obfs_sender, close_obfs_listener) = sync_mpsc::channel();
        // Start obfuscation server and patch the WireGuard config to point the endpoint to it.
        let obfuscator = args
            .runtime
            .block_on(obfuscation::apply_obfuscation_config(
                &mut config,
                close_obfs_sender.clone(),
            ))?;
        // Don't adjust MTU if overridden by user
        if params.options.mtu.is_none() {
            if let Some(obfuscator) = obfuscator.as_ref() {
                config.mtu = config.mtu.saturating_sub(obfuscator.packet_overhead());
            }
            config.mtu = clamp_mtu(params, config.mtu);
        }

        let userspace_wireguard =
            *FORCE_USERSPACE_WIREGUARD || config.daita || cfg!(feature = "boringtun");

        #[cfg(target_os = "windows")]
        let (setup_done_tx, setup_done_rx) = mpsc::channel(0);
        let tunnel = Self::open_tunnel(
            args.runtime.clone(),
            &config,
            log_path,
            #[cfg(target_os = "windows")]
            args.resource_dir,
            #[cfg(not(all(target_os = "windows", not(feature = "boringtun"))))]
            args.tun_provider.clone(),
            #[cfg(not(all(windows, feature = "boringtun")))]
            args.route_manager.clone(),
            #[cfg(target_os = "windows")]
            setup_done_tx,
            userspace_wireguard,
        )?;
        let iface_name = tunnel.get_interface_name();

        let obfuscator = Arc::new(AsyncMutex::new(obfuscator));

        let gateway = config.ipv4_gateway;
        let (cancel_token, cancel_receiver) = connectivity::CancelToken::new();
        let mut connectivity_monitor = connectivity::Check::new(
            gateway,
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            iface_name.clone(),
            args.retry_attempt,
            cancel_receiver,
        )
        .map_err(Error::ConnectivityMonitorError)?;

        let monitor = WireguardMonitor {
            runtime: args.runtime.clone(),
            tunnel: Arc::new(AsyncMutex::new(Some(tunnel))),
            event_hook: args.event_hook.clone(),
            close_msg_receiver: close_obfs_listener,
            pinger_stop_sender: cancel_token,
            obfuscator,
        };

        let mut event_hook = args.event_hook.clone();
        let moved_tunnel = monitor.tunnel.clone();
        let moved_close_obfs_sender = close_obfs_sender.clone();
        let moved_obfuscator = monitor.obfuscator.clone();
        let detect_mtu = params.options.mtu.is_none();
        let tunnel_fut = async move {
            let tunnel = moved_tunnel;
            let close_obfs_sender: sync_mpsc::Sender<CloseMsg> = moved_close_obfs_sender;
            let obfuscator = moved_obfuscator;
            // TODO: Make the tun crate either not use 'netsh' or make it possible to disable
            // configuration of address and DNS
            #[cfg(windows)]
            if !(cfg!(feature = "boringtun") && userspace_wireguard) {
                Self::add_device_ip_addresses(&iface_name, &config.tunnel.addresses, setup_done_rx)
                    .await?;
            }

            let metadata = Self::tunnel_metadata(&iface_name, &config);
            let allowed_traffic = Self::allowed_traffic_during_tunnel_config(&config);
            event_hook
                .on_event(TunnelEvent::InterfaceUp(metadata.clone(), allowed_traffic))
                .await;

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

            let ephemeral_obfs_sender = close_obfs_sender.clone();
            if config.quantum_resistant || config.daita {
                if let Err(e) = ephemeral::config_ephemeral_peers(
                    &tunnel,
                    &mut config,
                    args.retry_attempt,
                    obfuscator.clone(),
                    ephemeral_obfs_sender,
                )
                .await
                {
                    // We have received a small amount of reports about ephemeral peer nogationation
                    // timing out on Windows for 2024.9-beta1. These verbose data usage logs are
                    // a temporary measure to help us understand the issue. They can be removed
                    // if the issue is resolved.
                    log_tunnel_data_usage(&config, &tunnel).await;
                    return Err(e);
                }

                let metadata = Self::tunnel_metadata(&iface_name, &config);
                event_hook
                    .on_event(TunnelEvent::InterfaceUp(
                        metadata,
                        Self::allowed_traffic_after_tunnel_config(),
                    ))
                    .await;
            }

            if detect_mtu {
                let config = config.clone();
                let iface_name = iface_name.clone();
                tokio::task::spawn(async move {
                    if config.daita {
                        // TODO: For now, we assume the MTU during the tunnel lifetime.
                        // We could instead poke maybenot whenever we detect changes to it.
                        log::warn!("MTU detection is not supported with DAITA. Skipping");
                        return;
                    }

                    if let Err(e) = mtu_detection::automatic_mtu_correction(
                        gateway,
                        iface_name,
                        config.mtu,
                        #[cfg(windows)]
                        config.ipv6_gateway.is_some(),
                    )
                    .await
                    {
                        log::error!(
                            "{}",
                            e.display_chain_with_msg(
                                "Failed to automatically adjust MTU based on dropped packets"
                            )
                        );
                    };
                });
            }

            let lock = tunnel.lock().await;
            let borrowed_tun = lock.as_ref().expect("The tunnel was dropped unexpectedly");
            match connectivity_monitor
                .establish_connectivity(borrowed_tun.as_ref())
                .await
            {
                Ok(true) => Ok(()),
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
            }?;
            drop(lock);

            // Add any default route(s) that may exist.
            args.route_manager
                .add_routes(Self::get_post_tunnel_routes(&iface_name, &config).collect())
                .await
                .map_err(Error::SetupRoutingError)
                .map_err(CloseMsg::SetupError)?;

            let metadata = Self::tunnel_metadata(&iface_name, &config);
            event_hook.on_event(TunnelEvent::Up(metadata)).await;

            if let Err(error) = connectivity::Monitor::init(connectivity_monitor)
                .run(Arc::downgrade(&tunnel))
                .await
            {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Connectivity monitor failed")
                );
            }

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

    /// Starts a WireGuard tunnel with the given config
    ///
    /// This differs from [`start`] on other platforms in multiple ways. Here is a list of some
    /// notable differences:
    /// - A ping is sent between the Wireguard-GO tunnel is started and an ephemeral peer is
    ///   negotiated. There seems to be a race condition between starting the tunnel and the tunnel
    ///   being ready to serve traffic.
    /// - No routes are configured on android.
    #[cfg(target_os = "android")]
    pub fn start(
        params: &TunnelParameters,
        log_path: Option<&Path>,
        args: TunnelArgs<'_>,
    ) -> Result<WireguardMonitor> {
        let desired_mtu = get_desired_mtu(params);
        let mut config =
            Config::from_parameters(params, desired_mtu).map_err(Error::WireguardConfigError)?;
        let (close_obfs_sender, close_obfs_listener) = sync_mpsc::channel();
        // Start obfuscation server and patch the WireGuard config to point the endpoint to it.
        let obfuscator = args
            .runtime
            .block_on(obfuscation::apply_obfuscation_config(
                &mut config,
                close_obfs_sender.clone(),
                args.tun_provider.clone(),
            ))?;
        // Don't adjust MTU if overridden by user
        if params.options.mtu.is_none() {
            if let Some(obfuscator) = obfuscator.as_ref() {
                config.mtu = config.mtu.saturating_sub(obfuscator.packet_overhead());
            }
            config.mtu = clamp_mtu(params, config.mtu);
        }

        let should_negotiate_ephemeral_peer = config.quantum_resistant || config.daita;

        let (cancel_token, cancel_receiver) = connectivity::CancelToken::new();
        let mut connectivity_monitor = connectivity::Check::new(
            config.ipv4_gateway,
            args.retry_attempt,
            cancel_receiver.clone(),
        )
        .map_err(Error::ConnectivityMonitorError)?;

        #[cfg(feature = "boringtun")]
        let tunnel = args
            .runtime
            .block_on(Self::open_boringtun_tunnel(
                &config,
                log_path,
                args.tun_provider.clone(),
                args.route_manager,
            ))
            .map(Box::new)? as Box<dyn Tunnel>;

        #[cfg(not(feature = "boringtun"))]
        let tunnel = args
            .runtime
            .block_on(Self::open_wireguard_go_tunnel(
                &config,
                log_path,
                args.tun_provider.clone(),
                args.route_manager,
                // In case we should negotiate an ephemeral peer, we should specify via AllowedIPs
                // that we only allows traffic to/from the gateway. This is only needed on Android
                // since we lack a firewall there.
                should_negotiate_ephemeral_peer,
                cancel_receiver,
            ))
            .map(Box::new)? as Box<dyn Tunnel>;

        let iface_name = tunnel.get_interface_name();
        let tunnel = Arc::new(AsyncMutex::new(Some(tunnel)));
        let mut event_hook = args.event_hook;
        let monitor = WireguardMonitor {
            runtime: args.runtime.clone(),
            tunnel: Arc::clone(&tunnel),
            event_hook: event_hook.clone(),
            close_msg_receiver: close_obfs_listener,
            pinger_stop_sender: cancel_token,
            obfuscator: Arc::new(AsyncMutex::new(obfuscator)),
        };

        let moved_close_obfs_sender = close_obfs_sender.clone();
        let moved_obfuscator = monitor.obfuscator.clone();
        let tunnel_fut = async move {
            let close_obfs_sender: sync_mpsc::Sender<CloseMsg> = moved_close_obfs_sender;
            let obfuscator = moved_obfuscator;

            let metadata = Self::tunnel_metadata(&iface_name, &config);
            let allowed_traffic = Self::allowed_traffic_during_tunnel_config(&config);
            event_hook
                .on_event(TunnelEvent::InterfaceUp(metadata.clone(), allowed_traffic))
                .await;

            let lock = tunnel.lock().await;
            let borrowed_tun = lock.as_ref().expect("The tunnel was dropped unexpectedly");
            match connectivity_monitor
                .establish_connectivity(borrowed_tun.as_ref())
                .await
            {
                Ok(true) => Ok(()),
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
            }?;
            drop(lock);

            if should_negotiate_ephemeral_peer {
                let ephemeral_obfs_sender = close_obfs_sender.clone();

                if let Err(e) = ephemeral::config_ephemeral_peers(
                    &tunnel,
                    &mut config,
                    args.retry_attempt,
                    obfuscator.clone(),
                    ephemeral_obfs_sender,
                    args.tun_provider,
                )
                .await
                {
                    // We have received a small amount of reports about ephemeral peer nogationation
                    // timing out on Windows for 2024.9-beta1. These verbose data usage logs are
                    // a temporary measure to help us understand the issue. They can be removed
                    // if the issue is resolved.
                    log_tunnel_data_usage(&config, &tunnel).await;
                    return Err(e);
                }

                let metadata = Self::tunnel_metadata(&iface_name, &config);
                event_hook
                    .on_event(TunnelEvent::InterfaceUp(
                        metadata,
                        Self::allowed_traffic_after_tunnel_config(),
                    ))
                    .await;
            }

            let metadata = Self::tunnel_metadata(&iface_name, &config);
            event_hook.on_event(TunnelEvent::Up(metadata)).await;

            if let Err(error) = connectivity::Monitor::init(connectivity_monitor)
                .run(Arc::downgrade(&tunnel))
                .await
            {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Connectivity monitor failed")
                );
            }

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

    fn allowed_traffic_during_tunnel_config(config: &Config) -> AllowedTunnelTraffic {
        // During ephemeral peer negotiation, only allow traffic to the config service.
        if config.quantum_resistant || config.daita {
            let config_endpoint = Endpoint::new(
                config.ipv4_gateway,
                talpid_tunnel_config_client::CONFIG_SERVICE_PORT,
                TransportProtocol::Tcp,
            );
            if config.is_multihop() {
                // If multihop is enabled, allow traffic to the exit peer as well.
                AllowedTunnelTraffic::Two(
                    config_endpoint,
                    Endpoint::from_socket_address(
                        config.exit_peer().endpoint,
                        TransportProtocol::Udp,
                    ),
                )
            } else {
                AllowedTunnelTraffic::One(config_endpoint)
            }
        } else {
            AllowedTunnelTraffic::All
        }
    }

    fn allowed_traffic_after_tunnel_config() -> AllowedTunnelTraffic {
        // After ephemeral peer negotiation, allow all tunnel traffic again.
        AllowedTunnelTraffic::All
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
        use futures::StreamExt;

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

    #[allow(clippy::too_many_arguments)]
    #[cfg(target_os = "windows")]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
        #[cfg(feature = "boringtun")] tun_provider: Arc<
            std::sync::Mutex<tun_provider::TunProvider>,
        >,
        #[cfg(not(feature = "boringtun"))] route_manager: talpid_routing::RouteManagerHandle,
        setup_done_tx: mpsc::Sender<std::result::Result<(), BoxedError>>,
        userspace_wireguard: bool,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        if userspace_wireguard {
            log::debug!("Using userspace WireGuard implementation");

            #[cfg(feature = "boringtun")]
            let tunnel = runtime
                .block_on(Self::open_boringtun_tunnel(config, log_path, tun_provider))
                .map(Box::new)?;

            #[cfg(not(feature = "boringtun"))]
            let tunnel = runtime
                .block_on(Self::open_wireguard_go_tunnel(
                    config,
                    log_path,
                    setup_done_tx,
                    route_manager,
                ))
                .map(Box::new)?;
            Ok(tunnel)
        } else {
            log::debug!("Using kernel WireGuard implementation");

            wireguard_nt::WgNtTunnel::start_tunnel(config, log_path, resource_dir, setup_done_tx)
                .map(|tun| Box::new(tun) as Box<dyn Tunnel + 'static>)
                .map_err(Error::TunnelError)
        }
    }

    #[cfg(target_os = "macos")]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        _userspace_wireguard: bool,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        log::debug!("Using userspace WireGuard implementation");

        let tunnel = runtime
            .block_on(Self::open_wireguard_go_tunnel(
                config,
                log_path,
                tun_provider,
            ))
            .map(Box::new)?;
        Ok(tunnel)
    }

    #[cfg(target_os = "linux")]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        userspace_wireguard: bool,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        if userspace_wireguard {
            log::debug!("Using userspace WireGuard implementation");

            #[cfg(not(feature = "boringtun"))]
            let f = Self::open_wireguard_go_tunnel(config, log_path, tun_provider);

            #[cfg(feature = "boringtun")]
            let f = Self::open_boringtun_tunnel(config, log_path, tun_provider);

            let tunnel = runtime.block_on(f).map(Box::new)?;
            Ok(tunnel)
        } else {
            let res = if will_nm_manage_dns() {
                log::debug!("Using kernel WireGuard implementation through NetworkManager");
                wireguard_kernel::NetworkManagerTunnel::new(runtime.clone(), config)
                    .map(|tunnel| Box::new(tunnel) as TunnelType)
            } else {
                log::debug!("Using kernel WireGuard implementation through netlink");
                wireguard_kernel::NetlinkTunnel::new(runtime.clone(), config)
                    .map(|tunnel| Box::new(tunnel) as TunnelType)
            };

            res.or_else(|err| {
                    log::warn!("Failed to initialize kernel WireGuard tunnel, falling back to userspace WireGuard implementation:\n{}",err.display_chain() );
                    Ok(runtime
                        .block_on(Self::open_wireguard_go_tunnel(
                            config,
                            log_path,
                            tun_provider,
                        ))
                        .map(Box::new)?)
                })
        }
    }

    /// Configure and start a Wireguard-go tunnel.
    #[cfg(not(feature = "boringtun"))]
    #[allow(clippy::unused_async)]
    async fn open_wireguard_go_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        #[cfg(unix)] tun_provider: Arc<std::sync::Mutex<tun_provider::TunProvider>>,
        #[cfg(target_os = "android")] route_manager: RouteManagerHandle,
        #[cfg(windows)] setup_done_tx: mpsc::Sender<std::result::Result<(), BoxedError>>,
        #[cfg(windows)] route_manager: talpid_routing::RouteManagerHandle,
        #[cfg(target_os = "android")] gateway_only: bool,
        #[cfg(target_os = "android")] cancel_receiver: connectivity::CancelReceiver,
    ) -> Result<wireguard_go::WgGoTunnel> {
        #[cfg(all(unix, not(target_os = "android")))]
        let routes = config.get_tunnel_destinations();

        #[cfg(all(unix, not(target_os = "android")))]
        let tunnel = wireguard_go::WgGoTunnel::start_tunnel(config, log_path, tun_provider, routes)
            .map_err(Error::TunnelError)?;

        #[cfg(target_os = "windows")]
        let tunnel =
            wireguard_go::WgGoTunnel::start_tunnel(config, log_path, route_manager, setup_done_tx)
                .await
                .map_err(Error::TunnelError)?;

        // Android uses multihop implemented in Mullvad's wireguard-go fork. When negotiating
        // with an ephemeral peer, this multihop strategy require us to restart the tunnel
        // every time we want to reconfigure it. As such, we will actually start a multihop
        // tunnel at a later stage, after we have negotiated with the first ephemeral peer.
        // At this point, when the tunnel *is first started*, we establish a regular, singlehop
        // tunnel to where the ephemeral peer resides.
        //
        // Refer to `docs/architecture.md` for details on how to use multihop + PQ.
        #[cfg(target_os = "android")]
        let config = Self::patch_allowed_ips(config, gateway_only);

        #[cfg(target_os = "android")]
        let tunnel = if let Some(exit_peer) = &config.exit_peer {
            wireguard_go::WgGoTunnel::start_multihop_tunnel(
                &config,
                exit_peer,
                log_path,
                tun_provider,
                route_manager,
                cancel_receiver,
            )
            .await
            .map_err(Error::TunnelError)?
        } else {
            wireguard_go::WgGoTunnel::start_tunnel(
                #[allow(clippy::needless_borrow)]
                &config,
                log_path,
                tun_provider,
                route_manager,
                cancel_receiver,
            )
            .await
            .map_err(Error::TunnelError)?
        };

        Ok(tunnel)
    }

    /// Configure and start a boringtun tunnel.
    #[cfg(feature = "boringtun")]
    async fn open_boringtun_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<std::sync::Mutex<tun_provider::TunProvider>>,
        #[cfg(target_os = "android")] route_manager_handle: RouteManagerHandle,
        //#[cfg(target_os = "android")] gateway_only: bool,
        //#[cfg(target_os = "android")] connectivity_check: connectivity::Check<
        //    connectivity::Cancellable,
        //>,
    ) -> Result<boringtun::BoringTun> {
        let routes = config
            .get_tunnel_destinations()
            .flat_map(Self::replace_default_prefixes);

        let tunnel = boringtun::BoringTun::start_tunnel(
            config,
            log_path,
            tun_provider,
            routes,
            #[cfg(target_os = "android")]
            route_manager_handle,
        )
        .await
        .map_err(Error::TunnelError)?;

        Ok(tunnel)
    }

    /// Blocks the current thread until tunnel disconnects
    pub fn wait(mut self) -> Result<()> {
        let wait_result = match self.close_msg_receiver.recv() {
            Ok(CloseMsg::EphemeralPeerNegotiationTimeout) | Ok(CloseMsg::PingErr) => {
                Err(Error::TimeoutError)
            }
            Ok(CloseMsg::Stop) | Ok(CloseMsg::ObfuscatorExpired) => Ok(()),
            Ok(CloseMsg::SetupError(error)) => Err(error),
            Ok(CloseMsg::ObfuscatorFailed(error)) => Err(error),
            Err(_) => Ok(()),
        };

        self.pinger_stop_sender.close();

        self.runtime
            .block_on(self.event_hook.on_event(TunnelEvent::Down));

        self.stop_tunnel();

        wait_result
    }

    /// Tear down the tunnel.
    ///
    /// NOTE: will panic if called from within a tokio runtime.
    fn stop_tunnel(&mut self) {
        match self.tunnel.blocking_lock().take() {
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
    #[cfg(not(target_os = "android"))]
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
                talpid_routing::NetNode::DefaultNode,
            )
        })
    }

    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    #[cfg(not(target_os = "android"))]
    fn get_tunnel_nodes(
        iface_name: &str,
        config: &Config,
    ) -> (talpid_routing::Node, talpid_routing::Node) {
        #[cfg(windows)]
        {
            let v4 = talpid_routing::Node::new(config.ipv4_gateway.into(), iface_name.to_string());
            let v6 = if let Some(ipv6_gateway) = config.ipv6_gateway.as_ref() {
                talpid_routing::Node::new((*ipv6_gateway).into(), iface_name.to_string())
            } else {
                talpid_routing::Node::device(iface_name.to_string())
            };
            (v4, v6)
        }

        #[cfg(not(windows))]
        {
            let node = talpid_routing::Node::device(iface_name.to_string());
            (node.clone(), node)
        }
    }

    /// Return routes for all allowed IPs, as well as the gateway, except 0.0.0.0/0.
    #[cfg(not(target_os = "android"))]
    fn get_pre_tunnel_routes<'a>(
        iface_name: &str,
        config: &'a Config,
    ) -> impl Iterator<Item = RequiredRoute> + 'a {
        let gateway_node = talpid_routing::Node::device(iface_name.to_string());
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
            config
                .get_tunnel_destinations()
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
    #[cfg(not(target_os = "android"))]
    fn get_post_tunnel_routes<'a>(
        iface_name: &str,
        config: &'a Config,
    ) -> impl Iterator<Item = RequiredRoute> + 'a {
        let (node_v4, node_v6) = Self::get_tunnel_nodes(iface_name, config);
        let iter = config
            .get_tunnel_destinations()
            .filter(|allowed_ip| allowed_ip.prefix() == 0)
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
        use talpid_tunnel::{IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, WIREGUARD_HEADER_SIZE};

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

    /// Replace default (0-prefix) routes with more specific routes.
    #[cfg(feature = "boringtun")]
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

/// Log the tunnel stats from the current tunnel.
///
/// This will log the amount of outgoing and incoming data to and from the exit (and entry) relay
/// so far.
async fn log_tunnel_data_usage(config: &Config, tunnel: &Arc<AsyncMutex<Option<TunnelType>>>) {
    let tunnel = tunnel.lock().await;
    let Some(tunnel) = &*tunnel else { return };
    let Ok(tunnel_stats) = tunnel.get_tunnel_stats().await else {
        return;
    };
    if let Some(stats) = config
        .exit_peer
        .as_ref()
        .map(|peer| peer.public_key.as_bytes())
        .and_then(|pubkey| tunnel_stats.get(pubkey))
    {
        log::warn!("Exit peer stats: {:?}", stats);
    };
    let pubkey = config.entry_peer.public_key.as_bytes();
    if let Some(stats) = tunnel_stats.get(pubkey) {
        log::warn!("Entry peer stats: {:?}", stats);
    }
}

#[derive(Debug)]
enum CloseMsg {
    Stop,
    EphemeralPeerNegotiationTimeout,
    PingErr,
    SetupError(Error),
    ObfuscatorExpired,
    ObfuscatorFailed(Error),
}

#[allow(unused)]
#[async_trait::async_trait]
pub(crate) trait Tunnel: Send + Sync {
    fn get_interface_name(&self) -> String;
    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError>;
    async fn get_tunnel_stats(&self) -> std::result::Result<stats::StatsMap, TunnelError>;
    fn set_config<'a>(
        &'a mut self,
        _config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send + 'a>>;
    #[cfg(daita)]
    /// A [`Tunnel`] capable of using DAITA.
    fn start_daita(&mut self, settings: DaitaSettings) -> std::result::Result<(), TunnelError>;
}

/// Errors to be returned from WireGuard implementations, namely implementers of the Tunnel trait
#[derive(thiserror::Error, Debug)]
pub enum TunnelError {
    /// A recoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by the implementation that indicates that trying to establish the
    /// tunnel again should work normally. The error encountered is known to be sporadic.
    #[error("Recoverable error while starting wireguard tunnel")]
    RecoverableStartWireguardError(#[source] Box<dyn std::error::Error + Send>),

    /// An unrecoverable error occurred while starting the wireguard tunnel
    ///
    /// This is an error returned by the implementation that indicates that trying to establish the
    /// tunnel again will likely fail with the same error. An error was encountered during tunnel
    /// configuration which can't be dealt with gracefully.
    #[error("Failed to start wireguard tunnel")]
    FatalStartWireguardError(#[source] Box<dyn std::error::Error + Send>),

    /// Failed to tear down wireguard tunnel.
    #[error("Failed to tear down wireguard tunnel")]
    StopWireguardError(#[source] Box<dyn std::error::Error + Send>),

    /// Error whilst trying to parse the WireGuard config to read the stats
    #[error("Reading tunnel stats failed")]
    StatsError(#[source] BoxedError),

    /// Error whilst trying to retrieve config of a WireGuard tunnel
    #[error("Failed to get config of WireGuard tunnel")]
    GetConfigError,

    /// Failed to set WireGuard tunnel config on device
    #[error("Failed to set config of WireGuard tunnel")]
    SetConfigError,

    /// Failed to duplicate tunnel file descriptor for wireguard-go
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
    #[error("Failed to duplicate tunnel file descriptor for wireguard-go")]
    FdDuplicationError(#[source] nix::Error),

    /// Failed to setup a tunnel device.
    #[cfg(not(windows))]
    #[error("Failed to create tunnel device")]
    SetupTunnelDevice(#[source] tun_provider::Error),

    /// Failed to set up a tunnel device
    #[cfg(windows)]
    #[error("Failed to create tunnel device")]
    SetupTunnelDevice(#[source] io::Error),

    /// Failed to set up a tunnel device
    #[cfg(windows)]
    #[error("Failed to create tunnel device")]
    SetupTunnelDevice2(#[source] tun_provider::Error),

    /// Failed to setup a tunnel device.
    #[cfg(windows)]
    #[error("Failed to config IP interfaces on tunnel device")]
    SetupIpInterfaces(#[source] io::Error),

    /// Failed to configure Wireguard sockets to bypass the tunnel.
    #[cfg(target_os = "android")]
    #[error("Failed to configure Wireguard sockets to bypass the tunnel")]
    BypassError(#[source] tun_provider::Error),

    /// Invalid tunnel interface name.
    #[error("Invalid tunnel interface name")]
    InterfaceNameError(#[source] std::ffi::NulError),

    /// Failed to convert adapter alias to UTF-8.
    #[cfg(target_os = "windows")]
    #[error("Failed to convert adapter alias")]
    InvalidAlias,

    /// Failure to set up logging
    #[error("Failed to set up logging")]
    LoggingError(#[source] logging::Error),

    /// Failed to receive DAITA event
    #[cfg(daita)]
    #[error("Failed to start DAITA")]
    StartDaita(#[source] Box<dyn std::error::Error + Send>),

    /// This tunnel does not support DAITA.
    #[cfg(daita)]
    #[error("Failed to start DAITA - tunnel implemenation does not support DAITA")]
    DaitaNotSupported,

    /// BoringTun device error
    #[cfg(feature = "boringtun")]
    #[error("Boringtun: {0:?}")]
    BoringTunDevice(::boringtun::device::Error),
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
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

// Set the MTU to the lowest possible whilst still allowing for IPv6 to help with wireless
// carriers that do a lot of encapsulation.
const DEFAULT_MTU: u16 = if cfg!(target_os = "android") {
    1280
} else {
    1380
};

#[cfg(any(target_os = "linux", target_os = "windows"))]
async fn get_desired_mtu(
    params: &TunnelParameters,
    route_manager: &talpid_routing::RouteManagerHandle,
) -> u16 {
    match params.options.mtu {
        Some(mtu) => mtu,
        None => {
            // Detect the MTU of the device
            route_manager
                .get_mtu_for_route(params.connection.peer.endpoint.ip())
                .await
                .unwrap_or(DEFAULT_MTU)
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "android"))]
fn get_desired_mtu(params: &TunnelParameters) -> u16 {
    params.options.mtu.unwrap_or(DEFAULT_MTU)
}

/// Calculates and appropriate tunnel MTU based on the given peer MTU minus header sizes
fn clamp_mtu(params: &TunnelParameters, peer_mtu: u16) -> u16 {
    use talpid_tunnel::{
        IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, MIN_IPV4_MTU, MIN_IPV6_MTU, WIREGUARD_HEADER_SIZE,
    };
    // Some users experience fragmentation issues even when we take the interface MTU and
    // subtract the header sizes. This is likely due to some program that they use which does
    // not change the interface MTU but adds its own header onto the outgoing packets. For this
    // reason we subtract some extra bytes from our MTU in order to give other programs some
    // safety margin.
    const MTU_SAFETY_MARGIN: u16 = 60;

    let total_header_size = WIREGUARD_HEADER_SIZE
        + match params.connection.peer.endpoint.is_ipv6() {
            false => IPV4_HEADER_SIZE,
            true => IPV6_HEADER_SIZE,
        };

    // The largest peer MTU that we allow
    let max_peer_mtu: u16 = 1500 - MTU_SAFETY_MARGIN - total_header_size;

    let min_mtu = match params.generic_options.enable_ipv6 {
        false => MIN_IPV4_MTU,
        true => MIN_IPV6_MTU,
    };

    peer_mtu
        .saturating_sub(total_header_size)
        .clamp(min_mtu, max_peer_mtu)
}
