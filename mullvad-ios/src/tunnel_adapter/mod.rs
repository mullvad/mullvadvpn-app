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

pub(crate) mod ffi;
mod pinger;
pub(crate) mod pq_negotiation;
pub(crate) mod tun_device;

use std::{
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
    smoltcp_network::{SmoltcpHandle, SmoltcpNetwork, SmoltcpNetworkConfig},
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
    pub retry_attempt: u32,
    pub establish_timeout_secs: u32,
    pub enable_pq: bool,
    pub enable_daita: bool,
    pub obfuscation: ObfuscationConfig,
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

        // 1. Create TUN device from fd
        let tun_dev = match IosTunDevice::new(config.tun_fd, config.mtu) {
            Ok(dev) => dev,
            Err(e) => {
                return Self::fire_error(&stopped, &callback, format!("TUN device: {e}"));
            }
        };

        let mut config = config;

        // 2. PQ/DAITA: negotiate ephemeral peer using smoltcp-only device (no TUN traffic leaks)
        let smoltcp_mtu = config.mtu.saturating_sub(WIREGUARD_HEADER_SIZE);
        // Returns (entry_key, entry_peer_builder, exit_key, exit_peer_builder)
        // In singlehop, entry fields are unused.
        // PQ result: (Option<(entry_key, entry_peer)>, exit_key, exit_peer)
        let (pq_entry, pq_exit_key, pq_exit_peer) = if config.enable_pq || config.enable_daita {
            let pq_timeout = Duration::from_secs(config.establish_timeout_secs.max(1) as u64);
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

            let (first_key, first_peer) = Self::run_pq_exchange(
                &config,
                first_peer_config,
                parent_pubkey.clone(),
                pq_timeout,
                &stopped,
                &stop_notify,
            )
            .await;

            let (first_key, mut first_peer) = match (first_key, first_peer) {
                (Some(k), Some(p)) => (k, p),
                _ => {
                    if !is_stopped() {
                        stopped.store(true, Ordering::SeqCst);
                        callback.on_timeout();
                    }
                    return;
                }
            };

            if !is_multihop {
                // Singlehop: done
                (None, first_key, first_peer)
            } else {
                if is_stopped() {
                    return;
                }

                // --- Phase 2: reach exit relay through entry with ephemeral key ---
                log::info!("PQ phase 2: negotiating with exit via entry ephemeral key");

                // Phase 2: build a multihop pair (entry ephemeral → exit device key)
                // so that 10.64.0.1:1337 reaches the EXIT relay's config service.
                // Update LWO client key to phase 1's ephemeral key (used for the entry connection)
                if let ObfuscationConfig::Lwo {
                    ref mut client_public_key,
                    ..
                } = config.obfuscation
                {
                    *client_public_key = gotatun::x25519::PublicKey::from(&first_key).to_bytes();
                }
                // Start a fresh obfuscation proxy for the entry connection.
                let obfuscation_p2 = match Self::start_obfuscation_proxy(&config).await {
                    Ok(ob) => ob,
                    Err(e) => {
                        return Self::fire_error(
                            &stopped,
                            &callback,
                            format!("PQ2 obfuscation: {e}"),
                        );
                    }
                };
                if let Some((ep, _)) = &obfuscation_p2 {
                    // Update the entry peer in first_peer to use the proxy
                    first_peer = first_peer.with_endpoint(*ep);
                }

                let entry_peer_config = config.entry_peer.as_ref().unwrap();
                let multihop_overhead = match entry_peer_config.endpoint.ip() {
                    IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
                    IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
                };

                let smoltcp_net_p2 = SmoltcpNetwork::new(SmoltcpNetworkConfig {
                    ipv4_addr: config.ipv4_addr,
                    ipv6_addr: config.ipv6_addr,
                    mtu: smoltcp_mtu,
                });
                let (smoltcp_handle_p2, ip_recv_p2, ip_send_p2, _guard_p2) =
                    smoltcp_net_p2.into_parts();

                let entry_mtu_p2 = MtuWatcher::new(config.mtu)
                    .increase(multihop_overhead as u16)
                    .expect("MTU overflow");
                let (ch_tx, ch_rx, udp_ch) = new_udp_tun_channel(
                    100,
                    config.ipv4_addr,
                    config.ipv6_addr.unwrap_or(Ipv6Addr::UNSPECIFIED),
                    entry_mtu_p2,
                );

                // Exit device: smoltcp IP pair, channeled UDP through entry
                let pq2_exit = match DeviceBuilder::new()
                    .with_udp(udp_ch)
                    .with_ip_pair(ip_send_p2, ip_recv_p2)
                    .build()
                    .await
                {
                    Ok(dev) => dev,
                    Err(e) => {
                        return Self::fire_error(
                            &stopped,
                            &callback,
                            format!("PQ2 exit device: {e}"),
                        );
                    }
                };
                // Entry device: real UDP, channel IP pair
                let pq2_entry = match DeviceBuilder::new()
                    .with_udp(UdpSocketFactory)
                    .with_ip_pair(ch_tx, ch_rx)
                    .build()
                    .await
                {
                    Ok(dev) => dev,
                    Err(e) => {
                        pq2_exit.stop().await;
                        return Self::fire_error(
                            &stopped,
                            &callback,
                            format!("PQ2 entry device: {e}"),
                        );
                    }
                };

                // Configure: entry with ephemeral key from phase 1, exit with device key
                let exit_initial = Peer::new(config.exit_peer.public_key.into())
                    .with_allowed_ips(config.exit_peer.allowed_ips.clone())
                    .with_endpoint(config.exit_peer.endpoint);
                let device_key = StaticSecret::from(config.private_key);

                let r1 =
                    Self::configure_device(&pq2_entry, first_key.clone(), first_peer.clone()).await;
                let r2 = Self::configure_device(&pq2_exit, device_key, exit_initial).await;
                if let Err(e) = r1.and(r2) {
                    pq2_entry.stop().await;
                    pq2_exit.stop().await;
                    return Self::fire_error(
                        &stopped,
                        &callback,
                        format!("Configure PQ phase 2: {e}"),
                    );
                }

                // Now 10.64.0.1:1337 reaches the EXIT relay's config service
                let exit_ephemeral_private = PrivateKey::new_from_random();
                let exit_ephemeral_pubkey = exit_ephemeral_private.public_key();

                let exchange_result = tokio::time::timeout(
                    pq_timeout,
                    Self::negotiate_ephemeral_peer(
                        &smoltcp_handle_p2,
                        parent_pubkey.clone(),
                        exit_ephemeral_pubkey,
                        config.enable_pq,
                        config.enable_daita,
                    ),
                )
                .await;

                pq2_entry.stop().await;
                pq2_exit.stop().await;
                if let Some((_, task)) = obfuscation_p2 {
                    task.abort();
                }

                match exchange_result {
                    Ok(Ok(exit_ephemeral)) => {
                        log::info!(
                            "PQ phase 2 complete (psk={}, daita={})",
                            exit_ephemeral.psk.is_some(),
                            exit_ephemeral.daita.is_some()
                        );

                        let exit_secret = StaticSecret::from(exit_ephemeral_private.to_bytes());
                        let mut exit_peer = Peer::new(config.exit_peer.public_key.into())
                            .with_allowed_ips(config.exit_peer.allowed_ips.clone())
                            .with_endpoint(config.exit_peer.endpoint);

                        if let Some(ref psk) = exit_ephemeral.psk {
                            exit_peer = exit_peer.with_preshared_key(*psk.as_bytes());
                        }

                        (Some((first_key, first_peer)), exit_secret, exit_peer)
                    }
                    Ok(Err(e)) => {
                        return Self::fire_error(&stopped, &callback, format!("PQ phase 2: {e}"));
                    }
                    Err(_) => {
                        stopped.store(true, Ordering::SeqCst);
                        callback.on_timeout();
                        return;
                    }
                }
            }
        } else {
            let private_key = StaticSecret::from(config.private_key);
            let peer = Peer::new(config.exit_peer.public_key.into())
                .with_allowed_ips(config.exit_peer.allowed_ips.clone())
                .with_endpoint(config.exit_peer.endpoint);
            (None, private_key, peer)
        };

        if is_stopped() {
            return;
        }

        // 4. Update LWO client key to ephemeral key if PQ was used.
        // LWO obfuscates the WireGuard handshake using the client public key.
        // After PQ, the handshake uses the ephemeral key, not the device key.
        if let ObfuscationConfig::Lwo {
            ref mut client_public_key,
            ..
        } = config.obfuscation
        {
            // The ingress device's key is what LWO needs.
            // Singlehop: pq_exit_key. Multihop: pq_entry's key.
            let ingress_key = if let Some((ref entry_key, _)) = pq_entry {
                // Multihop: entry ephemeral key
                gotatun::x25519::PublicKey::from(entry_key).to_bytes()
            } else {
                // Singlehop: exit ephemeral key (= the only key)
                gotatun::x25519::PublicKey::from(&pq_exit_key).to_bytes()
            };
            *client_public_key = ingress_key;
        }

        // Start a fresh obfuscation proxy for the final device
        let final_obfuscation = match Self::start_obfuscation_proxy(&config).await {
            Ok(ob) => ob,
            Err(e) => {
                return Self::fire_error(&stopped, &callback, format!("Final obfuscation: {e}"));
            }
        };
        if let Some((ep, _)) = &final_obfuscation {
            Self::apply_obfuscation(&mut config, *ep);
        }

        // 5. Build final device(s) with IpMux (TUN + smoltcp) — user traffic now flows
        let smoltcp_net2 = SmoltcpNetwork::new(SmoltcpNetworkConfig {
            ipv4_addr: config.ipv4_addr,
            ipv6_addr: config.ipv6_addr,
            mtu: smoltcp_mtu,
        });
        let (smoltcp_handle, ip_recv2, ip_send2, _smoltcp_guard2) = smoltcp_net2.into_parts();
        let (mux_recv, mux_send) = ip_mux(tun_dev.clone(), tun_dev, ip_recv2, ip_send2);

        let devices = if let Some(ref entry_peer_config) = config.entry_peer {
            let entry_peer = Peer::new(entry_peer_config.public_key.into())
                .with_allowed_ips(entry_peer_config.allowed_ips.clone())
                .with_endpoint(entry_peer_config.endpoint);

            let multihop_overhead = match entry_peer_config.endpoint.ip() {
                IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
                IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
            };
            let entry_mtu = MtuWatcher::new(config.mtu)
                .increase(multihop_overhead as u16)
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
                    return Self::fire_error(&stopped, &callback, format!("Exit device: {e}"));
                }
            };
            let entry_device = match DeviceBuilder::new()
                .with_udp(UdpSocketFactory)
                .with_ip_pair(tun_channel_tx, tun_channel_rx)
                .build()
                .await
            {
                Ok(dev) => dev,
                Err(e) => {
                    exit_device.stop().await;
                    return Self::fire_error(&stopped, &callback, format!("Entry device: {e}"));
                }
            };

            log::info!(
                "Multihop: entry={}, exit={}",
                entry_peer_config.endpoint,
                config.exit_peer.endpoint
            );

            // Use PQ key for entry if negotiated, otherwise original key.
            // The endpoint must match the (possibly obfuscated) config endpoint.
            let (entry_key, mut entry_peer_final) = pq_entry.unwrap_or_else(|| {
                let key = StaticSecret::from(config.private_key);
                (key, entry_peer)
            });
            // Update the peer endpoint to the obfuscated address
            entry_peer_final = entry_peer_final.with_endpoint(entry_peer_config.endpoint);

            let r1 = Self::configure_device(&entry_device, entry_key, entry_peer_final).await;
            let r2 = Self::configure_device(&exit_device, pq_exit_key, pq_exit_peer).await;
            if let Err(e) = r1.and(r2) {
                entry_device.stop().await;
                exit_device.stop().await;
                return Self::fire_error(&stopped, &callback, format!("Configure peers: {e}"));
            }

            Devices::Multihop {
                entry: entry_device,
                exit: exit_device,
            }
        } else {
            let device = match DeviceBuilder::new()
                .with_udp(UdpSocketFactory)
                .with_ip_pair(mux_send, mux_recv)
                .build()
                .await
            {
                Ok(dev) => dev,
                Err(e) => {
                    return Self::fire_error(&stopped, &callback, format!("GotaTun device: {e}"));
                }
            };
            // Update exit peer endpoint to the obfuscated address
            let pq_exit_peer = pq_exit_peer.with_endpoint(config.exit_peer.endpoint);
            if let Err(e) = Self::configure_device(&device, pq_exit_key, pq_exit_peer).await {
                device.stop().await;
                return Self::fire_error(&stopped, &callback, format!("Configure peers: {e}"));
            }
            Devices::Singlehop(device)
        };

        if is_stopped() {
            devices.stop().await;
            return;
        }

        // 5. Create pinger
        let icmp_socket = match smoltcp_handle.icmp_socket(0).await {
            Ok(socket) => socket,
            Err(e) => {
                devices.stop().await;
                return Self::fire_error(&stopped, &callback, format!("ICMP socket: {e}"));
            }
        };
        let mut pinger = SmoltcpPinger::new(icmp_socket, config.ipv4_gateway);

        // 5. Establish connectivity
        let establish_timeout = Duration::from_secs(config.establish_timeout_secs.max(1) as u64);
        log::info!("Establishing connectivity (timeout: {establish_timeout:?})");

        if let Err(e) = pinger.send_icmp().await {
            log::warn!("Initial ping failed: {e}");
        }

        let connected = tokio::select! {
            result = Self::wait_for_connectivity(&devices, &mut pinger, &stopped) => result,
            _ = tokio::time::sleep(establish_timeout) => false,
            _ = stop_notify.notified() => {
                devices.stop().await;
                return;
            },
        };

        if is_stopped() {
            devices.stop().await;
            return;
        }

        if !connected {
            devices.stop().await;
            stopped.store(true, Ordering::SeqCst);
            callback.on_timeout();
            return;
        }

        // 6. Connected!
        callback.on_connected();
        log::info!("Tunnel connected — starting ongoing monitoring");

        // 7. Ongoing monitoring
        Self::monitor_connectivity(&devices, &stopped, &stop_notify).await;

        devices.stop().await;

        if !is_stopped() {
            stopped.store(true, Ordering::SeqCst);
            callback.on_timeout();
        }
    }

    /// Run a single PQ exchange phase: start obfuscation proxy, build smoltcp-only device,
    /// configure peer, negotiate. Returns (Some(key), Some(peer)) on success, (None, None) on error.
    async fn run_pq_exchange(
        config: &TunnelConfig,
        peer_config: &PeerConfig,
        parent_pubkey: PublicKey,
        timeout: Duration,
        stopped: &AtomicBool,
        stop_notify: &Notify,
    ) -> (Option<StaticSecret>, Option<Peer>) {
        // Start a fresh obfuscation proxy for this PQ device
        let obfuscation = match Self::start_obfuscation_proxy(config).await {
            Ok(ob) => ob,
            Err(e) => {
                log::error!("PQ obfuscation proxy failed: {e}");
                return (None, None);
            }
        };

        let peer_endpoint = obfuscation
            .as_ref()
            .map(|(ep, _)| *ep)
            .unwrap_or(peer_config.endpoint);

        let smoltcp_mtu = config.mtu.saturating_sub(WIREGUARD_HEADER_SIZE);
        let smoltcp_net = SmoltcpNetwork::new(SmoltcpNetworkConfig {
            ipv4_addr: config.ipv4_addr,
            ipv6_addr: config.ipv6_addr,
            mtu: smoltcp_mtu,
        });
        let (handle, ip_recv, ip_send, _guard) = smoltcp_net.into_parts();

        let pq_device = match DeviceBuilder::new()
            .with_udp(UdpSocketFactory)
            .with_ip_pair(ip_send, ip_recv)
            .build()
            .await
        {
            Ok(dev) => dev,
            Err(e) => {
                log::error!("PQ device build failed: {e}");
                return (None, None);
            }
        };

        let private_key = StaticSecret::from(config.private_key);
        let initial_peer = Peer::new(peer_config.public_key.into())
            .with_allowed_ips(peer_config.allowed_ips.clone())
            .with_endpoint(peer_endpoint);

        if let Err(e) = Self::configure_device(&pq_device, private_key, initial_peer).await {
            pq_device.stop().await;
            log::error!("PQ device configure failed: {e}");
            return (None, None);
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
        // Stop the obfuscation proxy — each device gets a fresh one
        if let Some((_, task)) = obfuscation {
            task.abort();
        }

        match exchange_result {
            Ok(Ok(ephemeral_peer)) => {
                let ephemeral_secret = StaticSecret::from(ephemeral_private.to_bytes());
                let mut peer = Peer::new(peer_config.public_key.into())
                    .with_allowed_ips(peer_config.allowed_ips.clone())
                    .with_endpoint(peer_config.endpoint);

                if let Some(ref psk) = ephemeral_peer.psk {
                    peer = peer.with_preshared_key(*psk.as_bytes());
                }

                (Some(ephemeral_secret), Some(peer))
            }
            Ok(Err(e)) => {
                log::error!("PQ exchange failed: {e}");
                (None, None)
            }
            Err(_) => {
                log::warn!("PQ exchange timed out");
                (None, None)
            }
        }
    }

    /// Wait for rx_bytes > 0 on any peer of a device.
    async fn wait_for_rx<T: DeviceTransports>(
        device: &gotatun::device::Device<T>,
        stopped: &AtomicBool,
    ) {
        loop {
            if stopped.load(Ordering::SeqCst) {
                return;
            }
            let peers = device.read(async |d| d.peers().await).await;
            if peers.iter().any(|p| p.stats.rx_bytes > 0) {
                return;
            }
            tokio::time::sleep(CONNECTIVITY_CHECK_INTERVAL).await;
        }
    }

    /// Start a fresh obfuscation proxy for the given config.
    /// Returns the local proxy endpoint and the task handle (to keep it alive).
    /// Returns (None, None) if obfuscation is off.
    async fn start_obfuscation_proxy(
        config: &TunnelConfig,
    ) -> Result<Option<(SocketAddr, tokio::task::JoinHandle<()>)>, String> {
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
        let local_endpoint = obfuscator.endpoint();
        log::info!("Obfuscation proxy started at {local_endpoint}");
        let task = tokio::spawn(async move {
            let _ = obfuscator.run().await;
        });
        Ok(Some((local_endpoint, task)))
    }

    /// Apply obfuscation to the config: replace the ingress peer's endpoint with the proxy address.
    fn apply_obfuscation(config: &mut TunnelConfig, proxy_endpoint: SocketAddr) {
        if config.entry_peer.is_some() {
            config.entry_peer.as_mut().unwrap().endpoint = proxy_endpoint;
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

        // The connector is called once by tonic; use a Mutex to hand off the stream.
        let stream_cell = std::sync::Mutex::new(Some(stream));

        let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
        let conn = endpoint
            .connect_with_connector(service_fn(move |_| {
                let stream = stream_cell
                    .lock()
                    .unwrap()
                    .take()
                    .expect("connector called more than once");
                async move { Ok::<_, std::io::Error>(hyper_util::rt::tokio::TokioIo::new(stream)) }
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

    async fn has_rx(&self) -> bool {
        match self {
            Devices::Singlehop(dev) => {
                let peers = dev.read(async |d| d.peers().await).await;
                peers.iter().any(|p| p.stats.rx_bytes > 0)
            }
            Devices::Multihop { entry, .. } => {
                let peers = entry.read(async |d| d.peers().await).await;
                peers.iter().any(|p| p.stats.rx_bytes > 0)
            }
        }
    }

    async fn total_rx(&self) -> usize {
        match self {
            Devices::Singlehop(dev) => {
                let peers = dev.read(async |d| d.peers().await).await;
                peers.iter().map(|p| p.stats.rx_bytes).sum()
            }
            Devices::Multihop { entry, .. } => {
                let peers = entry.read(async |d| d.peers().await).await;
                peers.iter().map(|p| p.stats.rx_bytes).sum()
            }
        }
    }
}
