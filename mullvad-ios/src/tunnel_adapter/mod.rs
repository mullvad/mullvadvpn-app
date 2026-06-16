//! iOS tunnel adapter — manages a single GotaTun connection attempt.
//!
//! Each instance drives one connection lifecycle:
//! 1. Create TUN device from fd
//! 2. Set up smoltcp network stack + IP mux
//! 3. Build and configure a GotaTun Device with WireGuard peers
//! 4. Run connectivity monitor (pinger + stats)
//! 5. Call back to Swift: onConnected / onTimeout / onError
//!
//! After a terminal callback fires, the instance is considered dead.

#[cfg(target_os = "ios")]
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

use crate::gotatun::{
    WIREGUARD_HEADER_SIZE,
    ip_mux::ip_mux,
    smoltcp_network::{SmoltcpHandle, SmoltcpNetworkConfig, smoltcp_network},
};
use gotatun::{
    device::{DeviceBuilder, DeviceTransports, Peer},
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::MtuWatcher,
    udp::{channel::new_udp_tun_channel, socket::UdpSocketFactory},
    x25519::StaticSecret,
};
use ipnetwork::IpNetwork;
use talpid_tunnel_config_client::{
    self, EphemeralPeer, RelayConfigService, request_ephemeral_peer_with,
};
use talpid_types::net::wireguard::{PrivateKey, PublicKey};
use tokio::sync::Notify;
use tonic::transport::channel::Endpoint;
use tower::util::service_fn;
use tunnel_obfuscation::create_obfuscator;

use self::pinger::SmoltcpPinger;
use self::tun_device::IosTunDevice;

/// Guard that aborts the obfuscation proxy task on drop.
struct ObfuscationGuard {
    endpoint: SocketAddr,
    _task: tokio::task::JoinHandle<()>,
}

impl ObfuscationGuard {
    fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
}

/// Error from a PQ key exchange phase.
enum PqExchangeError {
    Timeout,
    Failed(String),
}

/// Result of [`IosTunnelAdapter::negotiate_pq`]: the keys and peers to configure
/// the final device(s) with — `(entry, exit_key, exit_peer)`. `entry` is `Some`
/// only for multihop PQ.
type PqResult = (Option<(StaticSecret, Peer)>, StaticSecret, Peer);

const CONFIG_SERVICE_ADDR: &str = "10.64.0.1:1337";

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
    pub ipv6_addr: Option<Ipv6Addr>,
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
        self.mtu.saturating_sub(WIREGUARD_HEADER_SIZE)
    }

    /// Timeout for establishing connectivity, clamped to at least one second.
    fn establish_timeout(&self) -> Duration {
        Duration::from_secs(self.establish_timeout_secs.max(1) as u64)
    }
}

