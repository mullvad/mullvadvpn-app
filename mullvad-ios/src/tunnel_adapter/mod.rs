#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub(crate) mod ffi;
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
use ipnetwork::IpNetwork;
use talpid_netstack::{
    ip_mux::ip_mux,
    smoltcp_network::{SmoltcpHandle, SmoltcpNetworkConfig, smoltcp_network},
};
use talpid_tunnel_config_client::negotiation::{
    Ingress, IngressTransport, NegotiationConfig, NegotiationError, Relay, Relays,
    negotiate_ephemeral_peers,
};
use talpid_types::{
    ErrorExt,
    net::wireguard::{PresharedKey, PrivateKey, PublicKey},
};
use tokio::sync::Notify;
use tunnel_obfuscation::create_local_socket_obfuscator;

use self::pinger::SmoltcpPinger;
use self::tun_device::IosTunDevice;

/// WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
pub const WIREGUARD_OVERHEAD: u16 = 8 + 32;

/// Guard that aborts the obfuscation proxy task on drop.
struct ObfuscationGuard {
    endpoint: SocketAddr,
    task: tokio::task::JoinHandle<()>,
}

impl ObfuscationGuard {
    fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
}

impl Drop for ObfuscationGuard {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A UDP transport bound ahead of the tunnel starting.
/// Allowing them to bind ahead of time allows for reusing them and also lets the tunnel connection
/// fail fast.
#[derive(Clone)]
pub struct BoundUdpTransports {
    socket: UdpSocket,
}

impl BoundUdpTransports {
    /// Bind the socket, or fail describing why.
    pub async fn bind() -> io::Result<Self> {
        let params = UdpTransportFactoryParams {
            addr: None,
            port: 0,
        };
        let (socket, _recv) = UdpSocketFactory::default().bind(&params).await?;
        Ok(Self { socket })
    }
}

impl UdpTransportFactory for BoundUdpTransports {
    type Send = UdpSocket;
    type Recv = UdpSocket;

    async fn bind(
        &mut self,
        _params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        Ok((self.socket.clone(), self.socket.clone()))
    }
}

/// Reaches the ingress relay using the pre-bound UDP socket, through an obfuscation proxy if
/// obfuscation is enabled.
struct IosIngressTransport<'a> {
    config: &'a mut TunnelConfig,
    udp: BoundUdpTransports,
}

impl IngressTransport for IosIngressTransport<'_> {
    type Factory = BoundUdpTransports;
    type Guard = Option<ObfuscationGuard>;

    async fn connect(
        &mut self,
        client_public_key: &PublicKey,
    ) -> io::Result<Ingress<Self::Factory, Self::Guard>> {
        IosTunnelAdapter::set_lwo_client_public_key(self.config, *client_public_key.as_bytes());
        let obfuscation = IosTunnelAdapter::start_obfuscation_proxy(self.config)
            .await
            .map_err(io::Error::other)?;
        let ingress_peer = self
            .config
            .entry_peer
            .as_ref()
            .unwrap_or(&self.config.exit_peer);
        let endpoint = obfuscation
            .as_ref()
            .map_or(ingress_peer.endpoint, ObfuscationGuard::endpoint);

        Ok(Ingress {
            factory: self.udp.clone(),
            endpoint,
            timer_params: None,
            guard: obfuscation,
        })
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

#[derive(Debug)]
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

impl std::error::Error for ObfuscationProxyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ObfuscationProxyError::InvalidQuicToken(_) => None,
            ObfuscationProxyError::LocalSocketError(error) => Some(error),
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

/// Configuration for a single tunnel connection attempt.
pub struct TunnelConfig {
    pub tun_fd: i32,
    pub private_key: [u8; 32],
    pub ipv4_addr: Ipv4Addr,
    pub ipv6_addr: Ipv6Addr,
    pub mtu: u16,
    pub exit_peer: PeerConfig,
    pub entry_peer: Option<PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub establish_timeout_secs: u32,
    pub enable_pq: bool,
    pub enable_daita: bool,
    pub obfuscation: ObfuscationConfig,
}

impl TunnelConfig {
    /// MTU available to the inner smoltcp stack after WireGuard overhead.
    fn smoltcp_mtu(&self) -> u16 {
        self.mtu.saturating_sub(WIREGUARD_OVERHEAD)
    }

    /// Timeout for establishing connectivity, clamped to at least one second.
    fn establish_timeout(&self) -> Duration {
        Duration::from_secs(self.establish_timeout_secs.max(1) as u64)
    }
}

/// Obfuscation configuration for the tunnel.
#[cfg_attr(test, derive(Debug))]
pub enum ObfuscationConfig {
    Off,
    UdpOverTcp,
    Shadowsocks,
    Quic {
        hostname: String,
        token: String,
    },
    Lwo {
        client_public_key: [u8; 32],
        server_public_key: [u8; 32],
    },
}

pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNetwork>,
}

