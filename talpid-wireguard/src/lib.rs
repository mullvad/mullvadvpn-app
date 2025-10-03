//! Manage WireGuard tunnels.

#![deny(missing_docs)]

use self::config::Config;
#[cfg(windows)]
use futures::channel::mpsc;
use futures::future::Future;
use obfuscation::ObfuscatorHandle;
#[cfg(windows)]
use std::io;
#[cfg(not(target_os = "android"))]
use std::net::IpAddr;
use std::{
    convert::Infallible,
    path::Path,
    pin::Pin,
    sync::{Arc, mpsc as sync_mpsc},
};
#[cfg(not(target_os = "android"))]
use std::{env, sync::LazyLock};
#[cfg(not(target_os = "android"))]
use talpid_routing::{self, RequiredRoute};
use talpid_tunnel::{EventHook, TunnelArgs, TunnelEvent, TunnelMetadata, tun_provider};
use talpid_tunnel::{IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, WIREGUARD_HEADER_SIZE};

#[cfg(daita)]
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::{
    BoxedError, ErrorExt,
    net::{
        AllowedTunnelTraffic, Endpoint, TransportProtocol,
        wireguard::{PeerConfig, TunnelParameters},
    },
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
    TunnelError(#[from] TunnelError),

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
            Error::TunnelError(TunnelError::SetupTunnelDevice(_)) => true,

            _ => false,
        }
    }

    /// Get the inner tunnel device error, if there is one
    #[cfg(windows)]
    pub fn get_tunnel_device_error(&self) -> Option<&io::Error> {
        match self {
            Error::TunnelError(TunnelError::SetupTunnelDevice(tun_provider::Error::Io(error))) => {
                Some(error)
            }
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

#[cfg(not(target_os = "android"))]
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
        args: TunnelArgs<'_>,
        _log_path: Option<&Path>,
    ) -> Result<WireguardMonitor> {
        let route_mtu = args
            .runtime
            .block_on(get_route_mtu(params, &args.route_manager));

        let userspace_multihop = true; // TODO

        let tunnel_mtu = params.options.mtu.unwrap_or_else(|| {
            let mut overhead = wireguard_overhead(&params.connection.peer);
            if let Some(exit_peer) = &params.connection.exit_peer
                && userspace_multihop
            {
                overhead += wireguard_overhead(exit_peer);
            }
            clamp_tunnel_mtu(params, route_mtu.saturating_sub(dbg!(overhead)))
        });

        let mut config = crate::config::Config::from_parameters(params, tunnel_mtu)
            .map_err(Error::WireguardConfigError)?;

        let endpoint_addrs: Vec<IpAddr> = params
            .get_next_hop_endpoints()
            .iter()
            .map(|ep| ep.address.ip())
            .collect();

        let (close_obfs_sender, close_obfs_listener) = sync_mpsc::channel();
        // Start obfuscation server and patch the WireGuard config to point the endpoint to it.
        let obfuscation_mtu = route_mtu;
        let obfuscator = args
            .runtime
            .block_on(obfuscation::apply_obfuscation_config(
                &mut config,
                obfuscation_mtu,
                close_obfs_sender.clone(),
            ))?;
        // Adjust tunnel MTU again for obfuscation packet overhead
        if params.options.mtu.is_none()
            && let Some(obfuscator) = obfuscator.as_ref()
        {
            config.mtu = clamp_tunnel_mtu(
                params,
                config.mtu.saturating_sub(obfuscator.packet_overhead()),
            );
        }

        // NOTE: We force userspace WireGuard while boringtun is enabled to more easily test it
        // TODO: Consider removing `cfg!(feature = "boringtun")`
        let userspace_wireguard =
            *FORCE_USERSPACE_WIREGUARD || config.daita || cfg!(feature = "boringtun");

        #[cfg(target_os = "windows")]
        let (setup_done_tx, setup_done_rx) = mpsc::channel(0);
        let tunnel = Self::open_tunnel(
            args.runtime.clone(),
            &config,
            #[cfg(target_os = "windows")]
            args.resource_dir,
            #[cfg(not(all(target_os = "windows", not(feature = "boringtun"))))]
            args.tun_provider.clone(),
            #[cfg(all(windows, not(feature = "boringtun")))]
            args.route_manager.clone(),
            #[cfg(target_os = "windows")]
            setup_done_tx,
            userspace_wireguard,
            _log_path,
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
            #[cfg(windows)]
            if cfg!(feature = "boringtun") && userspace_wireguard {
                // NOTE: For boringtun, we use the `tun` crate to create our tunnel interface.
                // It will automatically configure the IP address and DNS servers using `netsh`.
                // This is quite slow, so we need to wait for the interface to be created.
                Self::wait_for_ip_addresses(&config, &iface_name).await?;
            } else {
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

            let routes = Self::get_pre_tunnel_routes(&iface_name, &config, userspace_wireguard)
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
                    obfuscation_mtu,
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
                .add_routes(
                    Self::get_post_tunnel_routes(&iface_name, &config, userspace_wireguard)
                        .collect(),
                )
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
        args: TunnelArgs<'_>,
        #[allow(unused_variables)] log_path: Option<&Path>,
    ) -> Result<WireguardMonitor> {
        let route_mtu = args
            .runtime
            .block_on(get_route_mtu(params, &args.route_manager));

        let tunnel_mtu = params.options.mtu.unwrap_or_else(|| {
            clamp_tunnel_mtu(params, route_mtu.saturating_sub(wireguard_overhead(params)))
        });

        let mut config = crate::config::Config::from_parameters(params, tunnel_mtu)
            .map_err(Error::WireguardConfigError)?;

        // Start obfuscation server and patch the WireGuard config to point the endpoint to it.
        let (close_obfs_sender, close_obfs_listener) = sync_mpsc::channel();
        let obfuscation_mtu = route_mtu;
        let obfuscator = args
            .runtime
            .block_on(obfuscation::apply_obfuscation_config(
                &mut config,
                obfuscation_mtu,
                close_obfs_sender.clone(),
                args.tun_provider.clone(),
            ))?;
        // Adjust MTU again for obfuscation packet overhead
        if params.options.mtu.is_none()
            && let Some(obfuscator) = obfuscator.as_ref()
        {
            config.mtu = clamp_tunnel_mtu(
                params,
                config.mtu.saturating_sub(obfuscator.packet_overhead()),
            );
        }

        let should_negotiate_ephemeral_peer = config.quantum_resistant || config.daita;

        let (cancel_token, cancel_receiver) = connectivity::CancelToken::new();
        #[allow(unused_mut)]
        let mut connectivity_monitor = connectivity::Check::new(
            config.ipv4_gateway,
            args.retry_attempt,
            cancel_receiver.clone(),
        )
        .map_err(Error::ConnectivityMonitorError)?;

        #[cfg(feature = "boringtun")]
        let tunnel = args
            .runtime
            .block_on(boringtun::open_boringtun_tunnel(
                &config,
                args.tun_provider.clone(),
                args.route_manager,
                should_negotiate_ephemeral_peer,
            ))
            .map(Box::new)? as Box<dyn Tunnel>;

        #[cfg(not(feature = "boringtun"))]
        let tunnel = args
            .runtime
            .block_on(wireguard_go::open_wireguard_go_tunnel(
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

            #[cfg(feature = "boringtun")]
            {
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
            }

            if should_negotiate_ephemeral_peer {
                let ephemeral_obfs_sender = close_obfs_sender.clone();

                if let Err(e) = ephemeral::config_ephemeral_peers(
                    &tunnel,
                    &mut config,
                    args.retry_attempt,
                    obfuscation_mtu,
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

    #[cfg(windows)]
    async fn wait_for_ip_addresses(
        config: &Config,
        iface_name: &String,
    ) -> std::result::Result<(), CloseMsg> {
        log::debug!("Waiting for tunnel IP interfaces to arrive");
        let luid = talpid_windows::net::luid_from_alias(iface_name).map_err(|error| {
            log::error!("Failed to obtain tunnel interface LUID: {}", error);
            CloseMsg::SetupError(Error::IpInterfacesError)
        })?;
        talpid_windows::net::wait_for_interfaces(luid, true, config.ipv6_gateway.is_some())
            .await
            .map_err(|error| {
                log::error!("Failed to obtain tunnel interface LUID: {}", error);
                CloseMsg::SetupError(Error::IpInterfacesError)
            })?;
        talpid_windows::net::wait_for_addresses(luid)
            .await
            .map_err(|error| {
                log::error!("Failed to obtain tunnel interface LUID: {}", error);
                CloseMsg::SetupError(Error::IpInterfacesError)
            })?;
        log::debug!("Done waiting for tunnel IP interfaces to arrive");
        Ok(())
    }

    #[cfg(windows)]
    async fn add_device_ip_addresses(
        iface_name: &str,
        addresses: &[std::net::IpAddr],
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
        resource_dir: &Path,
        #[cfg(feature = "boringtun")] tun_provider: Arc<
            std::sync::Mutex<tun_provider::TunProvider>,
        >,
        #[cfg(not(feature = "boringtun"))] route_manager: talpid_routing::RouteManagerHandle,
        setup_done_tx: mpsc::Sender<std::result::Result<(), BoxedError>>,
        userspace_wireguard: bool,
        _log_path: Option<&Path>,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        if userspace_wireguard {
            log::debug!("Using userspace WireGuard implementation");

            #[cfg(feature = "boringtun")]
            let tunnel = runtime
                .block_on(boringtun::open_boringtun_tunnel(config, tun_provider))
                .map(Box::new)?;

            #[cfg(not(feature = "boringtun"))]
            let tunnel = runtime
                .block_on(wireguard_go::open_wireguard_go_tunnel(
                    config,
                    _log_path,
                    setup_done_tx,
                    route_manager,
                ))
                .map(Box::new)?;
            Ok(tunnel)
        } else {
            log::debug!("Using kernel WireGuard implementation");

            wireguard_nt::WgNtTunnel::start_tunnel(config, _log_path, resource_dir, setup_done_tx)
                .map(|tun| Box::new(tun) as Box<dyn Tunnel + 'static>)
                .map_err(Error::TunnelError)
        }
    }

    #[cfg(target_os = "macos")]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        tun_provider: Arc<std::sync::Mutex<tun_provider::TunProvider>>,
        _userspace_wireguard: bool,
        _log_path: Option<&Path>,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        log::debug!("Using userspace WireGuard implementation");

        #[cfg(not(feature = "boringtun"))]
        let tunnel = runtime
            .block_on(wireguard_go::open_wireguard_go_tunnel(
                config,
                _log_path,
                tun_provider,
            ))
            .map(Box::new)?;

        #[cfg(feature = "boringtun")]
        let tunnel = runtime
            .block_on(boringtun::open_boringtun_tunnel(config, tun_provider))
            .map(Box::new)?;
        Ok(tunnel)
    }

    #[cfg(target_os = "linux")]
    fn open_tunnel(
        runtime: tokio::runtime::Handle,
        config: &Config,
        tun_provider: Arc<std::sync::Mutex<tun_provider::TunProvider>>,
        userspace_wireguard: bool,
        _log_path: Option<&Path>,
    ) -> Result<TunnelType> {
        log::debug!("Tunnel MTU: {}", config.mtu);

        if userspace_wireguard {
            log::debug!("Using userspace WireGuard implementation");

            #[cfg(not(feature = "boringtun"))]
            let f = wireguard_go::open_wireguard_go_tunnel(config, _log_path, tun_provider);

            #[cfg(feature = "boringtun")]
            let f = boringtun::open_boringtun_tunnel(config, tun_provider);

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

                    #[cfg(not(feature = "boringtun"))]
                    {
                        Ok(runtime
                            .block_on(wireguard_go::open_wireguard_go_tunnel(
                                config,
                                _log_path,
                                tun_provider,
                            ))
                            .map(Box::new)?)
                    }
                    #[cfg(feature = "boringtun")]
                    {
                        Ok(runtime
                            .block_on(boringtun::open_boringtun_tunnel(config, tun_provider))
                            .map(Box::new)?)
                    }
                })
        }
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
    fn get_endpoint_routes(
        endpoints: &[std::net::IpAddr],
    ) -> impl Iterator<Item = RequiredRoute> + '_ {
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
        #[allow(unused_variables)] userspace_wireguard: bool,
    ) -> impl Iterator<Item = RequiredRoute> + 'a {
        // e.g. utun4
        let gateway_node = talpid_routing::Node::device(iface_name.to_string());

        // e.g. route to 10.64.0.1 through utun4
        let gateway_routes = std::iter::once(RequiredRoute::new(
            ipnetwork::Ipv4Network::from(config.ipv4_gateway).into(),
            gateway_node.clone(),
        ))
        // same but ipv6
        .chain(config.ipv6_gateway.map(|gateway| {
            RequiredRoute::new(ipnetwork::Ipv6Network::from(gateway).into(), gateway_node)
        }));

        // e.g. utun4 and utun4
        let (node_v4, node_v6) = Self::get_tunnel_nodes(iface_name, config);

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let gateway_routes = gateway_routes.map(move |route| {
            Self::apply_route_mtu_for_multihop(route, config, userspace_wireguard)
        });

        gateway_routes.chain(
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
        )
    }

    /// Return any 0.0.0.0/0 routes specified by the allowed IPs.
    #[cfg(not(target_os = "android"))]
    fn get_post_tunnel_routes<'a>(
        iface_name: &str,
        config: &'a Config,
        #[allow(unused_variables)] userspace_wireguard: bool,
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
            .map(move |route| {
                Self::apply_route_mtu_for_multihop(route, config, userspace_wireguard)
            });

        #[cfg(target_os = "macos")]
        iter.map(move |route| {
            Self::apply_route_mtu_for_multihop(route, config, userspace_wireguard)
        })
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn apply_route_mtu_for_multihop(
        route: RequiredRoute,
        config: &Config,
        userspace_wireguard: bool,
    ) -> RequiredRoute {
        // For userspace multihop, per-route MTU is unnecessary. Packets are not sent back to
        // the tunnel interface, so we're not constrained by its MTU.
        let using_boringtun = userspace_wireguard && cfg!(feature = "boringtun");

        if !config.is_multihop() || using_boringtun {
            route
        } else {
            use talpid_tunnel::{IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, WIREGUARD_HEADER_SIZE};

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
// TODO regular async?
#[async_trait::async_trait]
pub(crate) trait Tunnel: Send + Sync {
    fn get_interface_name(&self) -> String;
    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError>;
    async fn get_tunnel_stats(&self) -> std::result::Result<stats::StatsMap, TunnelError>;
    // TODO regular async?
    fn set_config<'a>(
        &'a mut self,
        _config: Config,
        _daita: Option<DaitaSettings>,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), TunnelError>> + Send + 'a>>;
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

    /// Failed to set up a tunnel device
    #[error("Failed to setup a tunnel device")]
    SetupTunnelDevice(#[source] tun_provider::Error),

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
    #[cfg(any(windows, not(feature = "boringtun")))]
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

/// Get MTU based on the physical interface route
#[cfg(any(target_os = "linux", target_os = "windows"))]
async fn get_route_mtu(
    params: &TunnelParameters,
    route_manager: &talpid_routing::RouteManagerHandle,
) -> u16 {
    // Get the MTU of the device/route
    route_manager
        .get_mtu_for_route(params.connection.peer.endpoint.ip())
        .await
        .unwrap_or(DEFAULT_MTU)
}

/// Get MTU based on the physical interface route
#[cfg(any(target_os = "macos", target_os = "android"))]
#[allow(clippy::unused_async)]
#[allow(unused_variables)]
async fn get_route_mtu(
    params: &TunnelParameters,
    route_manager: &talpid_routing::RouteManagerHandle,
) -> u16 {
    DEFAULT_MTU
}

/// Clamp WireGuard tunnel MTU to reasonable values
fn clamp_tunnel_mtu(params: &TunnelParameters, mtu: u16) -> u16 {
    use talpid_tunnel::{MIN_IPV4_MTU, MIN_IPV6_MTU};

    let min_mtu = match params.generic_options.enable_ipv6 {
        false => MIN_IPV4_MTU,
        true => MIN_IPV6_MTU,
    };

    // Some users experience fragmentation issues even when we take the interface MTU and
    // subtract the header sizes. This is likely due to some program that they use which does
    // not change the interface MTU but adds its own header onto the outgoing packets. For this
    // reason we subtract some extra bytes from our MTU in order to give other programs some
    // safety margin.
    const MTU_SAFETY_MARGIN: u16 = 60;

    // The largest peer MTU that we allow
    // TODO: userspace multihop?
    let max_peer_mtu: u16 = 1500 - MTU_SAFETY_MARGIN - wireguard_overhead(&params.connection.peer);

    mtu.clamp(min_mtu, max_peer_mtu)
}

/// Calculates WireGuard per-packet overhead
const fn wireguard_overhead(peer: &PeerConfig) -> u16 {
    match peer.endpoint.ip() {
        IpAddr::V4(..) => IPV4_HEADER_SIZE + WIREGUARD_HEADER_SIZE,
        IpAddr::V6(..) => IPV6_HEADER_SIZE + WIREGUARD_HEADER_SIZE,
    }
}
