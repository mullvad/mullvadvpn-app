#[cfg(target_os = "android")]
use super::Error;
#[cfg(target_os = "android")]
use super::config;
use super::{
    Config, Tunnel, TunnelError,
    stats::{Stats, StatsMap},
};
#[cfg(target_os = "linux")]
use crate::config::MULLVAD_INTERFACE_NAME;
#[cfg(target_os = "android")]
use crate::connectivity;
use crate::logging::{clean_up_logging, initialize_logging};
#[cfg(all(unix, not(target_os = "android")))]
use ipnetwork::IpNetwork;
#[cfg(daita)]
use std::ffi::CString;
#[cfg(unix)]
use std::sync::{Arc, Mutex};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};
#[cfg(target_os = "android")]
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::Error as TunProviderError;
#[cfg(not(target_os = "windows"))]
use talpid_tunnel::tun_provider::{Tun, TunProvider};
#[cfg(daita)]
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::BoxedError;
#[cfg(target_os = "android")]
use talpid_types::net::wireguard::PeerConfig;

#[cfg(unix)]
const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

/// Maximum number of events that can be stored in the underlying buffer
#[cfg(daita)]
const DAITA_EVENTS_CAPACITY: u32 = 2048;

/// Maximum number of actions that can be stored in the underlying buffer
#[cfg(daita)]
const DAITA_ACTIONS_CAPACITY: u32 = 1024;

type Result<T> = std::result::Result<T, TunnelError>;

struct LoggingContext {
    ordinal: u64,
    #[allow(dead_code)]
    path: Option<PathBuf>,
}

impl LoggingContext {
    fn new(ordinal: u64, path: Option<PathBuf>) -> Self {
        LoggingContext { ordinal, path }
    }
}

impl Drop for LoggingContext {
    fn drop(&mut self) {
        clean_up_logging(self.ordinal);
    }
}

pub struct WgGoTunnel {
    // This should never be [None] _unless_ we have just called [Self::stop] and
    // we're restarting the tunnel.
    inner: Option<WgGoTunnelState>,
    #[cfg(target_os = "android")]
    r#type: Circuit,
}

#[cfg(target_os = "android")]
#[derive(Clone, Copy, Debug)]
enum Circuit {
    Singlehop,
    Multihop,
}

/// Configure and start a Wireguard-go tunnel.
#[allow(clippy::unused_async)]
pub(crate) async fn open_wireguard_go_tunnel(
    config: &Config,
    log_path: Option<&Path>,
    #[cfg(unix)] tun_provider: Arc<std::sync::Mutex<talpid_tunnel::tun_provider::TunProvider>>,
    #[cfg(target_os = "android")] route_manager: RouteManagerHandle,
    #[cfg(windows)] setup_done_tx: futures::channel::mpsc::Sender<
        std::result::Result<(), BoxedError>,
    >,
    #[cfg(windows)] route_manager: talpid_routing::RouteManagerHandle,
    #[cfg(target_os = "android")] gateway_only: bool,
    #[cfg(target_os = "android")] cancel_receiver: connectivity::CancelReceiver,
) -> Result<WgGoTunnel> {
    #[cfg(all(unix, not(target_os = "android")))]
    let routes = config.get_tunnel_destinations();

    #[cfg(all(unix, not(target_os = "android")))]
    let tunnel = WgGoTunnel::start_tunnel(config, log_path, tun_provider, routes)?;

    #[cfg(target_os = "windows")]
    let tunnel = WgGoTunnel::start_tunnel(config, log_path, route_manager, setup_done_tx).await?;

    // Android uses multihop implemented in Mullvad's wireguard-go fork. When negotiating
    // with an ephemeral peer, this multihop strategy require us to restart the tunnel
    // every time we want to reconfigure it. As such, we will actually start a multihop
    // tunnel at a later stage, after we have negotiated with the first ephemeral peer.
    // At this point, when the tunnel *is first started*, we establish a regular, singlehop
    // tunnel to where the ephemeral peer resides.
    //
    // Refer to `docs/architecture.md` for details on how to use multihop + PQ.
    #[cfg(target_os = "android")]
    let config = match gateway_only {
        true => config::patch_allowed_ips(config.clone()),
        false => config.clone(),
    };

    #[cfg(target_os = "android")]
    let tunnel = if let Some(exit_peer) = &config.exit_peer {
        WgGoTunnel::start_multihop_tunnel(
            &config,
            exit_peer,
            log_path,
            tun_provider,
            route_manager,
            cancel_receiver,
        )
        .await?
    } else {
        WgGoTunnel::start_tunnel(
            #[allow(clippy::needless_borrow)]
            &config,
            log_path,
            tun_provider,
            route_manager,
            cancel_receiver,
        )
        .await?
    };

    Ok(tunnel)
}

