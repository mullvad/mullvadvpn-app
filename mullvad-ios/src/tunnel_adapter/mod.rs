#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub(crate) mod ffi;
mod obfuscation;
mod params;
mod pinger;
mod pq;
pub(crate) mod tun_device;

use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::gotatun::{
    ip_mux::ip_mux,
    smoltcp_network::{SmoltcpHandle, smoltcp_network},
};
use gotatun::{
    device::{DeviceBuilder, Peer},
    udp::{
        UdpTransportFactory, UdpTransportFactoryParams,
        channel::new_udp_tun_channel,
        socket::{UdpSocket, UdpSocketFactory},
    },
    x25519::StaticSecret,
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use self::obfuscation::{ObfuscationProxy, ObfuscationSlot};
use self::pinger::SmoltcpPinger;
use self::pq::{HopKeys, NegotiatedKeys};
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
        let params = Self::params();
        let (socket, _recv) = UdpSocketFactory::default().bind(&params).await?;
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
        let socket_guard = self.socket.lock().await;

        Ok((socket_guard.clone(), socket_guard.clone()))
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
    NegotiatePQError(NegotiatePQError),
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
            TunnelError::NegotiatePQError(e) => write!(f, "Negotiate PQ error: {e}"),
        }
    }
}

pub enum NegotiatePQError {
    Timeout,
    ObfuscationProxyError(ObfuscationProxyError),
    DeviceError(gotatun::device::Error),
    ExchangeError(String),
    Phase2ExitDeviceError(gotatun::device::Error),
    Phase2EntryDeviceError(gotatun::device::Error),
    Phase2ExchangeError(String),
    Phase2ObfuscationError(ObfuscationProxyError),
    Phase2Timeout,
}

impl std::fmt::Display for NegotiatePQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NegotiatePQError::Timeout => write!(f, "Negotiate PQ timeout"),
            NegotiatePQError::ObfuscationProxyError(e) => write!(f, "Obfuscation proxy error: {e}"),
            NegotiatePQError::DeviceError(msg) => write!(f, "Device error: {msg}"),
            NegotiatePQError::ExchangeError(msg) => write!(f, "Exchange error: {msg}"),
            NegotiatePQError::Phase2ExitDeviceError(msg) => {
                write!(f, "Phase 2 exit device error: {msg}")
            }
            NegotiatePQError::Phase2EntryDeviceError(msg) => {
                write!(f, "Phase 2 entry device error: {msg}")
            }
            NegotiatePQError::Phase2ExchangeError(msg) => {
                write!(f, "Phase 2 exchange error: {msg}")
            }
            NegotiatePQError::Phase2ObfuscationError(e) => {
                write!(f, "Phase 2 obfuscation error: {e}")
            }
            NegotiatePQError::Phase2Timeout => write!(f, "Phase 2 timeout"),
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

// Connectivity timeouts
const PING_INTERVAL: Duration = Duration::from_secs(3);
const CONNECTIVITY_CHECK_INTERVAL: Duration = Duration::from_millis(200);
/// After this long without any rx, consider the connection lost.
/// WireGuard keepalives are typically every ~25s, so 2 minutes gives plenty of margin.
const MONITOR_TIMEOUT: Duration = Duration::from_secs(120);
const SLEEP_CYCLE_RESET_THRESHOLD: Duration = Duration::from_secs(120);

/// Configuration of one GotaTun device carrying user traffic.
struct HopConfig {
    private_key: StaticSecret,
    peer: Peer,
}

impl HopConfig {
    fn new(peer: &PeerParameters, keys: &HopKeys, endpoint: SocketAddr) -> Self {
        let mut wg_peer = Peer::new(peer.public_key.into())
            .with_allowed_ips(peer.allowed_ips.clone())
            .with_endpoint(endpoint);
        if let Some(psk) = keys.preshared_key {
            wg_peer = wg_peer.with_preshared_key(psk);
        }
        Self {
            private_key: keys.private_key.clone(),
            peer: wg_peer,
        }
    }
}

/// Configuration of the final user-traffic device(s).
///
/// Derived from the immutable [`TunnelParameters`], the keys negotiated with the relays and
/// the obfuscation proxy.
struct ConnectionConfig {
    entry: Option<HopConfig>,
    exit: HopConfig,
}

