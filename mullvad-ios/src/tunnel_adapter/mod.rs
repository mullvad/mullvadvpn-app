#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub(crate) mod ffi;
mod params;
mod pinger;
pub(crate) mod tun_device;

use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use gotatun::{
    device::{DeviceBuilder, Peer},
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::MtuWatcher,
    udp::{
        UdpTransportFactory, UdpTransportFactoryParams,
        channel::new_udp_tun_channel,
        socket::{UdpSocket, UdpSocketFactory},
    },
    x25519::StaticSecret,
};
use talpid_net::bypass::NoopBypass;
use talpid_netstack::{
    ip_mux::ip_mux,
    smoltcp_network::{SmoltcpHandle, SmoltcpNetworkConfig, smoltcp_network},
};
use talpid_tunnel_config_client::negotiation::{
    Negotiables, NegotiationConfig, NegotiationError, Relay, Relays, negotiate_ephemeral_peers,
};
use talpid_types::{
    ErrorExt,
    net::wireguard::{PresharedKey, PrivateKey, PublicKey},
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use tunnel_obfuscation::{
    create_transport,
    gotatun_transport::{
        MaybeObfuscatingRecv, MaybeObfuscatingSend, MaybeObfuscatingTransportFactory,
        RunningObfuscation,
    },
};

use self::pinger::SmoltcpPinger;
use self::tun_device::IosTunDevice;

pub use self::params::{ObfuscationParameters, PeerParameters, TunnelParameters};

/// A UDP transport bound ahead of the tunnel starting.
/// Allowing them to bind ahead of time allows for reusing them and also lets the tunnel connection
/// fail fast.
#[derive(Clone)]
pub struct BoundUdpTransports {
    socket: Arc<Mutex<UdpSocket>>,
}

impl BoundUdpTransports {
    /// Bind the socket, or fail describing why.
    pub async fn bind() -> io::Result<Self> {
        let (socket, _recv) = UdpSocketFactory::default().bind(&Self::params()).await?;
        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
        })
    }

    fn params() -> UdpTransportFactoryParams {
        UdpTransportFactoryParams {
            addr: None,
            port: 0,
        }
    }

    /// Rebind existing socket. It is expected that the associated GotaTun device will be suspended
    /// whilst the socket is rebound.
    pub async fn rebind(&self) -> io::Result<()> {
        let mut socket = self.socket.lock().await;
        let (new_socket, _recv) = UdpSocketFactory::default().bind(&Self::params()).await?;
        *socket = new_socket;
        Ok(())
    }
}

impl UdpTransportFactory for BoundUdpTransports {
    type Send = UdpSocket;
    type Recv = UdpSocket;

    async fn bind(
        &mut self,
        _params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let socket = self.socket.lock().await;
        Ok((socket.clone(), socket.clone()))
    }
}

/// The ingress device's UDP transport: the pre-bound socket, obfuscated in place.
///
/// The obfuscation is held the way [`BoundUdpTransports`] holds the socket beneath it: a device
/// reads either only when it binds, so both are replaced by suspending the device, swapping, and
/// waking it.
#[derive(Clone)]
pub struct ObfuscatingTransports {
    udp: BoundUdpTransports,
    obfuscation: Arc<Mutex<Option<RunningObfuscation>>>,
    /// The relay this addresses. See [`MaybeObfuscatingTransportFactory`].
    peer_endpoint: SocketAddr,
}

impl ObfuscatingTransports {
    fn new(
        udp: BoundUdpTransports,
        obfuscation: Option<RunningObfuscation>,
        peer_endpoint: SocketAddr,
    ) -> Self {
        Self {
            udp,
            obfuscation: Arc::new(Mutex::new(obfuscation)),
            peer_endpoint,
        }
    }

    /// Replace the obfuscation. The device uses it from its next bind on.
    async fn replace(&self, obfuscation: Option<RunningObfuscation>) {
        *self.obfuscation.lock().await = obfuscation;
    }
}

impl UdpTransportFactory for ObfuscatingTransports {
    type Send = MaybeObfuscatingSend<UdpSocket>;
    type Recv = MaybeObfuscatingRecv<UdpSocket>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let obfuscation = self.obfuscation.lock().await.clone();
        MaybeObfuscatingTransportFactory::new(self.udp.clone(), obfuscation, self.peer_endpoint)
            .bind(params)
            .await
    }
}

/// Error from a phase of [`IosTunnelAdapter::run`].
pub enum TunnelError {
    GotaTunDeviceError(gotatun::device::Error),
    MultihopEntryDeviceError(gotatun::device::Error),
    MultihopExitDeviceError(gotatun::device::Error),
    ObfuscationProxyError(ObfuscationProxyError),
    ICMPSocketError(io::Error),
    Timeout,
    TunnelDevice(io::Error),
    NegotiatePQError(NegotiationError),
}