impl WgGoTunnel {
    fn handle(&self) -> &WgGoTunnelState {
        debug_assert!(&self.inner.is_some());
        self.inner.as_ref().unwrap()
    }

    fn handle_mut(&mut self) -> &mut WgGoTunnelState {
        debug_assert!(&self.inner.is_some());
        self.inner.as_mut().unwrap()
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(tunnel) = self.inner.take() {
            tunnel
                .tunnel_handle
                .turn_off()
                .map_err(|e| TunnelError::StopWireguardError(Box::new(e)))?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    #[allow(clippy::unused_async)]
    async fn set_config(&mut self, config: Config) -> Result<()> {
        self.handle_mut().set_config(config)
    }

    #[cfg(target_os = "android")]
    pub async fn set_config(&mut self, config: Config) -> Result<()> {
        let log_path = self.handle()._logging_context.path.clone();
        let cancel_receiver = self.handle().cancel_receiver.clone();
        let tun_provider = Arc::clone(&self.handle().tun_provider);
        let route_manager = self.handle().route_manager.clone();

        match self.r#type {
            Circuit::Multihop if !config.is_multihop() => {
                self.stop()?;
                *self = Self::start_tunnel(
                    &config,
                    log_path.as_deref(),
                    tun_provider,
                    route_manager,
                    cancel_receiver,
                )
                .await?;
            }
            Circuit::Singlehop if config.is_multihop() => {
                self.stop()?;
                *self = Self::start_multihop_tunnel(
                    &config,
                    &config.exit_peer.clone().unwrap().clone(),
                    log_path.as_deref(),
                    tun_provider,
                    route_manager,
                    cancel_receiver,
                )
                .await?;
            }
            Circuit::Singlehop => {
                self.handle_mut().set_config(config)?;
                // HACK: Check if the tunnel is working by sending a ping in the tunnel.
                // This check is needed for PQ connections to be established.
                self.ensure_tunnel_is_running().await?;
            }
            Circuit::Multihop => {
                self.handle_mut().set_config(config)?;
                // HACK: Check if the tunnel is working by sending a ping in the tunnel.
                // This check is needed for PQ connections to be established.
                self.ensure_tunnel_is_running().await?;
            }
        };
        Ok(())
    }
}

pub(crate) struct WgGoTunnelState {
    interface_name: String,
    tunnel_handle: wireguard_go_rs::Tunnel,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    #[cfg(unix)]
    _tunnel_device: Tun,
    // context that maps to fs::File instance and stores the file path, used with logging callback
    _logging_context: LoggingContext,
    #[cfg(target_os = "android")]
    tun_provider: Arc<Mutex<TunProvider>>,
    #[cfg(target_os = "android")]
    route_manager: RouteManagerHandle,
    #[cfg(daita)]
    config: Config,
    /// This is used to cancel the connectivity checks that occur when toggling multihop
    #[cfg(target_os = "android")]
    cancel_receiver: connectivity::CancelReceiver,
    /// Default route change callback. This is used to rebind the endpoint socket when the default
    /// route (network) is changed.
    #[cfg(target_os = "windows")]
    _socket_update_cb: Option<talpid_routing::CallbackHandle>,
}

impl WgGoTunnelState {
    fn set_config(&mut self, config: Config) -> Result<()> {
        let wg_config_str = config.to_userspace_format();

        self.tunnel_handle
            .set_config(&wg_config_str)
            .map_err(|_| TunnelError::SetConfigError)?;

        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();

        // When reapplying the config, the endpoint socket may be discarded
        // and needs to be excluded again
        #[cfg(target_os = "android")]
        {
            let socket_v4 = self.tunnel_handle.get_socket_v4();
            let socket_v6 = self.tunnel_handle.get_socket_v6();
            let mut provider = tun_provider.lock().unwrap();
            provider
                .bypass(&socket_v4)
                .map_err(super::TunnelError::BypassError)?;
            provider
                .bypass(&socket_v6)
                .map_err(super::TunnelError::BypassError)?;
        }

        Ok(())
    }
}

impl WgGoTunnel {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        let (tunnel_device, tunnel_fd) = Self::get_tunnel(tun_provider, config, routes)?;