impl ConnectionConfig {
    fn new(
        params: &TunnelParameters,
        keys: &NegotiatedKeys,
        obfuscation: Option<&ObfuscationProxy>,
    ) -> Self {
        let ingress_endpoint = pq::ingress_endpoint(params.ingress_peer(), obfuscation);

        match params.entry_peer.as_ref().zip(keys.entry.as_ref()) {
            Some((entry, entry_keys)) => Self {
                entry: Some(HopConfig::new(entry, entry_keys, ingress_endpoint)),
                exit: HopConfig::new(&params.exit_peer, &keys.exit, params.exit_peer.endpoint),
            },
            None => Self {
                entry: None,
                exit: HopConfig::new(&params.exit_peer, &keys.exit, ingress_endpoint),
            },
        }
    }

    fn ingress(&self) -> &HopConfig {
        self.entry.as_ref().unwrap_or(&self.exit)
    }
}

/// Callbacks from the tunnel adapter to Swift.
pub trait TunnelCallbackHandler: Send + Sync + 'static {
    fn on_connected(&self);
    fn on_timeout(&self);
    fn on_error(&self, error: TunnelError);
}

pub(crate) enum TunnelAdapterChannelCommand {
    Wake,
    Suspend,
    Stop,
    BumpSockets,
}

/// All state for a final connected WireGuard session.
struct ActiveConnection {
    devices: Devices,
    transport_provider: BoundUdpTransports,
    last_suspended_at: Option<talpid_time::Instant>,
    params: TunnelParameters,
    keys: NegotiatedKeys,
    obfuscation: ObfuscationSlot,
}

impl ActiveConnection {
    /// Transport provider should be the one used by Devices.
    fn new(
        devices: Devices,
        params: TunnelParameters,
        keys: NegotiatedKeys,
        obfuscation: ObfuscationSlot,
        transport_provider: BoundUdpTransports,
    ) -> Self {
        Self {
            devices,
            transport_provider,
            last_suspended_at: None,
            params,
            keys,
            obfuscation,
        }
    }

    async fn stop(mut self) {
        self.devices.stop().await;
        self.obfuscation.reset();
    }

    async fn bump_sockets(&mut self) -> Result<(), TunnelError> {
        self.devices.suspend().await;
        if let Err(err) = self.transport_provider.rebind().await {
            log::error!("Failed to rebind sockets: {err}");
        }
        self.restart_obfuscation().await?;
        self.devices.wake().await;
        Ok(())
    }

    async fn handle_devices_wake(&mut self) -> Result<(), TunnelError> {
        let now = talpid_time::Instant::now();
        let last = self.last_suspended_at.unwrap_or(now);
        let elapsed = now.duration_since(last);

        if elapsed >= SLEEP_CYCLE_RESET_THRESHOLD {
            self.restart_obfuscation().await?;
        }
        self.devices.wake().await;
        Ok(())
    }

    /// Replace the obfuscation proxy and point the ingress device at the new one.
    /// No-op when obfuscation is off.
    async fn restart_obfuscation(&mut self) -> Result<(), TunnelError> {
        if matches!(self.params.obfuscation, ObfuscationParameters::Off) {
            return Ok(());
        }
        self.obfuscation.reset();
        let proxy = self
            .obfuscation
            .for_key(&self.params, self.keys.ingress().public_key())
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;

        let config = ConnectionConfig::new(&self.params, &self.keys, proxy);
        self.devices
            .update_first_hop_peer(config.ingress().peer.clone())
            .await
            .map_err(TunnelError::GotaTunDeviceError)?;
        Ok(())
    }

    async fn handle_devices_suspend(&mut self) {
        self.last_suspended_at = Some(talpid_time::Instant::now());
        self.devices.suspend().await;
    }

    async fn total_rx(&self) -> usize {
        self.devices.total_rx().await
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
            task_handle: Some(task),
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        _ = self.tx.send(TunnelAdapterChannelCommand::Stop);
        log::debug!("Stopping device");
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
        log::debug!("Awaking device");
        _ = self.tx.send(TunnelAdapterChannelCommand::Wake);
    }

    async fn run(
        params: TunnelParameters,
        udp: BoundUdpTransports,
        callback: Arc<dyn TunnelCallbackHandler>,
        rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: Arc<AtomicBool>,
    ) {
        // Every phase below returns a `Result`; the callback is fired exactly
        // once, here, based on the final outcome.
        match Self::run_inner(params, udp, &callback, rx, &stopped).await {
            Ok(()) => Self::fire_timeout(&stopped, &callback),
            Err(
                TunnelError::Timeout | TunnelError::NegotiatePQError(NegotiatePQError::Timeout),
            ) => Self::fire_timeout(&stopped, &callback),
            Err(error) => Self::fire_error(&stopped, &callback, error),
        }
    }