/// Obfuscation configuration for the tunnel.
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
    fn on_error(&self, message: String);
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
        callback: Arc<dyn TunnelCallbackHandler>,
    ) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let stop_notify = Arc::new(Notify::new());

        let task = runtime.spawn(Self::run(
            config,
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
        callback: Arc<dyn TunnelCallbackHandler>,
        stopped: Arc<AtomicBool>,
        stop_notify: Arc<Notify>,
    ) {
        let is_stopped = || stopped.load(Ordering::SeqCst);

        // 1. Create the TUN device from the fd handed over by iOS.
        let tun_dev = match IosTunDevice::new(config.tun_fd, config.mtu) {
            Ok(dev) => dev,
            Err(e) => return Self::fire_error(&stopped, &callback, format!("TUN device: {e}")),
        };

        let mut config = config;

        // 2. Negotiate the PQ/DAITA ephemeral peer(s) over a smoltcp-only device,
        //    or fall back to the static device peer. On any terminal outcome the
        //    callback has already fired, so we just return.
        let pq = match Self::negotiate_pq(&mut config, &stopped, &callback).await {
            Some(pq) => pq,
            None => return,
        };
        if is_stopped() {
            return;
        }

        // 3. After PQ the WireGuard handshake uses the ephemeral ingress key, so
        //    point LWO at it, then start the obfuscation proxy for the real device.
        Self::apply_lwo_ingress_key(&mut config, &pq);
        let _final_obfuscation = match Self::start_obfuscation_proxy(&config).await {
            Ok(ob) => {
                if let Some(ref guard) = ob {
                    Self::apply_obfuscation(&mut config, guard.endpoint());
                }
                ob
            }
            Err(e) => {
                return Self::fire_error(&stopped, &callback, format!("Final obfuscation: {e}"));
            }
        };

        // 4. Build the user-traffic device(s) behind an IpMux (TUN + smoltcp).
        let (smoltcp_handle, ip_recv, ip_send, _smoltcp_guard) =
            smoltcp_network(SmoltcpNetworkConfig {
                ipv4_addr: config.ipv4_addr,
                ipv6_addr: config.ipv6_addr,
                mtu: config.smoltcp_mtu(),
            });
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv, ip_send);

        let devices =
            match Self::build_devices(&config, pq, mux_recv, mux_send, &stopped, &callback).await {
                Some(devices) => devices,
                None => return,
            };
        if is_stopped() {
            devices.stop().await;
            return;
        }

        // 5. Establish connectivity, then monitor it until it drops or we stop.
        let connected = Self::establish_connectivity(
            &devices,
            &smoltcp_handle,
            &config,
            &stopped,
            &callback,
            &stop_notify,
        )
        .await;

        if is_stopped() {
            devices.stop().await;
            return;
        }
        if !connected {
            devices.stop().await;
            return Self::fire_timeout(&stopped, &callback);
        }

        callback.on_connected();
        log::info!("Tunnel connected — starting ongoing monitoring");
        Self::monitor_connectivity(&devices, &stopped, &stop_notify).await;
        devices.stop().await;
        Self::fire_timeout(&stopped, &callback);
    }

    /// Negotiate the post-quantum / DAITA ephemeral peer(s).
    ///
    /// Returns the `(entry, exit_key, exit_peer)` triple to configure the final
    /// device(s) with — `entry` is `Some` only for multihop PQ. Returns `None`
    /// if a terminal callback already fired (error/timeout) or we were stopped,
    /// in which case the caller should just return.
    async fn negotiate_pq(
        config: &mut TunnelConfig,
        stopped: &AtomicBool,
        callback: &Arc<dyn TunnelCallbackHandler>,
    ) -> Option<PqResult> {
        let is_stopped = || stopped.load(Ordering::SeqCst);

        // No PQ/DAITA: the device peer is just the static configured exit peer.
        if !(config.enable_pq || config.enable_daita) {
            let private_key = StaticSecret::from(config.private_key);
            return Some((None, private_key, Self::build_peer(&config.exit_peer)));
        }

        let pq_timeout = config.establish_timeout();
        let parent_pubkey = PrivateKey::from(config.private_key).public_key();
        let is_multihop = config.entry_peer.is_some();
        let first_peer_config = config.entry_peer.as_ref().unwrap_or(&config.exit_peer);

        // --- Phase 1: negotiate with the first relay (entry in multihop, exit in singlehop) ---
        log::info!(
            "PQ phase 1: negotiating with {} (pq={}, daita={})",
            first_peer_config.endpoint,
            config.enable_pq,
            config.enable_daita
        );

        let (first_key, mut first_peer) = match Self::run_pq_exchange(
            config,
            first_peer_config,
            parent_pubkey.clone(),
            pq_timeout,
        )
        .await
        {
            Ok(result) => result,
            Err(PqExchangeError::Timeout) => {
                Self::fire_timeout(stopped, callback);
                return None;
            }
            Err(PqExchangeError::Failed(e)) => {
                Self::fire_error(stopped, callback, e);
                return None;
            }
        };

        if !is_multihop {
            return Some((None, first_key, first_peer));
        }
        if is_stopped() {
            return None;
        }

        // --- Phase 2: reach the exit relay through the entry, using the phase-1
        //     ephemeral key, so that 10.64.0.1:1337 hits the EXIT config service. ---
        log::info!("PQ phase 2: negotiating with exit via entry ephemeral key");

        // The entry connection now uses phase 1's ephemeral key, so LWO must too.
        if let ObfuscationConfig::Lwo {
            ref mut client_public_key,
            ..
        } = config.obfuscation
        {
            *client_public_key = gotatun::x25519::PublicKey::from(&first_key).to_bytes();
        }
        let obfuscation_p2 = match Self::start_obfuscation_proxy(config).await {
            Ok(ob) => ob,
            Err(e) => {
                Self::fire_error(stopped, callback, format!("PQ2 obfuscation: {e}"));
                return None;
            }
        };
        if let Some(ref ob) = obfuscation_p2 {
            first_peer = first_peer.with_endpoint(ob.endpoint());
        }

        let entry_peer_config = config.entry_peer.as_ref().unwrap();
        let entry_mtu = MtuWatcher::new(config.mtu)
            .increase(Self::multihop_overhead(entry_peer_config.endpoint))
            .expect("MTU overflow");

        let (smoltcp_handle, ip_recv, ip_send, _guard) = smoltcp_network(SmoltcpNetworkConfig {
            ipv4_addr: config.ipv4_addr,
            ipv6_addr: config.ipv6_addr,
            mtu: config.smoltcp_mtu(),
        });
        let (ch_tx, ch_rx, udp_ch) = new_udp_tun_channel(
            100,
            config.ipv4_addr,
            config.ipv6_addr.unwrap_or(Ipv6Addr::UNSPECIFIED),
            entry_mtu,
        );

        // Exit device: smoltcp IP pair, UDP channeled through the entry.
        let pq2_exit = match DeviceBuilder::new()
            .with_udp(udp_ch)
            .with_ip_pair(ip_send, ip_recv)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                Self::fire_error(stopped, callback, format!("PQ2 exit device: {e}"));
                return None;
            }
        };
        // Entry device: real UDP, channel IP pair.
        let pq2_entry = match DeviceBuilder::new()
            .with_udp(UdpSocketFactory::default())
            .with_ip_pair(ch_tx, ch_rx)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                pq2_exit.stop().await;
                Self::fire_error(stopped, callback, format!("PQ2 entry device: {e}"));
                return None;
            }
        };

        // Entry uses the phase-1 ephemeral key; exit uses the device key.
        let device_key = StaticSecret::from(config.private_key);
        let exit_initial = Self::build_peer(&config.exit_peer);
        let configured = tokio::try_join!(
            Self::configure_device(&pq2_entry, first_key.clone(), first_peer.clone()),
            Self::configure_device(&pq2_exit, device_key, exit_initial),
        );
        if let Err(e) = configured {
            pq2_entry.stop().await;
            pq2_exit.stop().await;
            Self::fire_error(stopped, callback, format!("Configure PQ phase 2: {e}"));
            return None;
        }

        // Now 10.64.0.1:1337 reaches the EXIT relay's config service.
        let exit_ephemeral_private = PrivateKey::new_from_random();
        let exit_ephemeral_pubkey = exit_ephemeral_private.public_key();

        let exchange_result = tokio::time::timeout(
            pq_timeout,
            Self::negotiate_ephemeral_peer(
                &smoltcp_handle,
                parent_pubkey.clone(),
                exit_ephemeral_pubkey,
                config.enable_pq,
                config.enable_daita,
            ),
        )
        .await;

        pq2_entry.stop().await;
        pq2_exit.stop().await;
        drop(obfuscation_p2);

        match exchange_result {
            Ok(Ok(exit_ephemeral)) => {
                log::info!(
                    "PQ phase 2 complete (psk={}, daita={})",
                    exit_ephemeral.psk.is_some(),
                    exit_ephemeral.daita.is_some()
                );

                let exit_secret = StaticSecret::from(exit_ephemeral_private.to_bytes());
                let mut exit_peer = Self::build_peer(&config.exit_peer);
                if let Some(ref psk) = exit_ephemeral.psk {
                    exit_peer = exit_peer.with_preshared_key(*psk.as_bytes());
                }
                Some((Some((first_key, first_peer)), exit_secret, exit_peer))
            }
            Ok(Err(e)) => {
                Self::fire_error(stopped, callback, format!("PQ phase 2: {e}"));
                None
            }
            Err(_) => {
                Self::fire_timeout(stopped, callback);
                None
            }
        }
    }

    /// Build the final GotaTun device(s) carrying user traffic and configure
    /// their peers. Returns `None` (after firing the error callback) on failure.
    async fn build_devices(
        config: &TunnelConfig,
        pq: PqResult,
        mux_recv: tun_device::IosTunIpRecv,
        mux_send: tun_device::IosTunIpSend,
        stopped: &AtomicBool,
        callback: &Arc<dyn TunnelCallbackHandler>,
    ) -> Option<Devices> {
        let (pq_entry, pq_exit_key, pq_exit_peer) = pq;

        let Some(entry_peer_config) = config.entry_peer.as_ref() else {
            // Singlehop: one device, mux'd IP pair, real UDP.
            let device = match DeviceBuilder::new()
                .with_udp(UdpSocketFactory::default())
                .with_ip_pair(mux_send, mux_recv)
                .build()
                .await
            {
                Ok(dev) => dev,
                Err(e) => {
                    Self::fire_error(stopped, callback, format!("GotaTun device: {e}"));
                    return None;
                }
            };
            // Endpoint must match the (possibly obfuscated) config endpoint.
            let pq_exit_peer = pq_exit_peer.with_endpoint(config.exit_peer.endpoint);
            if let Err(e) = Self::configure_device(&device, pq_exit_key, pq_exit_peer).await {
                device.stop().await;
                Self::fire_error(stopped, callback, format!("Configure peers: {e}"));
                return None;
            }
            return Some(Devices::Singlehop(device));
        };

        // Multihop: exit device tunnels its UDP through the entry device.
        let entry_mtu = MtuWatcher::new(config.mtu)
            .increase(Self::multihop_overhead(entry_peer_config.endpoint))
            .expect("MTU overflow");
        let (tun_channel_tx, tun_channel_rx, udp_channels) = new_udp_tun_channel(
            100,
            config.ipv4_addr,
            config.ipv6_addr.unwrap_or(Ipv6Addr::UNSPECIFIED),
            entry_mtu,
        );

        let exit_device = match DeviceBuilder::new()
            .with_udp(udp_channels)
            .with_ip_pair(mux_send, mux_recv)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                Self::fire_error(stopped, callback, format!("Exit device: {e}"));
                return None;
            }
        };
        let entry_device = match DeviceBuilder::new()
            .with_udp(UdpSocketFactory::default())
            .with_ip_pair(tun_channel_tx, tun_channel_rx)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                exit_device.stop().await;
                Self::fire_error(stopped, callback, format!("Entry device: {e}"));
                return None;
            }
        };

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

        let configured = tokio::try_join!(
            Self::configure_device(&entry_device, entry_key, entry_peer),
            Self::configure_device(&exit_device, pq_exit_key, pq_exit_peer),
        );
        if let Err(e) = configured {
            entry_device.stop().await;
            exit_device.stop().await;
            Self::fire_error(stopped, callback, format!("Configure peers: {e}"));
            return None;
        }

        Some(Devices::Multihop {
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
        callback: &Arc<dyn TunnelCallbackHandler>,
        stop_notify: &Notify,
    ) -> bool {
        let icmp_socket = match smoltcp_handle.icmp_socket(0).await {
            Ok(socket) => socket,
            Err(e) => {
                Self::fire_error(stopped, callback, format!("ICMP socket: {e}"));
                return false;
            }
        };
        let mut pinger = SmoltcpPinger::new(icmp_socket, config.ipv4_gateway);

        let establish_timeout = config.establish_timeout();
        log::info!("Establishing connectivity (timeout: {establish_timeout:?})");

        if let Err(e) = pinger.send_icmp().await {
            log::warn!("Initial ping failed: {e}");
        }

        tokio::select! {
            result = Self::wait_for_connectivity(devices, &mut pinger, stopped) => result,
            _ = tokio::time::sleep(establish_timeout) => false,
            _ = stop_notify.notified() => false,
        }
    }

    /// Run a single PQ exchange phase: start obfuscation proxy, build smoltcp-only device,
    /// configure peer, negotiate.
    async fn run_pq_exchange(
        config: &TunnelConfig,
        peer_config: &PeerConfig,
        parent_pubkey: PublicKey,
        timeout: Duration,
    ) -> Result<(StaticSecret, Peer), PqExchangeError> {
        // Start a fresh obfuscation proxy for this PQ device
        let _obfuscation = Self::start_obfuscation_proxy(config)
            .await
            .map_err(|e| PqExchangeError::Failed(format!("PQ obfuscation proxy: {e}")))?;

        let peer_endpoint = _obfuscation
            .as_ref()
            .map(|ob| ob.endpoint())
            .unwrap_or(peer_config.endpoint);

        let (handle, ip_recv, ip_send, _guard) = smoltcp_network(SmoltcpNetworkConfig {
            ipv4_addr: config.ipv4_addr,
            ipv6_addr: config.ipv6_addr,
            mtu: config.smoltcp_mtu(),
        });

        let pq_device = DeviceBuilder::new()
            .with_udp(UdpSocketFactory::default())
            .with_ip_pair(ip_send, ip_recv)
            .build()
            .await
            .map_err(|e| PqExchangeError::Failed(format!("PQ device: {e}")))?;

        let private_key = StaticSecret::from(config.private_key);
        let initial_peer = Self::build_peer(peer_config).with_endpoint(peer_endpoint);

        if let Err(e) = Self::configure_device(&pq_device, private_key, initial_peer).await {
            pq_device.stop().await;
            return Err(PqExchangeError::Failed(format!("PQ device configure: {e}")));
        }

        let ephemeral_private = PrivateKey::new_from_random();
        let ephemeral_pubkey = ephemeral_private.public_key();

        let exchange_result = tokio::time::timeout(
            timeout,
            Self::negotiate_ephemeral_peer(
                &handle,
                parent_pubkey,
                ephemeral_pubkey,
                config.enable_pq,
                config.enable_daita,
            ),
        )
        .await;

        pq_device.stop().await;

        match exchange_result {
            Ok(Ok(ephemeral_peer)) => {
                let ephemeral_secret = StaticSecret::from(ephemeral_private.to_bytes());
                let mut peer = Self::build_peer(peer_config);

                if let Some(ref psk) = ephemeral_peer.psk {
                    peer = peer.with_preshared_key(*psk.as_bytes());
                }

                Ok((ephemeral_secret, peer))
            }
            Ok(Err(e)) => Err(PqExchangeError::Failed(format!("PQ exchange: {e}"))),
            Err(_) => Err(PqExchangeError::Timeout),
        }
    }

    /// Start a fresh obfuscation proxy for the given config.
    /// Returns a guard that keeps the proxy alive until dropped.
    /// Returns `None` if obfuscation is off.
    async fn start_obfuscation_proxy(
        config: &TunnelConfig,
    ) -> Result<Option<ObfuscationGuard>, String> {
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
                    .map_err(|e| format!("Invalid QUIC token: {e}"))?;
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
            }),
        };

        let obfuscator = create_obfuscator(&settings)
            .await
            .map_err(|e| format!("Obfuscation proxy: {e}"))?;
        let endpoint = obfuscator.endpoint();
        log::info!("Obfuscation proxy started at {endpoint}");
        let task = tokio::spawn(async move {
            let _ = obfuscator.run().await;
        });
        Ok(Some(ObfuscationGuard {
            endpoint,
            _task: task,
        }))
    }

    /// Apply obfuscation to the config: replace the ingress peer's endpoint with the proxy address.
    fn apply_obfuscation(config: &mut TunnelConfig, proxy_endpoint: SocketAddr) {
        if let Some(ref mut entry) = config.entry_peer {
            entry.endpoint = proxy_endpoint;
        } else {
            config.exit_peer.endpoint = proxy_endpoint;
        }
    }

    /// Negotiate an ephemeral peer via gRPC through the smoltcp TCP stack.
    async fn negotiate_ephemeral_peer(
        smoltcp_handle: &SmoltcpHandle,
        parent_pubkey: PublicKey,
        ephemeral_pubkey: PublicKey,
        enable_pq: bool,
        enable_daita: bool,
    ) -> Result<EphemeralPeer, String> {
        let addr: std::net::SocketAddr = CONFIG_SERVICE_ADDR
            .parse()
            .map_err(|e| format!("Bad config service addr: {e}"))?;

        let stream = smoltcp_handle
            .tcp_connect(addr)
            .await
            .map_err(|e| format!("TCP connect to config service: {e}"))?;

        // The connector is called exactly once by tonic; use a Mutex to hand off the stream.
        let stream_cell = std::sync::Mutex::new(Some(stream));

        let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
        let conn = endpoint
            .connect_with_connector(service_fn(move |_| {
                let stream = stream_cell
                    .lock()
                    .ok()
                    .and_then(|mut guard| guard.take())
                    .ok_or_else(|| io::Error::other("connector stream unavailable"));
                async move { Ok::<_, io::Error>(hyper_util::rt::tokio::TokioIo::new(stream?)) }
            }))
            .await
            .map_err(|e| format!("gRPC connect: {e}"))?;

        let service = RelayConfigService::new(conn);
        request_ephemeral_peer_with(
            service,
            parent_pubkey,
            ephemeral_pubkey,
            enable_pq,
            enable_daita,
        )
        .await
        .map_err(|e| format!("Ephemeral peer exchange: {e}"))
    }

    async fn configure_device<T: DeviceTransports>(
        device: &gotatun::device::Device<T>,
        private_key: StaticSecret,
        peer: Peer,
    ) -> Result<(), String> {
        device
            .write(async |dev| {
                dev.clear_peers();
                dev.set_private_key(private_key).await;
                dev.add_peer(peer);
            })
            .await
            .map_err(|e| format!("{e:#}"))
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
                log::debug!("Connectivity established — rx_bytes > 0");
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
                log::warn!("No RX for {:?} — connection lost", last_rx_time.elapsed());
                return;
            }
        }
    }

    fn fire_error(stopped: &AtomicBool, callback: &Arc<dyn TunnelCallbackHandler>, msg: String) {
        if !stopped.swap(true, Ordering::SeqCst) {
            log::error!("Tunnel adapter error: {msg}");
            callback.on_error(msg);
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
        let ObfuscationConfig::Lwo {
            ref mut client_public_key,
            ..
        } = config.obfuscation
        else {
            return;
        };
        let (pq_entry, pq_exit_key, _) = pq;
        *client_public_key = match pq_entry {
            Some((entry_key, _)) => gotatun::x25519::PublicKey::from(entry_key).to_bytes(),
            None => gotatun::x25519::PublicKey::from(pq_exit_key).to_bytes(),
        };
    }
}