/// Callbacks from the tunnel adapter to Swift.
pub trait TunnelCallbackHandler: Send + Sync + 'static {
    fn on_connected(&self);
    fn on_timeout(&self);
    fn on_error(&self, error: TunnelError);
}

/// A single tunnel connection attempt.
pub struct IosTunnelAdapter {
    stopped: Arc<AtomicBool>,
    stop_notify: Arc<Notify>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl IosTunnelAdapter {
    pub fn start(
        runtime: tokio::runtime::Handle,
        config: TunnelConfig,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
    ) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let stop_notify = Arc::new(Notify::new());

        let task = runtime.spawn(Self::run(
            config,
            udp,
            callback,
            stopped.clone(),
            stop_notify.clone(),
        ));

        Self {
            stopped,
            stop_notify,
            task_handle: Some(task),
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        self.stop_notify.notify_waiters();
        if let Some(handle) = &self.task_handle {
            handle.abort();
        }
    }

    pub fn recycle_udp_sockets(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("recycle_udp_sockets: not yet implemented");
    }

    pub fn suspend(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("suspend: not yet implemented");
    }

    pub fn wake(&self) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        log::debug!("wake: not yet implemented");
    }

    async fn run(
        config: TunnelConfig,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
        stopped: Arc<AtomicBool>,
        stop_notify: Arc<Notify>,
    ) {
        // Every phase below returns a `Result`; the callback is fired exactly
        // once, here, based on the final outcome.
        match Self::run_inner(config, udp, &callback, &stopped, &stop_notify).await {
            Ok(()) => Self::fire_timeout(&stopped, &callback),
            Err(
                TunnelError::Timeout | TunnelError::NegotiatePQError(NegotiationError::Timeout),
            ) => Self::fire_timeout(&stopped, &callback),
            Err(error) => Self::fire_error(&stopped, &callback, error),
        }
    }

    async fn run_inner(
        mut config: TunnelConfig,
        udp: BoundUdpTransports,
        callback: &Arc<dyn TunnelCallbackHandler>,
        stopped: &AtomicBool,
        stop_notify: &Notify,
    ) -> Result<(), TunnelError> {
        // 1. Create the TUN device from the fd handed over by iOS.
        let tun_dev =
            IosTunDevice::new(config.tun_fd, config.mtu).map_err(TunnelError::TunnelDevice)?;

        // 2. Negotiate the PQ/DAITA ephemeral peer(s) over a smoltcp-only device,
        //    or fall back to the static device peer.
        let pq = Self::negotiate_pq(&mut config, &udp)
            .await
            .map_err(TunnelError::NegotiatePQError)?;
        if stopped.load(Ordering::SeqCst) {
            // Cancelled externally; the outcome below is discarded since `run`
            // no-ops when it sees the tunnel is already stopped.
            return Err(TunnelError::Timeout);
        }

        // 3. After PQ the WireGuard handshake uses the ephemeral ingress key, so
        //    point LWO at it, then start the obfuscation proxy for the real device.
        Self::apply_lwo_ingress_key(&mut config, &pq);
        let final_obfuscation = Self::start_obfuscation_proxy(&config)
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;
        if let Some(ref guard) = final_obfuscation {
            Self::apply_obfuscation(&mut config, guard.endpoint());
        }

        // 4. Build the user-traffic device(s) behind an IpMuxRecv and IpMuxSend (TUN + smoltcp).
        let (smoltcp_handle, ip_recv, ip_send, _smoltcp_guard) =
            smoltcp_network(SmoltcpNetworkConfig {
                ipv4_addr: config.ipv4_addr,
                ipv6_addr: Some(config.ipv6_addr),
                mtu: config.smoltcp_mtu(),
            });
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv, ip_send);

        let devices = Self::build_devices(&config, &udp, pq, mux_recv, mux_send).await?;
        if stopped.load(Ordering::SeqCst) {
            devices.stop().await;
            return Err(TunnelError::Timeout);
        }