        let interface_name = tunnel_device
            .interface_name()
            .map_err(TunnelError::SetupTunnelDevice)?;
        let wg_config_str = config.to_userspace_format();
        let logging_context = initialize_logging(log_path)
            .map(|ordinal| LoggingContext::new(ordinal, log_path.map(Path::to_owned)))
            .map_err(TunnelError::LoggingError)?;

        let mtu = config.mtu as isize;

        let handle = wireguard_go_rs::Tunnel::turn_on(
            mtu,
            &wg_config_str,
            tunnel_fd,
            Some(logging::wg_go_logging_callback),
            logging_context.ordinal,
        )
        .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        let tunnel = WgGoTunnelState {
            interface_name,
            tunnel_handle: handle,
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
            #[cfg(daita)]
            config: config.clone(),
        };

        Ok(WgGoTunnel {
            inner: Some(tunnel),
        })
    }

    #[cfg(target_os = "windows")]
    pub async fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        route_manager: talpid_routing::RouteManagerHandle,
        mut setup_done_tx: futures::channel::mpsc::Sender<std::result::Result<(), BoxedError>>,
    ) -> Result<Self> {
        use futures::SinkExt;
        use talpid_types::ErrorExt;

        let wg_config_str = config.to_userspace_format();
        let logging_context = initialize_logging(log_path)
            .map(|ordinal| LoggingContext::new(ordinal, log_path.map(Path::to_owned)))
            .map_err(TunnelError::LoggingError)?;

        let socket_update_cb = route_manager
            .add_default_route_change_callback(Box::new(Self::default_route_changed_callback))
            .await
            .ok();
        if socket_update_cb.is_none() {
            log::warn!("Failed to register default route callback");
        }

        let handle = wireguard_go_rs::Tunnel::turn_on(
            c"Mullvad",
            config.mtu,
            &wg_config_str,
            Some(logging::wg_go_logging_callback),
            logging_context.ordinal,
        )
        .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        let has_ipv6 = config.ipv6_gateway.is_some();

        let luid = handle.luid().to_owned();

        let setup_task = async move {
            log::debug!("Waiting for tunnel IP interfaces to arrive");
            talpid_windows::net::wait_for_interfaces(luid, true, has_ipv6)
                .await
                .map_err(|e| BoxedError::new(TunnelError::SetupIpInterfaces(e)))?;
            log::debug!("Waiting for tunnel IP interfaces: Done");

            if let Err(error) =
                talpid_tunnel::network_interface::initialize_interfaces(luid, None, None)
            {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to set tunnel interface metric"),
                );
            }

            Ok(())
        };

        tokio::spawn(async move {
            let _ = setup_done_tx.send(setup_task.await).await;
        });

        let interface_name = handle.name();

        Ok(WgGoTunnel {
            inner: Some(WgGoTunnelState {
                interface_name: interface_name.to_owned(),
                tunnel_handle: handle,
                _logging_context: logging_context,
                _socket_update_cb: socket_update_cb,
                #[cfg(daita)]
                config: config.clone(),
            }),
        })
    }

    // Callback to be used to rebind the tunnel sockets when the default route changes
    #[cfg(target_os = "windows")]
    fn default_route_changed_callback(
        event_type: talpid_routing::EventType<'_>,
        _family: talpid_windows::net::AddressFamily,
    ) {
        use talpid_routing::EventType::*;
        match event_type {
            // if there is no new default route, or if the route was removed, update the bind
            Updated(_) | Removed => wireguard_go_rs::update_bind(),
            // ignore interface updates that don't affect the interface to use
            UpdatedDetails(_) => (),
        }
    }

    #[cfg(unix)]
    fn get_tunnel(
        tun_provider: Arc<Mutex<TunProvider>>,
        config: &Config,
        #[cfg(not(target_os = "android"))] routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<(Tun, std::os::fd::OwnedFd)> {
        let mut last_error = None;
        let mut tun_provider = tun_provider.lock().unwrap();

        let tun_config = tun_provider.config_mut();
        #[cfg(target_os = "linux")]
        {
            tun_config.name = Some(MULLVAD_INTERFACE_NAME.to_string());
            tun_config.packet_information = true;
        }
        tun_config.addresses = config.tunnel.addresses.clone();
        tun_config.ipv4_gateway = config.ipv4_gateway;
        tun_config.ipv6_gateway = config.ipv6_gateway;
        tun_config.mtu = config.mtu;

        #[cfg(not(target_os = "android"))]
        {
            tun_config.routes = routes.collect();
        }

        #[cfg(target_os = "android")]
        {
            // Route everything into the tunnel and have wireguard-go act as a firewall when
            // blocking. These will not necessarily be the actual routes used by android. Those will
            // be generated at a later stage e.g. if Local Network Sharing is enabled.
            // If IPv6 is not enabled in the tunnel we should not route IPv6 traffic as this
            // leads to leaks.
            tun_config.routes = if config.ipv6_gateway.is_some() {
                vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()]
            } else {
                vec!["0.0.0.0/0".parse().unwrap()]
            }
        }

        for _ in 1..=MAX_PREPARE_TUN_ATTEMPTS {
            let tunnel_device = tun_provider
                .open_tun()
                .map_err(TunnelError::SetupTunnelDevice)?;

            match nix::unistd::dup(&tunnel_device) {
                Ok(fd) => return Ok((tunnel_device, fd)),
                #[cfg(not(target_os = "macos"))]
                Err(error @ nix::errno::Errno::EBADFD) => last_error = Some(error),
                Err(error @ nix::errno::Errno::EBADF) => last_error = Some(error),
                Err(error) => return Err(TunnelError::FdDuplicationError(error)),
            }
        }

        Err(TunnelError::FdDuplicationError(
            last_error.expect("Should be collected in loop"),
        ))
    }
}