    async fn run_inner(
        params: TunnelParameters,
        udp: BoundUdpTransports,
        callback: &Arc<dyn TunnelCallbackHandler>,
        mut rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<(), TunnelError> {
        // 1. Create the TUN device from the fd handed over by iOS.
        let tun_dev =
            IosTunDevice::new(params.tun_fd, params.mtu).map_err(TunnelError::TunnelDevice)?;

        // 2. Negotiate the PQ/DAITA ephemeral peer(s) over smoltcp-only devices, or fall
        //    back to the device key. The obfuscation proxy is shared with the final device.
        let mut obfuscation = ObfuscationSlot::default();
        let keys = pq::negotiate(&params, &udp, &mut obfuscation, stopped)
            .await
            .map_err(TunnelError::NegotiatePQError)?;

        if stopped.load(Ordering::SeqCst) {
            // Cancelled externally; the outcome below is discarded since `run`
            // no-ops when it sees the tunnel is already stopped.
            return Err(TunnelError::Timeout);
        }

        // 3. The final device handshakes with the negotiated ingress key; LWO must follow it.
        let proxy = obfuscation
            .for_key(&params, keys.ingress().public_key())
            .await
            .map_err(TunnelError::ObfuscationProxyError)?;

        let config = ConnectionConfig::new(&params, &keys, proxy);

        // 4. Build the user-traffic device(s) behind an IpMuxRecv and IpMuxSend (TUN + smoltcp).
        let (smoltcp_handle, ip_recv, ip_send, _smoltcp_guard) =
            smoltcp_network(params.smoltcp_network_config());
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv, ip_send);

        let devices = Self::build_devices(&params, &udp, config, mux_recv, mux_send).await?;
        if stopped.load(Ordering::SeqCst) {
            devices.stop().await;
            return Err(TunnelError::Timeout);
        }

        // 5. Establish connectivity, then monitor it until it drops or we stop.
        let connected = match Self::establish_connectivity(
            &devices,
            &smoltcp_handle,
            &params,
            stopped,
            &mut rx,
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

        let mut holder = ActiveConnection::new(devices, params, keys, obfuscation, udp);

        callback.on_connected();
        log::info!("Tunnel connected - starting ongoing monitoring");
        Self::monitor_connectivity(&mut holder, rx, stopped).await?;
        holder.stop().await;
        Err(TunnelError::Timeout)
    }

    /// Build the final GotaTun device(s) carrying user traffic.
    async fn build_devices(
        params: &TunnelParameters,
        udp: &BoundUdpTransports,
        config: ConnectionConfig,
        mux_recv: tun_device::IosTunIpRecv,
        mux_send: tun_device::IosTunIpSend,
    ) -> Result<Devices, TunnelError> {
        let Some((entry_params, entry)) = params.entry_peer.as_ref().zip(config.entry) else {
            // Singlehop: one device, mux'd IP pair, real UDP.
            let device = DeviceBuilder::new()
                .with_udp(udp.clone())
                .with_ip_pair(mux_send, mux_recv)
                .with_private_key(config.exit.private_key)
                .with_peer(config.exit.peer)
                .build()
                .await
                .map_err(TunnelError::GotaTunDeviceError)?;
            return Ok(Devices::Singlehop(device));
        };

        // Multihop: exit device tunnels its UDP through the entry device.
        let (tun_channel_tx, tun_channel_rx, udp_channels) = new_udp_tun_channel(
            100,
            params.ipv4_addr,
            params.ipv6_addr,
            params.entry_mtu(entry_params),
        );

        let exit_device = DeviceBuilder::new()
            .with_udp(udp_channels)
            .with_ip_pair(mux_send, mux_recv)
            .with_private_key(config.exit.private_key)
            .with_peer(config.exit.peer)
            .build()
            .await
            .map_err(TunnelError::MultihopExitDeviceError)?;

        log::info!(
            "Multihop: entry={}, exit={}",
            entry_params.endpoint,
            params.exit_peer.endpoint
        );

        let entry_device = match DeviceBuilder::new()
            .with_udp(udp.clone())
            .with_ip_pair(tun_channel_tx, tun_channel_rx)
            .with_peer(entry.peer)
            .with_private_key(entry.private_key)
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
        params: &TunnelParameters,
        stopped: &AtomicBool,
        rx: &mut UnboundedReceiver<TunnelAdapterChannelCommand>,
    ) -> Result<bool, TunnelError> {
        // Bind the socket to the pinger's ident so echo replies reach it.
        let ping_ident: u16 = rand::random();
        let icmp_socket = smoltcp_handle
            .icmp_socket(ping_ident)
            .await
            .map_err(TunnelError::ICMPSocketError)?;
        let mut pinger = SmoltcpPinger::new(icmp_socket, params.ipv4_gateway, ping_ident);

        let establish_timeout = params.establish_timeout();
        log::info!("Establishing connectivity (timeout: {establish_timeout:?})");

        if let Err(e) = pinger.send_icmp().await {
            log::warn!("Initial ping failed: {e}");
        }

        Ok(tokio::select! {
            result = Self::wait_for_connectivity(devices, &mut pinger, stopped) => result,
            _ = tokio::time::sleep(establish_timeout) => false,
            event = rx.recv() =>
                !matches!(event, Some(TunnelAdapterChannelCommand::Stop))
            ,
        })
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
    async fn monitor_connectivity(
        device_holder: &mut ActiveConnection,
        mut rx: UnboundedReceiver<TunnelAdapterChannelCommand>,
        stopped: &AtomicBool,
    ) -> Result<(), TunnelError> {
        let mut last_rx_bytes: usize = 0;
        let mut last_rx_time = Instant::now();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                event = rx.recv() => {
                   match event {
                        Some(TunnelAdapterChannelCommand::BumpSockets) => {
                            device_holder.bump_sockets().await?;
                            last_rx_time = Instant::now();
                        },
                        Some(TunnelAdapterChannelCommand::Wake) => {
                            device_holder.handle_devices_wake().await?;
                            last_rx_time = Instant::now();
                        },
                        Some(TunnelAdapterChannelCommand::Suspend) => {
                            device_holder.handle_devices_suspend().await;
                        },
                        Some(TunnelAdapterChannelCommand::Stop) => () ,
                        None => break Ok(())
                    };
                }
            }

            if stopped.load(Ordering::SeqCst) {
                break Ok(());
            }

            let total_rx = device_holder.total_rx().await;

            if total_rx > last_rx_bytes {
                last_rx_bytes = total_rx;
                last_rx_time = Instant::now();
            } else if last_rx_time.elapsed() > MONITOR_TIMEOUT {
                log::warn!("No RX for {:?} - connection lost", last_rx_time.elapsed());
                break Ok(());
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

    /// Replace the peer of the ingress device
    async fn update_first_hop_peer(&self, peer: Peer) -> Result<bool, gotatun::device::Error> {
        match self {
            Devices::Singlehop(dev) => dev.update_peer(peer).await,
            Devices::Multihop { entry, .. } => entry.update_peer(peer).await,
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
    use super::params::tests::{params, peer};
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

    fn keys(seed: u8, psk: Option<[u8; 32]>) -> HopKeys {
        HopKeys {
            private_key: StaticSecret::from([seed; 32]),
            preshared_key: psk,
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
    fn connection_config_uses_relay_endpoints_without_obfuscation() {
        let p = params();
        let k = NegotiatedKeys {
            entry: None,
            exit: keys(2, None),
        };
        let config = ConnectionConfig::new(&p, &k, None);
        assert!(config.entry.is_none());
        assert_eq!(config.exit.peer.endpoint, Some(p.exit_peer.endpoint));
        assert_eq!(config.exit.peer.preshared_key, None);
        assert_eq!(config.exit.private_key.to_bytes(), [2u8; 32]);
    }

    #[test]
    fn connection_config_carries_negotiated_keys_per_hop() {
        let mut p = params();
        p.entry_peer = Some(peer("9.9.9.9:51820"));
        let k = NegotiatedKeys {
            entry: Some(keys(1, Some([11u8; 32]))),
            exit: keys(2, Some([22u8; 32])),
        };

        let config = ConnectionConfig::new(&p, &k, None);
        let entry = config.entry.as_ref().unwrap();
        assert_eq!(entry.private_key.to_bytes(), [1u8; 32]);
        assert_eq!(entry.peer.preshared_key, Some([11u8; 32]));
        assert_eq!(entry.peer.endpoint, Some("9.9.9.9:51820".parse().unwrap()));
        assert_eq!(config.exit.private_key.to_bytes(), [2u8; 32]);
        assert_eq!(config.exit.peer.preshared_key, Some([22u8; 32]));
        assert_eq!(config.exit.peer.endpoint, Some(p.exit_peer.endpoint));
        assert_eq!(config.ingress().peer.public_key, entry.peer.public_key);
    }

    #[test]
    fn hop_config_keeps_parameter_allowed_ips() {
        let p = peer("1.2.3.4:51820");
        let hop = HopConfig::new(&p, &keys(1, None), "127.0.0.1:9999".parse().unwrap());
        assert_eq!(hop.peer.allowed_ips, p.allowed_ips);
        assert_eq!(hop.peer.endpoint, Some("127.0.0.1:9999".parse().unwrap()));
    }
}
