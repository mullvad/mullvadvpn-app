#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub(crate) mod ffi;
mod monitor;
mod obfuscation;
pub(crate) mod params;
mod pinger;
pub(crate) mod tun_device;

use std::{
    io,
    net::{IpAddr, SocketAddr},
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
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
use talpid_error::ErrorExt;
use talpid_netstack::{
    ip_mux::ip_mux,
    smoltcp_network::{SmoltcpHandle, SmoltcpNetworkGuard, smoltcp_network},
};
use talpid_tunnel_config_client::negotiation::{
    Negotiables, NegotiatedPeers, NegotiationConfig, NegotiationError, Relay, Relays,
    negotiate_ephemeral_peers,
};
use talpid_types::net::wireguard::{PrivateKey, PublicKey};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use tunnel_obfuscation::gotatun_transport::{MaybeObfuscatingTransportFactory, RunningObfuscation};

use self::monitor::{ConnectivityMonitor, TunnelStats, TunnelStatus};
use self::obfuscation::{ObfuscatingTransports, create_obfuscation};
use self::pinger::SmoltcpPinger;
use self::tun_device::IosTunDevice;

pub use self::obfuscation::ObfuscationProxyError;
pub use self::params::{PeerParameters, TunnelParameters};

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

/// Error from a phase of [`IosTunnelAdapter::run`].
pub enum TunnelError {
    RebindUdpSocket(io::Error),
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
            TunnelError::RebindUdpSocket(msg) => write!(f, "Failed to rebind UDP socket: {msg}"),
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

/// All state of a connected tunnel, kept so that it can be suspended, woken, and moved onto fresh
/// sockets while it runs.
struct ActiveConnection {
    devices: Devices,
    transport_provider: BoundUdpTransports,
    /// When the tunnel was last suspended, to decide whether to restart the obfuscator on wake.
    last_suspended_at: Option<talpid_time::Instant>,
    obfuscation: ObfuscatingTransports,
    params: TunnelParameters,
    /// Key the ingress device handshakes with, which LWO obfuscates for.
    ingress_public_key: PublicKey,
    pinger: SmoltcpPinger,
    user_tx: Arc<AtomicU64>,
    /// The smoltcp stack the pinger sends through.
    _smoltcp: SmoltcpNetworkGuard,
    /// `None` while suspended.
    monitor: Option<ConnectivityMonitor>,
}

impl ActiveConnection {
    /// Bumps sockets for obfuscators that need their sockets recycled to survive sleep and path
    /// updates.
    async fn bump_sockets(&mut self) -> Result<(), TunnelError> {
        self.devices.suspend().await;
        self.transport_provider
            .rebind()
            .await
            .map_err(TunnelError::RebindUdpSocket)?;
        self.restart_obfuscation().await?;
        // Only wake the tunnel up if it is was up originally.
        if !self.is_suspended() {
            self.devices.wake().await;
        }
        Ok(())
    }

    /// The monitor only exists while the tunnel is awake.
    fn is_suspended(&self) -> bool {
        self.monitor.is_none()
    }

    /// Replace the obfuscation proxy and point the ingress device at the new one.
    /// No-op when obfuscation need not rebind sockets.
    async fn restart_obfuscation(&mut self) -> Result<(), TunnelError> {
        self.obfuscation.replace(None).await;
        let obfuscation = create_obfuscation(&self.params)
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;
        self.obfuscation
            .replace(obfuscation.map(|obfuscation| {
                obfuscation.with_client_public_key(self.ingress_public_key.clone())
            }))
            .await;
        Ok(())
    }

    async fn stats(&self) -> TunnelStats {
        TunnelStats {
            user_tx: self.user_tx.load(Ordering::Relaxed),
            rx_bytes: self.devices.rx_bytes().await,
        }
    }

    /// Establish connectivity, then monitor it until it is lost or the adapter stops. Returns
    /// `Ok` when stopped, [`TunnelError::Timeout`] when the connection times out, and other errors
    /// otherwise. Tunnel monitoring does not take place when the tunnel is suspended, awaking a
    /// tunnel will reset the tunnel monitor.
    async fn run(
        &mut self,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
        callback: &dyn TunnelCallbackHandler,
    ) -> Result<(), TunnelError> {
        loop {
            self.drive_connectivity_monitor(callback).await?;

            tokio::select! {
                _ = self.wait_for_connectivity_monitor() => {}
                command = rx.recv() => {
                    let Some(command) = command else { return Ok(()) };

                    let should_shutdown = self.handle_command(command).await?;
                    if should_shutdown {
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Samples the tunnel traffic stats and evaluates tunnel connection.
    async fn drive_connectivity_monitor(
        &mut self,
        callback: &dyn TunnelCallbackHandler,
    ) -> Result<(), TunnelError> {
        let now = Instant::now();
        let stats = self.stats().await;
        let Some(monitor) = &mut self.monitor else {
            return Ok(());
        };
        match monitor.evaluate_connectivity(now, stats) {
            TunnelStatus::Healthy => {}
            TunnelStatus::RequiresProbing(count) => {
                log::trace!("Sending {count} ping(s)");
                for _ in 0..count {
                    if let Err(e) = self.pinger.send_icmp().await {
                        log::warn!("Ping failed: {e}");
                    }
                }
            }
            TunnelStatus::Connected => {
                callback.on_connected();
            }
            TunnelStatus::TimedOut => return Err(TunnelError::Timeout),
        }
        Ok(())
    }

    /// Sleeps until the connectivity monitor needs another sample.
    async fn wait_for_connectivity_monitor(&self) {
        match &self.monitor {
            Some(monitor) => tokio::time::sleep(monitor.interval()).await,
            None => std::future::pending().await,
        }
    }

    /// Apply an adapter command. Returns true if connection should stop.
    async fn handle_command(
        &mut self,
        command: TunnelAdapterChannelCommand,
    ) -> Result<bool, TunnelError> {
        match command {
            TunnelAdapterChannelCommand::Suspend => {
                self.last_suspended_at = Some(talpid_time::Instant::now());
                self.devices.suspend().await;
                self.monitor = None;
            }
            TunnelAdapterChannelCommand::Wake => {
                let Some(last_suspended_at) = self.last_suspended_at.take() else {
                    log::error!("Wake without a Suspend before it - nothing to do");
                    return Ok(false);
                };
                let elapsed = talpid_time::Instant::now().duration_since(last_suspended_at);
                if elapsed >= SLEEP_CYCLE_RESET_THRESHOLD {
                    log::info!("Restarting obfuscation after {elapsed:?} suspended");
                    self.restart_obfuscation().await?;
                }
                self.devices.wake().await;

                let now = Instant::now();
                let stats = self.stats().await;
                log::debug!(
                    "Woke after {elapsed:?}; tx {} packets, rx {} bytes",
                    stats.user_tx,
                    stats.rx_bytes
                );
                self.monitor = Some(ConnectivityMonitor::new(now, stats, now));
            }
            TunnelAdapterChannelCommand::BumpSockets => {
                self.bump_sockets().await?;
                if let Some(monitor) = &mut self.monitor {
                    monitor.reset();
                }
            }
            TunnelAdapterChannelCommand::Stop => return Ok(true),
        }
        Ok(false)
    }
}

/// A single tunnel connection attempt.
pub struct IosTunnelAdapter {
    stopped: Arc<AtomicBool>,
    tx: UnboundedSender<TunnelAdapterChannelCommand>,
    runtime: tokio::runtime::Handle,
    task_handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl IosTunnelAdapter {
    pub fn start(
        runtime: tokio::runtime::Handle,
        params: TunnelParameters,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
    ) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let task = runtime.spawn(Self::run(params, udp, callback, rx, stopped.clone()));

        Self {
            stopped,
            tx,
            runtime,
            task_handle: std::sync::Mutex::new(Some(task)),
        }
    }

    /// Stop the tunnel and wait for its task to finish. Not to be called from the runtime.
    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        log::debug!("Stopping device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Stop);
        if let Some(task) = self.task_handle.lock().unwrap().take() {
            _ = self.runtime.block_on(task);
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
        log::debug!("Awaking device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Wake);
    }

    async fn run(
        params: TunnelParameters,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
        mut rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: Arc<AtomicBool>,
    ) {
        // Every phase below returns a `Result`; the callback is fired exactly
        // once, here, based on the final outcome.
        match Self::run_inner(params, udp, &*callback, &mut rx).await {
            Ok(()) => Self::fire_timeout(&stopped, &callback),
            Err(
                TunnelError::Timeout | TunnelError::NegotiatePQError(NegotiationError::Timeout),
            ) => Self::fire_timeout(&stopped, &callback),
            Err(error) => Self::fire_error(&stopped, &callback, error),
        }
    }

    async fn run_inner(
        params: TunnelParameters,
        udp: BoundUdpTransports,
        callback: &dyn TunnelCallbackHandler,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
    ) -> Result<(), TunnelError> {
        let Some(mut connection) = Self::connect(params, udp, rx).await? else {
            return Ok(());
        };
        let result = connection.run(rx, callback).await;
        connection.devices.stop().await;
        result
    }

    /// Build the tunnel: TUN device, obfuscation, peers, GotaTun device(s) and pinger.
    /// Returns `None` if the adapter was stopped meanwhile.
    async fn connect(
        params: TunnelParameters,
        udp: BoundUdpTransports,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
    ) -> Result<Option<ActiveConnection>, TunnelError> {
        // 1. Create the TUN device from the fd handed over by iOS.
        let tun_dev =
            IosTunDevice::new(params.tun_fd, params.mtu).map_err(TunnelError::TunnelDevice)?;
        let user_tx = tun_dev.tx_packets();
        let obfuscation = create_obfuscation(&params)
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;

        // 2. Negotiate the PQ/DAITA ephemeral peer(s) over a smoltcp-only device,
        //    or fall back to the static device peer(s).
        let negotiated = {
            let mut negotiate_pq = pin!(Self::negotiate_pq(&params, &udp, obfuscation.clone()));
            loop {
                tokio::select! {
                    negotiated = &mut negotiate_pq => break negotiated?,
                    command = rx.recv() => match command {
                        Some(TunnelAdapterChannelCommand::Stop) | None => {
                            // NOTE: PQ devices are stopped when dropped
                            return Ok(None);
                        }
                        Some(_) => {}
                    },
                }
            }
        };

        // 3. After PQ the WireGuard handshake uses the ephemeral ingress key, so point LWO at it.
        let ingress_key = negotiated.as_ref().map_or_else(
            || StaticSecret::from(params.private_key),
            |negotiated| StaticSecret::from(negotiated.private_key.to_bytes()),
        );
        let ingress_public_key =
            PublicKey::from(gotatun::x25519::PublicKey::from(&ingress_key).to_bytes());
        let obfuscation = ObfuscatingTransports::new(
            udp.clone(),
            obfuscation
                .map(|obfuscation| obfuscation.with_client_public_key(ingress_public_key.clone())),
            params.ingress_peer().endpoint,
        );

        // 4. Build the user-traffic device(s) behind an IpMuxRecv and IpMuxSend (TUN + smoltcp).
        let (smoltcp_handle, ip_recv, ip_send, smoltcp_guard) =
            smoltcp_network(params.smoltcp_network_config());
        let pinger = Self::create_pinger(&smoltcp_handle, params.ipv4_gateway).await?;
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv, ip_send);
        let devices = Self::build_devices(
            &params,
            &obfuscation,
            negotiated.as_ref(),
            ingress_key,
            mux_recv,
            mux_send,
        )
        .await?;

        let establish_timeout = params.establish_timeout();
        log::info!("Establishing connectivity (timeout: {establish_timeout:?})");
        let now = Instant::now();
        Ok(Some(ActiveConnection {
            devices,
            transport_provider: udp,
            last_suspended_at: None,
            obfuscation,
            params,
            ingress_public_key,
            pinger,
            user_tx,
            _smoltcp: smoltcp_guard,
            monitor: Some(ConnectivityMonitor::new(
                now,
                TunnelStats::default(),
                now + establish_timeout,
            )),
        }))
    }

    /// Negotiate the post-quantum / DAITA ephemeral peer(s), if either is enabled.
    async fn negotiate_pq(
        params: &TunnelParameters,
        udp: &BoundUdpTransports,
        obfuscation: Option<RunningObfuscation>,
    ) -> Result<Option<NegotiatedPeers>, TunnelError> {
        if !(params.enable_pq || params.enable_daita) {
            return Ok(None);
        }

        let relay = |peer: &PeerParameters| Relay {
            public_key: PublicKey::from(peer.public_key),
            endpoint: peer.endpoint,
        };
        let relays = match &params.entry_peer {
            None => Relays::Singlehop(relay(&params.exit_peer)),
            Some(entry_peer) => Relays::Multihop {
                entry: relay(entry_peer),
                exit: relay(&params.exit_peer),
            },
        };
        let negotiate = Negotiables {
            post_quantum: params.enable_pq,
            daita: params.enable_daita,
        };
        let negotiation_config = NegotiationConfig {
            private_key: PrivateKey::from(params.private_key),
            tunnel_ipv4: params.ipv4_addr,
            config_service_ip: params.ipv4_gateway,
            relays,
            // iOS only uses LWO v1, which keeps the default timers.
            ingress_timer_params: None,
            timeout: params.establish_timeout(),
            handshake_timeout: params.establish_timeout(),
            tcp_timeout: Some(params.establish_timeout()),
            // Each relay has a device of its own, so it can have a key of its own.
            separate_exit_key: true,
        };

        let ingress_endpoint = params.ingress_peer().endpoint;
        let ingress_transport = |client_public_key: &PublicKey| {
            let obfuscation = obfuscation
                .clone()
                .map(|obfuscation| obfuscation.with_client_public_key(client_public_key.clone()));
            MaybeObfuscatingTransportFactory::new(udp.clone(), obfuscation, ingress_endpoint)
        };
        negotiate_ephemeral_peers(&negotiation_config, negotiate, ingress_transport)
            .await
            .map(Some)
            .map_err(TunnelError::NegotiatePQError)
    }

    /// Build the GotaTun device(s) carrying user traffic, with the keys and PSKs from
    /// `negotiated` if any.
    async fn build_devices(
        params: &TunnelParameters,
        obfuscation: &ObfuscatingTransports,
        negotiated: Option<&NegotiatedPeers>,
        ingress_key: StaticSecret,
        mux_recv: tun_device::IosTunIpRecv,
        mux_send: tun_device::IosTunIpSend,
    ) -> Result<Devices, TunnelError> {
        let ingress_peer = |peer: &PeerParameters| {
            let mut peer = Self::build_peer(peer);
            if let Some(psk) = negotiated.and_then(|negotiated| negotiated.ingress_psk.as_ref()) {
                peer = peer.with_preshared_key(*psk.as_bytes());
            }
            if let Some(daita) = negotiated.and_then(|negotiated| negotiated.daita.as_ref()) {
                peer = peer.with_daita(daita.into());
            }
            peer
        };

        let Some(entry_peer_config) = params.entry_peer.as_ref() else {
            // Singlehop: one device, mux'd IP pair, obfuscated UDP.
            let device = DeviceBuilder::new()
                .with_udp(obfuscation.clone())
                .with_ip_pair(mux_send, mux_recv)
                .with_private_key(ingress_key)
                .with_peer(ingress_peer(&params.exit_peer))
                .build()
                .await
                .map_err(TunnelError::GotaTunDeviceError)?;
            return Ok(Devices::Singlehop(device));
        };

        // Multihop: exit device tunnels its UDP through the entry device.
        let entry_mtu = MtuWatcher::new(params.mtu)
            .increase(Self::multihop_overhead(entry_peer_config.endpoint))
            .expect("MTU overflow");
        let (tun_channel_tx, tun_channel_rx, udp_channels) =
            new_udp_tun_channel(100, params.ipv4_addr, params.ipv6_addr, entry_mtu);

        // The exit's own key if negotiated, otherwise the ingress key.
        let exit_key = negotiated
            .and_then(|negotiated| negotiated.exit_private_key.as_ref())
            .map_or_else(
                || ingress_key.clone(),
                |key| StaticSecret::from(key.to_bytes()),
            );
        let mut exit_peer = Self::build_peer(&params.exit_peer);
        if let Some(psk) = negotiated.and_then(|negotiated| negotiated.exit_psk.as_ref()) {
            exit_peer = exit_peer.with_preshared_key(*psk.as_bytes());
        }
        let exit_device = DeviceBuilder::new()
            .with_udp(udp_channels)
            .with_ip_pair(mux_send, mux_recv)
            .with_private_key(exit_key)
            .with_peer(exit_peer)
            .build()
            .await
            .map_err(TunnelError::MultihopExitDeviceError)?;

        log::info!(
            "Multihop: entry={}, exit={}",
            entry_peer_config.endpoint,
            params.exit_peer.endpoint
        );

        let entry_device = match DeviceBuilder::new()
            .with_udp(obfuscation.clone())
            .with_ip_pair(tun_channel_tx, tun_channel_rx)
            .with_peer(ingress_peer(entry_peer_config))
            .with_private_key(ingress_key)
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

    async fn create_pinger(
        smoltcp_handle: &SmoltcpHandle,
        gateway: std::net::Ipv4Addr,
    ) -> Result<SmoltcpPinger, TunnelError> {
        // Bind the socket to the pinger's ident so echo replies reach it.
        let ping_ident: u16 = rand::random();
        let icmp_socket = smoltcp_handle
            .icmp_socket(ping_ident)
            .await
            .map_err(TunnelError::ICMPSocketError)?;
        Ok(SmoltcpPinger::new(icmp_socket, gateway, ping_ident))
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
}

impl Drop for IosTunnelAdapter {
    fn drop(&mut self) {
        self.stop();
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
    async fn stop(self) {
        match self {
            Devices::Singlehop(dev) => dev.stop().await,
            Devices::Multihop { entry, exit } => {
                entry.stop().await;
                exit.stop().await;
            }
        }
    }

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

    /// Bytes received by the innermost device - the exit device in multihop, the only device in
    /// singlehop - so that a dead exit relay is not masked by a live entry relay.
    async fn rx_bytes(&self) -> usize {
        let peers = match self {
            Devices::Singlehop(dev) => dev.read(async |d| d.peers().await).await,
            Devices::Multihop { exit, .. } => exit.read(async |d| d.peers().await).await,
        };
        peers.iter().map(|p| p.stats.rx_bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsRawFd;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

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

    /// Start a real adapter on a socket pair standing in for the TUN device, with a loopback
    /// relay that never answers.
    fn start_adapter(
        runtime: &tokio::runtime::Runtime,
        tun: &std::os::unix::net::UnixStream,
        enable_pq: bool,
    ) -> (IosTunnelAdapter, Arc<CountingCallback>) {
        let mut params = super::params::tests::params();
        params.tun_fd = tun.as_raw_fd();
        params.exit_peer.endpoint = "127.0.0.1:1".parse().unwrap();
        params.enable_pq = enable_pq;
        let udp = runtime.block_on(BoundUdpTransports::bind()).unwrap();
        let (concrete, dynamic) = callback();
        let adapter = IosTunnelAdapter::start(runtime.handle().clone(), params, udp, dynamic);
        (adapter, concrete)
    }

    #[test]
    fn stop_waits_for_the_task_and_fires_no_callback() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (tun, _peer) = std::os::unix::net::UnixStream::pair().unwrap();

        // Stopped at once, while establishing, and during PQ negotiation.
        for (delay, enable_pq) in [
            (Duration::ZERO, false),
            (Duration::from_millis(300), false),
            (Duration::from_millis(300), true),
        ] {
            let (adapter, callback) = start_adapter(&runtime, &tun, enable_pq);
            std::thread::sleep(delay);
            adapter.stop();

            assert!(adapter.task_handle.lock().unwrap().is_none());
            assert_eq!(callback.connected.load(Ordering::SeqCst), 0);
            assert_eq!(callback.timeout.load(Ordering::SeqCst), 0);
            assert!(
                callback.errors.lock().unwrap().is_empty(),
                "{:?}",
                callback.errors.lock().unwrap().len()
            );
        }
    }

    /// Bumping the sockets of an awake tunnel must bring its device back up on the new socket,
    /// rather than leave it down until the next Wake.
    #[test]
    fn bump_sockets_while_awake_keeps_the_device_up() {
        /// First byte of a WireGuard handshake initiation.
        const HANDSHAKE_INITIATION: u8 = 1;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (tun, _peer) = std::os::unix::net::UnixStream::pair().unwrap();
        let relay = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();

        // Source of the next handshake initiation not sent from `old_source`, if any.
        let next_initiation = |old_source: Option<std::net::SocketAddr>| {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut buf = [0u8; 2048];
            while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
                relay.set_read_timeout(Some(left)).unwrap();
                let Ok((len, source)) = relay.recv_from(&mut buf) else {
                    break;
                };
                if len > 0 && buf[0] == HANDSHAKE_INITIATION && Some(source) != old_source {
                    return Some(source);
                }
            }
            None
        };

        let mut params = super::params::tests::params();
        params.tun_fd = tun.as_raw_fd();
        params.exit_peer.endpoint = relay.local_addr().unwrap();
        // Outlast the test, so that the adapter only stops when told to.
        params.establish_timeout_secs = 30;
        let udp = runtime.block_on(BoundUdpTransports::bind()).unwrap();
        let (_, dynamic) = callback();
        let adapter = IosTunnelAdapter::start(runtime.handle().clone(), params, udp, dynamic);

        let old_source = next_initiation(None).expect("no handshake before the bump");
        adapter.recycle_udp_sockets();
        let new_source = next_initiation(Some(old_source));
        adapter.stop();

        assert!(new_source.is_some(), "no handshake after the bump");
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
}