#[cfg(target_os = "android")]
impl WgGoTunnel {
    pub async fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        route_manager: RouteManagerHandle,
        cancel_receiver: connectivity::CancelReceiver,
    ) -> Result<Self> {
        route_manager
            .clear_route_cache()
            .await
            .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        let (mut tunnel_device, tunnel_fd) = Self::get_tunnel(Arc::clone(&tun_provider), config)?;
        let is_new_tunnel = tunnel_device.is_new;

        let interface_name: String = tunnel_device
            .interface_name()
            .expect("Tunnel name trivially exists on Android");
        let logging_context = initialize_logging(log_path)
            .map(|ordinal| LoggingContext::new(ordinal, log_path.map(Path::to_owned)))
            .map_err(TunnelError::LoggingError)?;

        let wg_config_str = config.to_userspace_format();

        let handle = wireguard_go_rs::Tunnel::turn_on(
            &wg_config_str,
            tunnel_fd,
            Some(logging::wg_go_logging_callback),
            logging_context.ordinal,
        )
        .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        Self::bypass_tunnel_sockets(&handle, &mut tunnel_device)
            .map_err(TunnelError::BypassError)?;

        let tunnel = WgGoTunnelState {
            interface_name,
            tunnel_handle: handle,
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
            tun_provider,
            route_manager,
            #[cfg(daita)]
            config: config.clone(),
            cancel_receiver,
        };
        let tunnel = Self {
            inner: Some(tunnel),
            r#type: Circuit::Singlehop,
        };

        if is_new_tunnel {
            tunnel.wait_for_routes().await?;
        }

        // HACK: Check if the tunnel is working by sending a ping in the tunnel. For other platforms
        // this is done in the tunnel_fut in WireguardMonitor.start, however that caused it to crash
        // in GO on Android.
        //
        // Tracked by DROID-1825 (Investigate GO crash issue with runtime.GC())
        tunnel.ensure_tunnel_is_running().await?;

        Ok(tunnel)
    }

    pub async fn start_multihop_tunnel(
        config: &Config,
        exit_peer: &PeerConfig,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        route_manager: RouteManagerHandle,
        cancel_receiver: connectivity::CancelReceiver,
    ) -> Result<Self> {
        route_manager
            .clear_route_cache()
            .await
            .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        let (mut tunnel_device, tunnel_fd) = Self::get_tunnel(Arc::clone(&tun_provider), config)?;
        let is_new_tunnel = tunnel_device.is_new;

        let interface_name: String = tunnel_device
            .interface_name()
            .expect("Tunnel name trivially exists on Android");
        let logging_context = initialize_logging(log_path)
            .map(|ordinal| LoggingContext::new(ordinal, log_path.map(Path::to_owned)))
            .map_err(TunnelError::LoggingError)?;

        let entry_config_str = config::userspace_format(
            &config.tunnel.private_key,
            std::iter::once(&config.entry_peer),
        );

        let exit_config_str =
            config::userspace_format(&config.tunnel.private_key, std::iter::once(exit_peer));

        let private_ip = config
            .tunnel
            .addresses
            .iter()
            .find(|addr| addr.is_ipv4())
            .map(|addr| CString::new(addr.to_string()).unwrap())
            .ok_or(TunnelError::SetConfigError)?;

        let handle = wireguard_go_rs::Tunnel::turn_on_multihop(
            &exit_config_str,
            &entry_config_str,
            &private_ip,
            tunnel_fd,
            Some(logging::wg_go_logging_callback),
            logging_context.ordinal,
        )
        .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        Self::bypass_tunnel_sockets(&handle, &mut tunnel_device)
            .map_err(TunnelError::BypassError)?;

        let tunnel = WgGoTunnelState {
            interface_name,
            tunnel_handle: handle,
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
            tun_provider,
            route_manager,
            #[cfg(daita)]
            config: config.clone(),
            cancel_receiver: cancel_receiver.clone(),
        };

        let tunnel = Self {
            inner: Some(tunnel),
            r#type: Circuit::Multihop,
        };

        if is_new_tunnel {
            tunnel.wait_for_routes().await?;
        }

        // HACK: Check if the tunnel is working by sending a ping in the tunnel. For other platforms
        // this is done in the tunnel_fut in WireguardMonitor.start, however that caused it to crash
        // in GO on Android.
        //
        // Tracked by DROID-1825 (Investigate GO crash issue with runtime.GC())
        tunnel.ensure_tunnel_is_running().await?;

        Ok(tunnel)
    }

    fn bypass_tunnel_sockets(
        handle: &wireguard_go_rs::Tunnel,
        tunnel_device: &mut Tun,
    ) -> std::result::Result<(), TunProviderError> {
        let socket_v4 = handle.get_socket_v4();
        let socket_v6 = handle.get_socket_v6();

        tunnel_device.bypass(&socket_v4)?;
        tunnel_device.bypass(&socket_v6)?;

        Ok(())
    }

    /// There is a brief period of time between setting up a Wireguard-go tunnel and the tunnel being ready to serve
    /// traffic. This function blocks until the tunnel starts to serve traffic or until [connectivity::Check] times out.
    async fn wait_for_routes(&self) -> Result<()> {
        let expected_routes = self.handle().tun_provider.lock().unwrap().real_routes();

        // Wait for routes to come up
        self.handle()
            .route_manager
            .clone()
            .wait_for_routes(expected_routes)
            .await
            .map_err(Error::SetupRoutingError)
            .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;

        Ok(())
    }
    async fn ensure_tunnel_is_running(&self) -> Result<()> {
        let addr = self.handle().config.ipv4_gateway;
        let cancel_receiver = self.handle().cancel_receiver.clone();
        let mut check = connectivity::Check::new(addr, 0, cancel_receiver)
            .map_err(|err| TunnelError::RecoverableStartWireguardError(Box::new(err)))?;

        // TODO: retry attempt?

        let connection_established = check
            .establish_connectivity(self)
            .await
            .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;

        // Timed out
        if !connection_established {
            return Err(TunnelError::RecoverableStartWireguardError(Box::new(
                super::Error::TimeoutError,
            )));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> String {
        self.handle().interface_name.clone()
    }

    fn stop(mut self: Box<Self>) -> Result<()> {
        WgGoTunnel::stop(&mut self)?;
        Ok(())
    }

    async fn get_tunnel_stats(&self) -> Result<StatsMap> {
        // NOTE: wireguard-go might perform blocking I/O, but it's most likely not a problem
        self.handle()
            .tunnel_handle
            .get_config(|cstr| {
                Stats::parse_config_str(cstr.to_str().expect("Go strings are always UTF-8"))
            })
            .ok_or(TunnelError::GetConfigError)?
            .map_err(|error| TunnelError::StatsError(BoxedError::new(error)))
    }

    fn set_config(
        &mut self,
        config: Config,
        daita: Option<DaitaSettings>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.set_config(config).await?;

            if let Some(daita) = daita {
                log::info!("Initializing DAITA for wireguard device");
                let peer_public_key = self.handle().config.entry_peer.public_key.clone();

                let machines = daita.client_machines.join("\n");
                let machines =
                    CString::new(machines).map_err(|err| TunnelError::StartDaita(Box::new(err)))?;

                self.handle()
                    .tunnel_handle
                    .activate_daita(
                        peer_public_key.as_bytes(),
                        &machines,
                        daita.max_padding_frac,
                        daita.max_blocking_frac,
                        DAITA_EVENTS_CAPACITY,
                        DAITA_ACTIONS_CAPACITY,
                    )
                    .map_err(|e| TunnelError::StartDaita(Box::new(e)))?;
            }

            Ok(())
        })
    }
}