impl Drop for IosTunnelAdapter {
    fn drop(&mut self) {
        self.stop();
    }
}

fn localhost_wg_endpoint(peer: SocketAddr) -> SocketAddr {
    if peer.is_ipv4() {
        SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
    } else {
        SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
    }
}

// MARK: - Device container

/// Holds either a single device or an entry+exit pair for multihop.
enum Devices {
    Singlehop(
        gotatun::device::Device<(
            UdpSocketFactory,
            tun_device::IosTunIpSend,
            tun_device::IosTunIpRecv,
        )>,
    ),
    Multihop {
        entry: gotatun::device::Device<(
            UdpSocketFactory,
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

    /// Peer stats of the ingress device — the one whose rx reflects tunnel
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
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    /// Records callback invocations so tests can assert on terminal behaviour.
    #[derive(Default)]
    struct CountingCallback {
        connected: AtomicUsize,
        timeout: AtomicUsize,
        errors: Mutex<Vec<String>>,
    }

    impl TunnelCallbackHandler for CountingCallback {
        fn on_connected(&self) {
            self.connected.fetch_add(1, Ordering::SeqCst);
        }
        fn on_timeout(&self) {
            self.timeout.fetch_add(1, Ordering::SeqCst);
        }
        fn on_error(&self, message: String) {
            self.errors.lock().unwrap().push(message);
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
            ipv6_addr: None,
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

        IosTunnelAdapter::fire_error(&stopped, &dynamic, "boom".into());
        assert!(stopped.load(Ordering::SeqCst));
        assert_eq!(concrete.errors.lock().unwrap().as_slice(), ["boom"]);

        // Already stopped: neither a second error nor a timeout fires.
        IosTunnelAdapter::fire_error(&stopped, &dynamic, "again".into());
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
        assert_eq!(c.smoltcp_mtu(), 1280 - WIREGUARD_HEADER_SIZE);
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
        assert!(matches!(off.obfuscation, ObfuscationConfig::Off));
    }
}