impl std::fmt::Display for TunnelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelError::GotaTunDeviceError(msg) => write!(f, "GotaTun device error: {msg}"),
            TunnelError::MultihopEntryDeviceError(msg) => {
                write!(f, "Multihop entry device error: {msg}")
            }
            TunnelError::MultihopExitDeviceError(msg) => {
                write!(f, "Multihop exit device error: {msg}")
            }
            TunnelError::ObfuscationProxyError(e) => write!(f, "Obfuscation proxy error: {e}"),
            TunnelError::ICMPSocketError(msg) => write!(f, "ICMP socket error: {msg}"),
            TunnelError::Timeout => write!(f, "Timeout"),
            TunnelError::TunnelDevice(msg) => write!(f, "Tunnel device error: {msg}"),
            TunnelError::NegotiatePQError(e) => {
                f.write_str(&e.display_chain_with_msg("Negotiate PQ error"))
            }
        }
    }
}

pub enum ObfuscationProxyError {
    InvalidQuicToken(String),
    LocalSocketError(tunnel_obfuscation::Error),
}

impl std::fmt::Display for ObfuscationProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObfuscationProxyError::InvalidQuicToken(msg) => write!(f, "Invalid QUIC token: {msg}"),
            ObfuscationProxyError::LocalSocketError(msg) => write!(f, "Local socket error: {msg}"),
        }
    }
}

/// Result of [`IosTunnelAdapter::negotiate_pq`]: the keys and peers to configure
/// the final device(s) with `(entry, exit_key, exit_peer)`. `entry` is `Some`
/// only for multihop PQ.
type PqResult = (Option<(StaticSecret, Peer)>, StaticSecret, Peer);

// Connectivity timeouts
const PING_INTERVAL: Duration = Duration::from_secs(3);
const CONNECTIVITY_CHECK_INTERVAL: Duration = Duration::from_millis(200);
/// After this long without any rx, consider the connection lost.
/// WireGuard keepalives are typically every ~25s, so 2 minutes gives plenty of margin.
const MONITOR_TIMEOUT: Duration = Duration::from_secs(120);
/// After a suspension at least this long, restart the obfuscator on wake rather than trust it
/// still holds a healthy connection.
const SLEEP_CYCLE_RESET_THRESHOLD: Duration = Duration::from_secs(120);

/// Callbacks from the tunnel adapter to Swift.
pub trait TunnelCallbackHandler: Send + Sync + 'static {
    fn on_connected(&self);
    fn on_timeout(&self);
    fn on_error(&self, error: TunnelError);
}

/// What the Swift side asks the running tunnel to do.
pub(crate) enum TunnelAdapterChannelCommand {
    Wake,
    Suspend,
    Stop,
    BumpSockets,
}

/// All state of a connected tunnel, kept so that it can be suspended, woken, and moved onto
/// fresh sockets while it runs.
struct ActiveConnection {
    devices: Devices,
    transport_provider: BoundUdpTransports,
    /// When the tunnel was last suspended, to decide whether to restart the obfuscator on wake.
    last_suspended_at: std::sync::Mutex<Option<talpid_time::Instant>>,
    obfuscation: ObfuscatingTransports,
    config: TunnelParameters,
    /// Key the ingress device handshakes with, which LWO obfuscates for.
    ingress_public_key: PublicKey,
}

impl ActiveConnection {
    /// Move the tunnel onto a freshly bound socket, after the network path changed under it.
    ///
    /// The devices are suspended across this because they read the socket, and the obfuscation,
    /// only when they bind.
    async fn bump_sockets(&self) -> Result<(), TunnelError> {
        self.devices.suspend().await;
        if let Err(err) = self.transport_provider.rebind().await {
            log::error!("Failed to rebind sockets: {err}");
        }
        self.restart_obfuscation().await?;
        self.devices.wake().await;
        Ok(())
    }

    /// Replace the obfuscator, e.g. because a path change invalidated a socket it held, or
    /// because it has sat idle through a long suspension.
    async fn restart_obfuscation(&self) -> Result<(), TunnelError> {
        // Drop the old obfuscator before building its replacement, to release its socket, if it
        // has one of its own.
        self.obfuscation.replace(None).await;
        let obfuscation = IosTunnelAdapter::create_obfuscation(&self.config)
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;
        self.obfuscation
            .replace(obfuscation.map(|obfuscation| {
                obfuscation.with_client_public_key(self.ingress_public_key.clone())
            }))
            .await;
        Ok(())
    }