        // 5. Establish connectivity, then monitor it until it drops or we stop.
        let connected = match Self::establish_connectivity(
            &devices,
            &smoltcp_handle,
            &config,
            stopped,
            stop_notify,
        )
        .await
        {
            Ok(connected) => connected,
            Err(e) => {
                devices.stop().await;
                return Err(e);
            }
        };
        if !connected {
            devices.stop().await;
            return Err(TunnelError::Timeout);
        }

        callback.on_connected();
        log::info!("Tunnel connected - starting ongoing monitoring");
        Self::monitor_connectivity(&devices, stopped, stop_notify).await;
        devices.stop().await;
        Err(TunnelError::Timeout)
    }

    /// Negotiate the post-quantum / DAITA ephemeral peer(s).
    ///
    /// Returns the `(entry, exit_key, exit_peer)` triple to configure the final
    /// device(s) with - `entry` is `Some` only for multihop PQ.
    async fn negotiate_pq(
        config: &mut TunnelConfig,
        udp: &BoundUdpTransports,
    ) -> Result<PqResult, NegotiationError> {
        // No PQ/DAITA: the device peer is just the static configured exit peer.
        if !(config.enable_pq || config.enable_daita) {
            let private_key = StaticSecret::from(config.private_key);
            return Ok((None, private_key, Self::build_peer(&config.exit_peer)));
        }

        let relay = |peer: &PeerConfig| Relay {
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
        let negotiation_config = NegotiationConfig {
            private_key: PrivateKey::from(config.private_key),
            tunnel_ipv4: config.ipv4_addr,
            tunnel_ipv6: Some(config.ipv6_addr),
            config_service_ip: config.ipv4_gateway,
            relays,
            enable_post_quantum: config.enable_pq,
            enable_daita: config.enable_daita,
            timeout: config.establish_timeout(),
            // Each relay has a device of its own, so it can have a key of its own.
            separate_exit_key: true,
        };

        let mut transport = IosIngressTransport {
            config: &mut *config,
            udp: udp.clone(),
        };
        let negotiated = negotiate_ephemeral_peers(&negotiation_config, &mut transport).await?;

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
        config: &TunnelConfig,
        udp: &BoundUdpTransports,
        pq: PqResult,
        mux_recv: tun_device::IosTunIpRecv,
        mux_send: tun_device::IosTunIpSend,
    ) -> Result<Devices, TunnelError> {
        let (pq_entry, pq_exit_key, pq_exit_peer) = pq;

        let Some(entry_peer_config) = config.entry_peer.as_ref() else {
            // Singlehop: one device, mux'd IP pair, real UDP.
            let device = DeviceBuilder::new()
                .with_udp(udp.clone())
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

        // Use the PQ entry key if negotiated, otherwise the device key. The
        // endpoint must match the (possibly obfuscated) config endpoint.
        let (entry_key, entry_peer) = pq_entry.unwrap_or_else(|| {
            (
                StaticSecret::from(config.private_key),
                Self::build_peer(entry_peer_config),
            )
        });
        let entry_peer = entry_peer.with_endpoint(entry_peer_config.endpoint);

        let entry_device = match DeviceBuilder::new()
            .with_udp(udp.clone())
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
        devices: &Devices,
        smoltcp_handle: &SmoltcpHandle,
        config: &TunnelConfig,
        stopped: &AtomicBool,
        stop_notify: &Notify,
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

        Ok(tokio::select! {
            result = Self::wait_for_connectivity(devices, &mut pinger, stopped) => result,
            _ = tokio::time::sleep(establish_timeout) => false,
            _ = stop_notify.notified() => false,
        })
    }

    /// Start a fresh obfuscation proxy for the given config.
    /// Returns a guard that keeps the proxy alive until dropped.
    /// Returns `None` if obfuscation is off.
    async fn start_obfuscation_proxy(
        config: &TunnelConfig,
    ) -> Result<Option<ObfuscationGuard>, ObfuscationProxyError> {
        let ingress_endpoint = config
            .entry_peer
            .as_ref()
            .unwrap_or(&config.exit_peer)
            .endpoint;

        let settings = match &config.obfuscation {
            ObfuscationConfig::Off => return Ok(None),
            ObfuscationConfig::UdpOverTcp => {
                tunnel_obfuscation::Settings::Udp2Tcp(tunnel_obfuscation::udp2tcp::Settings {
                    peer: ingress_endpoint,
                })
            }
            ObfuscationConfig::Shadowsocks => {
                let wg_ep = localhost_wg_endpoint(ingress_endpoint);
                tunnel_obfuscation::Settings::Shadowsocks(
                    tunnel_obfuscation::shadowsocks::Settings {
                        shadowsocks_endpoint: ingress_endpoint,
                        wireguard_endpoint: wg_ep,
                    },
                )
            }
            ObfuscationConfig::Quic { hostname, token } => {
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
            ObfuscationConfig::Lwo {
                client_public_key,
                server_public_key,
            } => tunnel_obfuscation::Settings::Lwo(tunnel_obfuscation::lwo::Settings {
                server_addr: ingress_endpoint,
                client_public_key: talpid_types::net::wireguard::PublicKey::from(
                    *client_public_key,
                ),
                server_public_key: talpid_types::net::wireguard::PublicKey::from(
                    *server_public_key,
                ),
                version: talpid_types::net::obfuscation::LwoVersion::V1,
            }),
        };

        let obfuscator = create_local_socket_obfuscator(&settings)
            .await
            .map_err(ObfuscationProxyError::LocalSocketError)?;
        let endpoint = obfuscator.endpoint();
        log::info!("Obfuscation proxy started at {endpoint}");
        let task = tokio::spawn(async move {
            let _ = obfuscator.run().await;
        });
        Ok(Some(ObfuscationGuard { endpoint, task }))
    }

    /// Apply obfuscation to the config: replace the ingress peer's endpoint with the proxy address.
    fn apply_obfuscation(config: &mut TunnelConfig, proxy_endpoint: SocketAddr) {
        if let Some(ref mut entry) = config.entry_peer {
            entry.endpoint = proxy_endpoint;
        } else {
            config.exit_peer.endpoint = proxy_endpoint;
        }
    }

    /// Wait for the device to receive traffic (rx_bytes > 0 on any peer).
    async fn wait_for_connectivity(
        devices: &Devices,
        pinger: &mut SmoltcpPinger,
        stopped: &AtomicBool,
    ) -> bool {
        let mut last_ping = Instant::now();

        loop {
            if stopped.load(Ordering::SeqCst) {
                return false;
            }

            if devices.has_rx().await {
                log::debug!("Connectivity established - rx_bytes > 0");
                return true;
            }

            if last_ping.elapsed() >= PING_INTERVAL {
                if let Err(e) = pinger.send_icmp().await {
                    log::warn!("Ping failed: {e}");
                }
                last_ping = Instant::now();
            }

            tokio::time::sleep(CONNECTIVITY_CHECK_INTERVAL).await;
        }
    }

    /// Monitor an established connection. Returns when connectivity is lost or stopped.
    async fn monitor_connectivity(devices: &Devices, stopped: &AtomicBool, stop_notify: &Notify) {
        let mut last_rx_bytes: usize = 0;
        let mut last_rx_time = Instant::now();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                _ = stop_notify.notified() => return,
            }

            if stopped.load(Ordering::SeqCst) {
                return;
            }

            let total_rx = devices.total_rx().await;

            if total_rx > last_rx_bytes {
                last_rx_bytes = total_rx;
                last_rx_time = Instant::now();
            } else if last_rx_time.elapsed() > MONITOR_TIMEOUT {
                log::warn!("No RX for {:?} - connection lost", last_rx_time.elapsed());
                return;
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

    /// Build a WireGuard [`Peer`] from a [`PeerConfig`].
    fn build_peer(peer: &PeerConfig) -> Peer {
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

    /// After PQ, LWO obfuscates the handshake with the ingress device's ephemeral
    /// key (the entry key in multihop, the exit key in singlehop) rather than the
    /// device key. No-op unless obfuscation is LWO.
    fn apply_lwo_ingress_key(config: &mut TunnelConfig, pq: &PqResult) {
        let (pq_entry, pq_exit_key, _) = pq;
        let ingress_key = match pq_entry {
            Some((entry_key, _)) => entry_key,
            None => pq_exit_key,
        };
        Self::set_lwo_client_public_key(
            config,
            gotatun::x25519::PublicKey::from(ingress_key).to_bytes(),
        );
    }

    /// Make LWO obfuscate for a device that uses `client_public_key`. No-op unless obfuscation is
    /// LWO.
    fn set_lwo_client_public_key(config: &mut TunnelConfig, client_public_key: [u8; 32]) {
        if let ObfuscationConfig::Lwo {
            client_public_key: lwo_client_public_key,
            ..
        } = &mut config.obfuscation
        {
            *lwo_client_public_key = client_public_key;
        }
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
            BoundUdpTransports,
            tun_device::IosTunIpSend,
            tun_device::IosTunIpRecv,
        )>,
    ),
    Multihop {
        entry: gotatun::device::Device<(
            BoundUdpTransports,
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
    use super::*;
    use std::assert_matches;
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

    fn peer(endpoint: &str) -> PeerConfig {
        PeerConfig {
            public_key: [7u8; 32],
            endpoint: endpoint.parse().unwrap(),
            allowed_ips: vec!["0.0.0.0/0".parse().unwrap()],
        }
    }

    fn config() -> TunnelConfig {
        TunnelConfig {
            tun_fd: -1,
            private_key: [0u8; 32],
            ipv4_addr: Ipv4Addr::new(10, 0, 0, 2),
            ipv6_addr: "fd00::2".parse().unwrap(),
            mtu: 1280,
            exit_peer: peer("1.2.3.4:51820"),
            entry_peer: None,
            ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
            establish_timeout_secs: 4,
            enable_pq: false,
            enable_daita: false,
            obfuscation: ObfuscationConfig::Off,
        }
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
    fn establish_timeout_clamps_to_at_least_one_second() {
        let mut c = config();
        c.establish_timeout_secs = 0;
        assert_eq!(c.establish_timeout(), Duration::from_secs(1));
        c.establish_timeout_secs = 7;
        assert_eq!(c.establish_timeout(), Duration::from_secs(7));
    }

    #[test]
    fn smoltcp_mtu_subtracts_wireguard_overhead_and_saturates() {
        let mut c = config();
        c.mtu = 1280;
        assert_eq!(c.smoltcp_mtu(), 1280 - WIREGUARD_OVERHEAD);
        c.mtu = 10; // smaller than the overhead
        assert_eq!(c.smoltcp_mtu(), 0);
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
    fn apply_obfuscation_targets_entry_in_multihop_else_exit() {
        let proxy: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        // Singlehop: rewrites the exit endpoint.
        let mut singlehop = config();
        IosTunnelAdapter::apply_obfuscation(&mut singlehop, proxy);
        assert_eq!(singlehop.exit_peer.endpoint, proxy);

        // Multihop: rewrites the entry endpoint, leaves the exit untouched.
        let mut multihop = config();
        multihop.entry_peer = Some(peer("9.9.9.9:51820"));
        let exit_endpoint = multihop.exit_peer.endpoint;
        IosTunnelAdapter::apply_obfuscation(&mut multihop, proxy);
        assert_eq!(multihop.entry_peer.as_ref().unwrap().endpoint, proxy);
        assert_eq!(multihop.exit_peer.endpoint, exit_endpoint);
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
    fn apply_lwo_ingress_key_picks_entry_key_when_multihop() {
        let entry_secret = StaticSecret::from([1u8; 32]);
        let exit_secret = StaticSecret::from([2u8; 32]);
        let entry_pub = gotatun::x25519::PublicKey::from(&entry_secret).to_bytes();
        let exit_pub = gotatun::x25519::PublicKey::from(&exit_secret).to_bytes();

        let lwo = || ObfuscationConfig::Lwo {
            client_public_key: [0u8; 32],
            server_public_key: [9u8; 32],
        };
        let current = |c: &TunnelConfig| match c.obfuscation {
            ObfuscationConfig::Lwo {
                client_public_key, ..
            } => client_public_key,
            _ => unreachable!(),
        };

        // Multihop PQ: the entry ephemeral key is used.
        let mut multihop = config();
        multihop.obfuscation = lwo();
        let pq_multihop: PqResult = (
            Some((
                entry_secret,
                IosTunnelAdapter::build_peer(&peer("9.9.9.9:1")),
            )),
            StaticSecret::from([2u8; 32]),
            IosTunnelAdapter::build_peer(&peer("1.2.3.4:1")),
        );
        IosTunnelAdapter::apply_lwo_ingress_key(&mut multihop, &pq_multihop);
        assert_eq!(current(&multihop), entry_pub);

        // Singlehop PQ: the (only) exit key is used.
        let mut singlehop = config();
        singlehop.obfuscation = lwo();
        let pq_singlehop: PqResult = (
            None,
            exit_secret,
            IosTunnelAdapter::build_peer(&peer("1.2.3.4:1")),
        );
        IosTunnelAdapter::apply_lwo_ingress_key(&mut singlehop, &pq_singlehop);
        assert_eq!(current(&singlehop), exit_pub);

        // Non-LWO obfuscation is left untouched.
        let mut off = config();
        IosTunnelAdapter::apply_lwo_ingress_key(&mut off, &pq_singlehop);
        assert_matches!(off.obfuscation, ObfuscationConfig::Off);
    }
}