mod stats {
    use super::{Stats, StatsMap};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[derive(thiserror::Error, Debug, PartialEq)]
    pub enum Error {
        #[error("Failed to parse peer pubkey from string \"{0}\"")]
        PubKeyParse(String, #[source] hex::FromHexError),

        #[error("Failed to parse integer from string \"{0}\"")]
        IntParse(String, #[source] std::num::ParseIntError),
    }

    impl Stats {
        pub fn parse_config_str(config: &str) -> std::result::Result<StatsMap, Error> {
            let mut map = StatsMap::new();

            let mut peer = None;

            let mut tx_bytes = None;
            let mut rx_bytes = None;
            let mut last_handshake_time_sec = None;
            let mut last_handshake_time_nsec = None;

            // parts iterates over keys and values
            let parts = config.split('\n').filter_map(|line| {
                let mut pair = line.split('=');
                let key = pair.next()?;
                let value = pair.next()?;
                Some((key, value))
            });

            for (key, value) in parts {
                match key {
                    "public_key" => {
                        let mut buffer = [0u8; 32];
                        hex::decode_to_slice(value, &mut buffer)
                            .map_err(|err| Error::PubKeyParse(value.to_string(), err))?;
                        peer = Some(buffer);
                        tx_bytes = None;
                        rx_bytes = None;
                        last_handshake_time_sec = None;
                        last_handshake_time_nsec = None;
                    }
                    "rx_bytes" => {
                        rx_bytes = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }
                    "tx_bytes" => {
                        tx_bytes = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }
                    "last_handshake_time_sec" => {
                        last_handshake_time_sec = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }
                    "last_handshake_time_nsec" => {
                        last_handshake_time_nsec = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }

                    _ => continue,
                }

                if let (Some(peer_val), Some(tx_bytes_val), Some(rx_bytes_val)) =
                    (peer, tx_bytes, rx_bytes)
                {
                    let last_handshake_time = || -> Option<SystemTime> {
                        let handshake_sec = last_handshake_time_sec?;
                        let handshake_nsec = last_handshake_time_nsec?;
                        // handshake_{sec,nsec} are relative to UNIX_EPOCH
                        // https://www.wireguard.com/xplatform/
                        Some(UNIX_EPOCH + Duration::new(handshake_sec, handshake_nsec))
                    };

                    map.insert(
                        peer_val,
                        Self {
                            tx_bytes: tx_bytes_val,
                            rx_bytes: rx_bytes_val,
                            last_handshake_time: last_handshake_time(),
                            ..Default::default()
                        },
                    );
                    peer = None;
                    tx_bytes = None;
                    rx_bytes = None;
                    last_handshake_time_sec = None;
                    last_handshake_time_nsec = None;
                }
            }
            Ok(map)
        }
    }

    #[cfg(test)]
    mod test {
        use super::super::stats::{Error, Stats};

        #[test]
        fn test_parsing() {
            let valid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=2740\nrx_bytes=2396\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
            let pubkey = [0u8; 32];

            let stats = Stats::parse_config_str(valid_input).expect("Failed to parse valid input");
            assert_eq!(stats.len(), 1);
            let actual_keys: Vec<[u8; 32]> = stats.keys().cloned().collect();
            assert_eq!(actual_keys, [pubkey]);
            assert_eq!(stats[&pubkey].rx_bytes, 2396);
            assert_eq!(stats[&pubkey].tx_bytes, 2740);
        }

        #[test]
        fn test_parsing_invalid_input() {
            let invalid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=27error40\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
            let invalid_str = "27error40".to_string();
            let int_err = invalid_str.parse::<u64>().unwrap_err();

            assert_eq!(
                Stats::parse_config_str(invalid_input),
                Err(Error::IntParse(invalid_str, int_err))
            );
        }
    }
}

mod logging {
    use super::super::logging::{LogLevel, log};
    use std::ffi::c_char;

    // Callback that receives messages from WireGuard
    //
    // # Safety
    // - `msg` must be a valid pointer to a null-terminated UTF-8 string.
    pub unsafe extern "system" fn wg_go_logging_callback(
        level: WgLogLevel,
        msg: *const c_char,
        context: u64,
    ) {
        let managed_msg = if !msg.is_null() {
            // SAFETY: caller promises that the pointer is valid.
            unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() }
        } else {
            "Logging message from WireGuard is NULL".to_string()
        };

        let level = match level {
            WG_GO_LOG_VERBOSE => LogLevel::Verbose,
            _ => LogLevel::Error,
        };

        log(context, level, "wireguard-go", &managed_msg);
    }

    // wireguard-go supports log levels 0 through 3 with 3 being the most verbose
    // const WG_GO_LOG_SILENT: WgLogLevel = 0;
    // const WG_GO_LOG_ERROR: WgLogLevel = 1;
    const WG_GO_LOG_VERBOSE: WgLogLevel = 2;

    pub type WgLogLevel = u32;
}