    /// Apply a command from Swift. Returns whether the tunnel is suspended afterwards.
    async fn handle_command(
        &self,
        command: TunnelAdapterChannelCommand,
    ) -> Result<bool, TunnelError> {
        match command {
            TunnelAdapterChannelCommand::Suspend => {
                *self.last_suspended_at.lock().unwrap() = Some(talpid_time::Instant::now());
                self.devices.suspend().await;
                Ok(true)
            }
            TunnelAdapterChannelCommand::Wake => {
                let last_suspended_at = *self.last_suspended_at.lock().unwrap();
                let now = talpid_time::Instant::now();
                let elapsed = now.duration_since(last_suspended_at.unwrap_or(now));
                if elapsed >= SLEEP_CYCLE_RESET_THRESHOLD {
                    self.restart_obfuscation().await?;
                }
                self.devices.wake().await;
                Ok(false)
            }
            TunnelAdapterChannelCommand::BumpSockets => {
                self.bump_sockets().await?;
                Ok(false)
            }
            // Callers intercept `Stop` before it reaches here.
            TunnelAdapterChannelCommand::Stop => unreachable!(),
        }
    }
}

/// A single tunnel connection attempt.
pub struct IosTunnelAdapter {
    stopped: Arc<AtomicBool>,
    tx: UnboundedSender<TunnelAdapterChannelCommand>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl IosTunnelAdapter {
    pub fn start(
        runtime: tokio::runtime::Handle,
        config: TunnelParameters,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
    ) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let task = runtime.spawn(Self::run(config, udp, callback, rx, stopped.clone()));

        Self {
            stopped,
            tx,
            task_handle: Some(task),
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        log::debug!("Stopping device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Stop);
        if let Some(handle) = &self.task_handle {
            handle.abort();
        }
    }

    pub fn recycle_udp_sockets(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("Recycling UDP sockets");
        _ = self.tx.send(TunnelAdapterChannelCommand::BumpSockets);
    }

    pub fn suspend(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("Suspending device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Suspend);
    }

    pub fn wake(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("Waking device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Wake);
    }

    async fn run(
        config: TunnelParameters,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
        rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: Arc<AtomicBool>,
    ) {
        // Every phase below returns a `Result`; the callback is fired exactly
        // once, here, based on the final outcome.
        match Self::run_inner(config, udp, &callback, rx, &stopped).await {
            Ok(()) => Self::fire_timeout(&stopped, &callback),
            Err(
                TunnelError::Timeout | TunnelError::NegotiatePQError(NegotiationError::Timeout),
            ) => Self::fire_timeout(&stopped, &callback),
            Err(error) => Self::fire_error(&stopped, &callback, error),
        }
    }

    async fn run_inner(
        config: TunnelParameters,
        udp: BoundUdpTransports,
        callback: &Arc<dyn TunnelCallbackHandler>,
        mut rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<(), TunnelError> {
        // 1. Create the TUN device from the fd handed over by iOS.
        let tun_dev =
            IosTunDevice::new(config.tun_fd, config.mtu).map_err(TunnelError::TunnelDevice)?;
        let obfuscation = Self::create_obfuscation(&config)
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;

        // 2. Negotiate the PQ/DAITA ephemeral peer(s) over a smoltcp-only device,
        //    or fall back to the static device peer.
        let pq = Self::negotiate_pq(&config, &udp, obfuscation.clone()).await?;
        if stopped.load(Ordering::SeqCst) {
            // Cancelled externally; the outcome below is discarded since `run`
            // no-ops when it sees the tunnel is already stopped.
            return Err(TunnelError::Timeout);
        }

        // 3. After PQ the WireGuard handshake uses the ephemeral ingress key, so point LWO at it.
        let ingress_public_key = Self::ingress_public_key(&pq);
        let obfuscation = ObfuscatingTransports::new(
            udp.clone(),
            obfuscation
                .map(|obfuscation| obfuscation.with_client_public_key(ingress_public_key.clone())),
            config
                .entry_peer
                .as_ref()
                .unwrap_or(&config.exit_peer)
                .endpoint,
        );

        // 4. Build the user-traffic device(s) behind an IpMuxRecv and IpMuxSend (TUN + smoltcp).
        let (smoltcp_handle, ip_recv, ip_send, _smoltcp_guard) =
            smoltcp_network(SmoltcpNetworkConfig {
                ipv4_addr: config.ipv4_addr,
                ipv6_addr: Some(config.ipv6_addr),
                mtu: config.smoltcp_mtu(),
            });
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv, ip_send);

        let devices = Self::build_devices(&config, &obfuscation, pq, mux_recv, mux_send).await?;

        // The tunnel can be suspended or moved onto new sockets from here on, so it is held as
        // one piece for the rest of its life.
        let connection = ActiveConnection {
            devices,
            transport_provider: udp,
            last_suspended_at: std::sync::Mutex::new(None),
            obfuscation,
            config,
            ingress_public_key,
        };
        if stopped.load(Ordering::SeqCst) {
            connection.devices.stop().await;
            return Err(TunnelError::Timeout);
        }

        // 5. Establish connectivity, then monitor it until it drops or we stop.
        let connected = match Self::establish_connectivity(
            &connection,
            &smoltcp_handle,
            &connection.config,
            &mut rx,
            stopped,
        )
        .await
        {
            Ok(connected) => connected,
            Err(e) => {
                connection.devices.stop().await;
                return Err(e);
            }
        };
        if !connected {
            connection.devices.stop().await;
            return Err(TunnelError::Timeout);
        }

        callback.on_connected();
        log::info!("Tunnel connected - starting ongoing monitoring");
        let result = Self::monitor_connectivity(&connection, &mut rx, stopped).await;
        connection.devices.stop().await;
        result?;
        Err(TunnelError::Timeout)
    }

    /// Negotiate the post-quantum / DAITA ephemeral peer(s).
    ///
    /// Returns the `(entry, exit_key, exit_peer)` triple to configure the final
    /// device(s) with - `entry` is `Some` only for multihop PQ.
    async fn negotiate_pq(
        config: &TunnelParameters,
        udp: &BoundUdpTransports,
        obfuscation: Option<RunningObfuscation>,
    ) -> Result<PqResult, TunnelError> {
        // No PQ/DAITA: the device peer is just the static configured exit peer.
        if !(config.enable_pq || config.enable_daita) {
            let private_key = StaticSecret::from(config.private_key);
            return Ok((None, private_key, Self::build_peer(&config.exit_peer)));
        }

        let relay = |peer: &PeerParameters| Relay {
            public_key: PublicKey::from(peer.public_key),
            endpoint: peer.endpoint,
        };
        let relays = match &config.entry_peer {
            None => Relays::Singlehop(relay(&config.exit_peer)),
            Some(entry_peer) => Relays::Multihop {
                entry: relay(entry_peer),
                exit: relay(&config.exit_peer),
            },
        };
        let negotiate = Negotiables {
            post_quantum: config.enable_pq,
            daita: config.enable_daita,
        };
        let negotiation_config = NegotiationConfig {
            private_key: PrivateKey::from(config.private_key),
            tunnel_ipv4: config.ipv4_addr,
            config_service_ip: config.ipv4_gateway,
            relays,
            // iOS only uses LWO v1, which keeps the default timers.
            ingress_timer_params: None,
            timeout: config.establish_timeout(),
            handshake_timeout: config.establish_timeout(),
            // Each relay has a device of its own, so it can have a key of its own.
            separate_exit_key: true,
        };

        let ingress_endpoint = config
            .entry_peer
            .as_ref()
            .unwrap_or(&config.exit_peer)
            .endpoint;
        let ingress_transport = |client_public_key: &PublicKey| {
            let obfuscation = obfuscation
                .clone()
                .map(|obfuscation| obfuscation.with_client_public_key(client_public_key.clone()));
            MaybeObfuscatingTransportFactory::new(udp.clone(), obfuscation, ingress_endpoint)
        };
        let negotiated =
            negotiate_ephemeral_peers(&negotiation_config, negotiate, ingress_transport)
                .await
                .map_err(TunnelError::NegotiatePQError)?;

        let ingress_key = StaticSecret::from(negotiated.private_key.to_bytes());
        let exit_key = negotiated.exit_private_key.as_ref().map_or_else(
            || ingress_key.clone(),
            |exit_private_key| StaticSecret::from(exit_private_key.to_bytes()),
        );
        let exit_peer = Self::build_peer(&config.exit_peer);
        Ok(match &config.entry_peer {
            None => (
                None,
                exit_key,
                with_psk(exit_peer, negotiated.ingress_psk.as_ref()),
            ),
            Some(entry_peer) => (
                Some((
                    ingress_key,
                    with_psk(
                        Self::build_peer(entry_peer),
                        negotiated.ingress_psk.as_ref(),
                    ),
                )),
                exit_key,
                with_psk(exit_peer, negotiated.exit_psk.as_ref()),
            ),
        })
    }

    /// Build the final GotaTun device(s) carrying user traffic and configure
    /// their peers.
    async fn build_devices(
        config: &TunnelParameters,
        obfuscation: &ObfuscatingTransports,
        pq: PqResult,
        mux_recv: tun_device::IosTunIpRecv,
        mux_send: tun_device::IosTunIpSend,
    ) -> Result<Devices, TunnelError> {
        let (pq_entry, pq_exit_key, pq_exit_peer) = pq;

        let Some(entry_peer_config) = config.entry_peer.as_ref() else {
            // Singlehop: one device, mux'd IP pair, obfuscated UDP.
            let device = DeviceBuilder::new()
                .with_udp(obfuscation.clone())
                .with_ip_pair(mux_send, mux_recv)
                .with_private_key(pq_exit_key)
                .with_peer(pq_exit_peer.with_endpoint(config.exit_peer.endpoint))
                .build()
                .await
                .map_err(TunnelError::GotaTunDeviceError)?;
            return Ok(Devices::Singlehop(device));
        };

        // Multihop: exit device tunnels its UDP through the entry device.
        let entry_mtu = MtuWatcher::new(config.mtu)
            .increase(Self::multihop_overhead(entry_peer_config.endpoint))
            .expect("MTU overflow");
        let (tun_channel_tx, tun_channel_rx, udp_channels) =
            new_udp_tun_channel(100, config.ipv4_addr, config.ipv6_addr, entry_mtu);

        let exit_device = DeviceBuilder::new()
            .with_udp(udp_channels)
            .with_ip_pair(mux_send, mux_recv)
            .with_private_key(pq_exit_key)
            .with_peer(pq_exit_peer)
            .build()
            .await
            .map_err(TunnelError::MultihopExitDeviceError)?;

        log::info!(
            "Multihop: entry={}, exit={}",
            entry_peer_config.endpoint,
            config.exit_peer.endpoint
        );

        // Use the PQ entry key if negotiated, otherwise the device key.
        let (entry_key, entry_peer) = pq_entry.unwrap_or_else(|| {
            (
                StaticSecret::from(config.private_key),
                Self::build_peer(entry_peer_config),
            )
        });
        let entry_peer = entry_peer.with_endpoint(entry_peer_config.endpoint);

        let entry_device = match DeviceBuilder::new()
            .with_udp(obfuscation.clone())
            .with_ip_pair(tun_channel_tx, tun_channel_rx)
            .with_peer(entry_peer)
            .with_private_key(entry_key)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                exit_device.stop().await;
                return Err(TunnelError::MultihopEntryDeviceError(e));
            }
        };

        Ok(Devices::Multihop {
            entry: entry_device,
            exit: exit_device,
        })
    }

    /// Ping until the device sees inbound traffic, the establish timeout fires,
    /// or we are stopped. Returns whether the tunnel became connected.
    async fn establish_connectivity(
        connection: &ActiveConnection,
        smoltcp_handle: &SmoltcpHandle,
        config: &TunnelParameters,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<bool, TunnelError> {
        // Bind the socket to the pinger's ident so echo replies reach it.
        let ping_ident: u16 = rand::random();
        let icmp_socket = smoltcp_handle
            .icmp_socket(ping_ident)
            .await
            .map_err(TunnelError::ICMPSocketError)?;
        let mut pinger = SmoltcpPinger::new(icmp_socket, config.ipv4_gateway, ping_ident);

        let establish_timeout = config.establish_timeout();
        log::info!("Establishing connectivity (timeout: {establish_timeout:?})");

        if let Err(e) = pinger.send_icmp().await {
            log::warn!("Initial ping failed: {e}");
        }

        tokio::select! {
            result = Self::wait_for_connectivity(connection, &mut pinger, rx, stopped) => result,
            _ = tokio::time::sleep(establish_timeout) => Ok(false),
        }
    }

    /// Create the obfuscation that reaches the ingress relay. It is used by the temporary devices
    /// that negotiate ephemeral peers, and then reused by the tunnel devices.
    /// Returns `None` if obfuscation is off.
    async fn create_obfuscation(
        config: &TunnelParameters,
    ) -> Result<Option<RunningObfuscation>, ObfuscationProxyError> {
        let obfuscation = match Self::obfuscation_settings(config)? {
            None => return Ok(None),
            // LWO obfuscates each datagram in place, over the socket of the device.
            Some(tunnel_obfuscation::Settings::Lwo(settings)) => RunningObfuscation::Lwo(settings),
            Some(settings) => RunningObfuscation::Transport(
                create_transport(Arc::new(NoopBypass), &settings)
                    .await
                    .map_err(ObfuscationProxyError::LocalSocketError)?,
            ),
        };
        Ok(Some(obfuscation))
    }

    /// The settings of the obfuscator that reaches the ingress relay.
    /// Returns `None` if obfuscation is off.
    fn obfuscation_settings(
        config: &TunnelParameters,
    ) -> Result<Option<tunnel_obfuscation::Settings>, ObfuscationProxyError> {
        let ingress_endpoint = config
            .entry_peer
            .as_ref()
            .unwrap_or(&config.exit_peer)
            .endpoint;

        let settings = match &config.obfuscation {
            ObfuscationParameters::Off => return Ok(None),
            ObfuscationParameters::UdpOverTcp => {
                tunnel_obfuscation::Settings::Udp2Tcp(tunnel_obfuscation::udp2tcp::Settings {
                    peer: ingress_endpoint,
                })
            }
            ObfuscationParameters::Shadowsocks => {
                let wg_ep = localhost_wg_endpoint(ingress_endpoint);
                tunnel_obfuscation::Settings::Shadowsocks(
                    tunnel_obfuscation::shadowsocks::Settings {
                        shadowsocks_endpoint: ingress_endpoint,
                        wireguard_endpoint: wg_ep,
                    },
                )
            }
            ObfuscationParameters::Quic { hostname, token } => {
                let wg_ep = localhost_wg_endpoint(ingress_endpoint);
                let token = token
                    .parse::<tunnel_obfuscation::quic::AuthToken>()
                    .map_err(ObfuscationProxyError::InvalidQuicToken)?;
                tunnel_obfuscation::Settings::Quic(tunnel_obfuscation::quic::Settings::new(
                    ingress_endpoint,
                    hostname.clone(),
                    token,
                    wg_ep,
                ))
            }
            ObfuscationParameters::Lwo { server_public_key } => {
                // Placeholder client key: every user of these settings overrides it with the key
                // of the device the obfuscation is for, via `with_client_public_key`.
                let device_public_key =
                    gotatun::x25519::PublicKey::from(&StaticSecret::from(config.private_key));
                tunnel_obfuscation::Settings::Lwo(tunnel_obfuscation::lwo::Settings {
                    server_addr: ingress_endpoint,
                    client_public_key: talpid_types::net::wireguard::PublicKey::from(
                        device_public_key.to_bytes(),
                    ),
                    server_public_key: talpid_types::net::wireguard::PublicKey::from(
                        *server_public_key,
                    ),
                    version: talpid_types::net::obfuscation::LwoVersion::V1,
                })
            }
        };
        Ok(Some(settings))
    }

    /// Wait for the device to receive traffic (rx_bytes > 0 on any peer).
    async fn wait_for_connectivity(
        connection: &ActiveConnection,
        pinger: &mut SmoltcpPinger,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<bool, TunnelError> {
        let mut last_ping = Instant::now();
        // A suspended tunnel carries nothing, so there is no point pinging over it.
        let mut suspended = false;

        loop {
            if stopped.load(Ordering::SeqCst) {
                return Ok(false);
            }

            if !suspended {
                if connection.devices.has_rx().await {
                    log::debug!("Connectivity established - rx_bytes > 0");
                    return Ok(true);
                }

                if last_ping.elapsed() >= PING_INTERVAL {
                    if let Err(e) = pinger.send_icmp().await {
                        log::warn!("Ping failed: {e}");
                    }
                    last_ping = Instant::now();
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(CONNECTIVITY_CHECK_INTERVAL) => {}
                command = rx.recv() => {
                    match command {
                        // The channel only closes with the adapter, which stops us.
                        Some(TunnelAdapterChannelCommand::Stop) | None => return Ok(false),
                        Some(command) => {
                            suspended = connection.handle_command(command).await?;
                            // Give the tunnel the full ping interval again after it moved sockets.
                            last_ping = Instant::now() - PING_INTERVAL;
                        }
                    }
                }
            }
        }
    }

    /// Monitor an established connection. Returns when connectivity is lost or stopped.
    /// Watch the tunnel until it stops, loses connectivity, or is told to sleep or move sockets.
    async fn monitor_connectivity(
        connection: &ActiveConnection,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<(), TunnelError> {
        let mut last_rx_bytes: usize = 0;
        let mut last_rx_time = Instant::now();
        // A suspended tunnel receives nothing, so the RX deadline must not run while it sleeps.
        let mut suspended = false;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                command = rx.recv() => {
                    match command {
                        // The channel only closes when the adapter is dropped, which stops us.
                        Some(TunnelAdapterChannelCommand::Stop) | None => return Ok(()),
                        Some(command) => {
                            suspended = connection.handle_command(command).await?;
                            // Do not hold a sleep, or the sockets it spanned, against the tunnel.
                            last_rx_time = Instant::now();
                            continue;
                        }
                    }
                }
            }

            if stopped.load(Ordering::SeqCst) {
                return Ok(());
            }
            if suspended {
                continue;
            }

            let total_rx = connection.devices.total_rx().await;

            if total_rx > last_rx_bytes {
                last_rx_bytes = total_rx;
                last_rx_time = Instant::now();
            } else if last_rx_time.elapsed() > MONITOR_TIMEOUT {
                log::warn!("No RX for {:?} - connection lost", last_rx_time.elapsed());
                return Ok(());
            }
        }
    }

    fn fire_error(
        stopped: &AtomicBool,
        callback: &Arc<dyn TunnelCallbackHandler>,
        error: TunnelError,
    ) {
        if !stopped.swap(true, Ordering::SeqCst) {
            log::error!("Tunnel adapter error: {error}");
            callback.on_error(error);
        }
    }

    /// Mark the tunnel as stopped and fire the timeout callback, exactly once.
    fn fire_timeout(stopped: &AtomicBool, callback: &Arc<dyn TunnelCallbackHandler>) {
        if !stopped.swap(true, Ordering::SeqCst) {
            callback.on_timeout();
        }
    }

    /// Build a WireGuard [`Peer`] from a [`PeerParameters`].
    fn build_peer(peer: &PeerParameters) -> Peer {
        Peer::new(peer.public_key.into())
            .with_allowed_ips(peer.allowed_ips.clone())
            .with_endpoint(peer.endpoint)
    }

    /// Per-packet overhead the entry hop adds to the exit device's MTU budget.
    fn multihop_overhead(entry_endpoint: SocketAddr) -> u16 {
        let overhead = match entry_endpoint.ip() {
            IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
            IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
        };
        overhead as u16
    }

    /// The public key of the ingress device after PQ: the entry key in multihop, the exit key in
    /// singlehop. LWO obfuscates the handshake with this key rather than the device key.
    fn ingress_public_key(pq: &PqResult) -> PublicKey {
        let (pq_entry, pq_exit_key, _) = pq;
        let ingress_key = match pq_entry {
            Some((entry_key, _)) => entry_key,
            None => pq_exit_key,
        };
        PublicKey::from(gotatun::x25519::PublicKey::from(ingress_key).to_bytes())
    }
}

impl Drop for IosTunnelAdapter {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Use `psk` with `peer`, if there is one.
fn with_psk(peer: Peer, psk: Option<&PresharedKey>) -> Peer {
    match psk {
        Some(psk) => peer.with_preshared_key(*psk.as_bytes()),
        None => peer,
    }
}

fn localhost_wg_endpoint(peer: SocketAddr) -> SocketAddr {
    if peer.is_ipv4() {
        SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
    } else {
        SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
    }
}

/// Holds either a single device or an entry+exit pair for multihop.
enum Devices {
    Singlehop(
        gotatun::device::Device<(
            ObfuscatingTransports,
            tun_device::IosTunIpSend,
            tun_device::IosTunIpRecv,
        )>,
    ),
    Multihop {
        entry: gotatun::device::Device<(
            ObfuscatingTransports,
            gotatun::tun::channel::TunChannelTx,
            gotatun::tun::channel::TunChannelRx,
        )>,
        exit: gotatun::device::Device<(
            gotatun::udp::channel::UdpChannelFactory,
            tun_device::IosTunIpSend,
            tun_device::IosTunIpRecv,
        )>,
    },
}

impl Devices {
    async fn suspend(&self) {
        match self {
            Devices::Singlehop(dev) => dev.suspend().await,
            Devices::Multihop { entry, exit } => {
                entry.suspend().await;
                exit.suspend().await;
            }
        }
    }

    async fn wake(&self) {
        _ = match self {
            Devices::Singlehop(dev) => dev.resume().await,
            Devices::Multihop { entry, exit } => {
                _ = entry.resume().await;
                _ = exit.resume().await;
                Ok(())
            }
        };
    }

    async fn stop(self) {
        match self {
            Devices::Singlehop(dev) => dev.stop().await,
            Devices::Multihop { entry, exit } => {
                entry.stop().await;
                exit.stop().await;
            }
        }
    }

    /// Peer stats of the ingress device - the one whose rx reflects tunnel
    /// liveness (the entry device in multihop, the only device in singlehop).
    async fn ingress_peers(&self) -> Vec<gotatun::device::configure::PeerStats> {
        match self {
            Devices::Singlehop(dev) => dev.read(async |d| d.peers().await).await,
            Devices::Multihop { entry, .. } => entry.read(async |d| d.peers().await).await,
        }
    }

    async fn has_rx(&self) -> bool {
        self.ingress_peers()
            .await
            .iter()
            .any(|p| p.stats.rx_bytes > 0)
    }

    async fn total_rx(&self) -> usize {
        self.ingress_peers()
            .await
            .iter()
            .map(|p| p.stats.rx_bytes)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::params::tests::peer;
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    /// Records callback invocations so tests can assert on terminal behaviour.
    #[derive(Default)]
    struct CountingCallback {
        connected: AtomicUsize,
        timeout: AtomicUsize,
        errors: Mutex<Vec<TunnelError>>,
    }

    impl TunnelCallbackHandler for CountingCallback {
        fn on_connected(&self) {
            self.connected.fetch_add(1, Ordering::SeqCst);
        }
        fn on_timeout(&self) {
            self.timeout.fetch_add(1, Ordering::SeqCst);
        }
        fn on_error(&self, error: TunnelError) {
            self.errors.lock().unwrap().push(error);
        }
    }

    fn callback() -> (Arc<CountingCallback>, Arc<dyn TunnelCallbackHandler>) {
        let concrete = Arc::new(CountingCallback::default());
        let dynamic: Arc<dyn TunnelCallbackHandler> = concrete.clone();
        (concrete, dynamic)
    }

    /// A terminal callback fires exactly once and latches the stopped flag; a
    /// second terminal call (of either kind) is a no-op. This idempotency is the
    /// contract every extracted phase in `run` relies on.
    #[test]
    fn terminal_callbacks_fire_once_and_latch_stopped() {
        let (concrete, dynamic) = callback();
        let stopped = AtomicBool::new(false);

        let first_error = TunnelError::TunnelDevice(std::io::Error::other("boom!"));
        IosTunnelAdapter::fire_error(&stopped, &dynamic, first_error);
        assert!(stopped.load(Ordering::SeqCst));
        assert!(matches!(
            concrete.errors.lock().unwrap().as_slice().first().unwrap(),
            TunnelError::TunnelDevice(err) if format!("{}", err) == "boom!"
        ));

        // Already stopped: neither a second error nor a timeout fires.
        let second_error = TunnelError::TunnelDevice(std::io::Error::other("other!"));
        IosTunnelAdapter::fire_error(&stopped, &dynamic, second_error);
        IosTunnelAdapter::fire_timeout(&stopped, &dynamic);
        assert_eq!(concrete.errors.lock().unwrap().len(), 1);
        assert_eq!(concrete.timeout.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn fire_timeout_fires_once() {
        let (concrete, dynamic) = callback();
        let stopped = AtomicBool::new(false);

        IosTunnelAdapter::fire_timeout(&stopped, &dynamic);
        IosTunnelAdapter::fire_timeout(&stopped, &dynamic);
        assert!(stopped.load(Ordering::SeqCst));
        assert_eq!(concrete.timeout.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn multihop_overhead_is_larger_for_ipv6() {
        let v4 = IosTunnelAdapter::multihop_overhead("1.2.3.4:51820".parse().unwrap());
        let v6 = IosTunnelAdapter::multihop_overhead("[2001:db8::1]:51820".parse().unwrap());
        assert!(
            v6 > v4,
            "IPv6 header is larger than IPv4 (v4={v4}, v6={v6})"
        );
        assert_eq!((v6 - v4) as usize, Ipv6Header::LEN - Ipv4Header::LEN);
    }

    #[test]
    fn localhost_wg_endpoint_matches_family() {
        assert_eq!(
            localhost_wg_endpoint("1.2.3.4:51820".parse().unwrap()),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
        );
        assert_eq!(
            localhost_wg_endpoint("[2001:db8::1]:51820".parse().unwrap()),
            SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
        );
    }

    #[test]
    fn ingress_public_key_picks_entry_key_when_multihop() {
        let entry_secret = StaticSecret::from([1u8; 32]);
        let exit_secret = StaticSecret::from([2u8; 32]);
        let entry_pub = gotatun::x25519::PublicKey::from(&entry_secret).to_bytes();
        let exit_pub = gotatun::x25519::PublicKey::from(&exit_secret).to_bytes();

        // Multihop PQ: the entry ephemeral key is used.
        let pq_multihop: PqResult = (
            Some((
                entry_secret,
                IosTunnelAdapter::build_peer(&peer("9.9.9.9:1")),
            )),
            StaticSecret::from([2u8; 32]),
            IosTunnelAdapter::build_peer(&peer("1.2.3.4:1")),
        );
        assert_eq!(
            IosTunnelAdapter::ingress_public_key(&pq_multihop).as_bytes(),
            &entry_pub
        );

        // Singlehop PQ: the (only) exit key is used.
        let pq_singlehop: PqResult = (
            None,
            exit_secret,
            IosTunnelAdapter::build_peer(&peer("1.2.3.4:1")),
        );
        assert_eq!(
            IosTunnelAdapter::ingress_public_key(&pq_singlehop).as_bytes(),
            &exit_pub
        );
    }
}
