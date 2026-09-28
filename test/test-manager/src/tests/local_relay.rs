//! Create and manage a local WireGuard relay.

use anyhow::{Context, Result};
use futures::{Stream, stream};
use gotatun::{
    device::{self, Peer},
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
    udp::socket::UdpSocketFactory,
};
use ipnetwork::Ipv4Network;
use mullvad_types::CustomTunnelEndpoint;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use talpid_types::net::wireguard;
use tokio::sync::broadcast;

use crate::{
    network_monitor::{self, PacketMonitor, ParsedPacket},
    tests::config::TEST_CONFIG,
};

// Private key of the wireguard remote peer on host.
data_encoding_macro::base64_array!(
    "const CUSTOM_TUN_REMOTE_PRIVKEY" = "gLvQuyqazziyf+pUCAFUgTnWIwn6fPE5MOReOqPEGHU="
);
// Public key of the wireguard remote peer on host.
data_encoding_macro::base64_array!(
    "const CUSTOM_TUN_REMOTE_PUBKEY" = "7svBwGBefP7KVmH/yes+pZCfO6uSOYeGieYYa1+kZ0E="
);
// Public key of the wireguard local peer on guest.
data_encoding_macro::base64_array!(
    "const CUSTOM_TUN_LOCAL_PUBKEY" = "h6elqt3dfamtS/p9jxJ8bIYs8UW9YHfTFhvx0fabTFo="
);
// Private key of the wireguard local peer on guest.
data_encoding_macro::base64_array!(
    "const CUSTOM_TUN_LOCAL_PRIVKEY" = "mPue6Xt0pdz4NRAhfQSp/SLKo7kV7DW+2zvBq0N9iUI="
);

/// Port of the wireguard remote peer.
const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer.
const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);

/// The maximum size of packets sent to the guest.
const MTU: u16 = 1420;

/// Capacity of the channel carrying packets decrypted by the relay.
const DECRYPTED_PACKET_CHANNEL_CAPACITY: usize = 1024;

/// A WireGuard device relaying packets via in-process channels instead of a TUN device.
type RelayDevice = device::Device<(UdpSocketFactory, DecryptedPacketSink, GuestPacketSource)>;

/// A WireGuard peer on the host, standing in for a Mullvad relay.
///
/// This relay does not support PQ handshakes, etc.
///
/// The daemon connects to it using [`LocalRelay::custom_tunnel_endpoint`].
///
/// Decrypted packets are never routed anywhere. They can only be observed using
/// [`LocalRelay::monitor_until`].
///
/// If this value is dropped, the relay shuts down.
pub struct LocalRelay {
    /// Kept for its `Drop` impl, shutting the device down when the relay is dropped.
    _device: RelayDevice,
    /// Packets decrypted by the relay, i.e. what the guest sent inside the tunnel.
    decrypted_packets: DecryptedPacketSink,
}

impl LocalRelay {
    /// Tunnel address of the relay. This is the gateway and default DNS resolver of the tunnel.
    pub const GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);

    pub async fn start() -> Result<Self> {
        log::debug!("Starting local WireGuard relay");

        let peer = Peer::new(CUSTOM_TUN_LOCAL_PUBKEY.into()).with_allowed_ip(
            const { Ipv4Network::new_checked(CUSTOM_TUN_LOCAL_TUN_ADDR, 32).unwrap() }.into(),
        );

        let (tx, _) = broadcast::channel(DECRYPTED_PACKET_CHANNEL_CAPACITY);
        let decrypted_packets = DecryptedPacketSink { tx };

        let device = device::build()
            .with_default_udp()
            .with_ip_pair(
                decrypted_packets.clone(),
                GuestPacketSource {
                    mtu: MtuWatcher::new(MTU),
                },
            )
            .with_private_key(CUSTOM_TUN_REMOTE_PRIVKEY.into())
            .with_peer(peer)
            .with_listen_port(CUSTOM_TUN_REMOTE_REAL_PORT)
            .build()
            .await
            .context("Failed to create gotatun device")?;

        Ok(LocalRelay {
            _device: device,
            decrypted_packets,
        })
    }

    /// Endpoint which the daemon should use to connect to this relay.
    pub fn custom_tunnel_endpoint(&self) -> CustomTunnelEndpoint {
        let peer_addr = SocketAddr::new(
            IpAddr::V4(TEST_CONFIG.host_bridge_ip),
            CUSTOM_TUN_REMOTE_REAL_PORT,
        );

        CustomTunnelEndpoint {
            host: peer_addr.ip().to_string(),
            config: wireguard::ConnectionConfig {
                tunnel: wireguard::TunnelConfig {
                    addresses: vec![IpAddr::V4(CUSTOM_TUN_LOCAL_TUN_ADDR)],
                    private_key: wireguard::PrivateKey::from(CUSTOM_TUN_LOCAL_PRIVKEY),
                },
                peer: wireguard::PeerConfig {
                    public_key: wireguard::PublicKey::from(CUSTOM_TUN_REMOTE_PUBKEY),
                    allowed_ips: vec!["0.0.0.0/0".parse().unwrap()],
                    endpoint: peer_addr,
                    psk: None,
                    constant_packet_size: false,
                },
                ipv4_gateway: Self::GATEWAY,
                exit_peer: None,
                routes: None,
                #[cfg(target_os = "linux")]
                fwmark: None,
                ipv6_gateway: None,
            },
        }
    }

    /// Monitor packets that the guest sends inside the tunnel.
    ///
    /// Only packets decrypted after this call are observed.
    pub async fn monitor_until(
        &self,
        filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
        should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
        timeout: Option<Duration>,
    ) -> PacketMonitor {
        network_monitor::spawn_monitor(
            "tunnel",
            self.decrypted_packets.packets(),
            filter_fn,
            should_continue_fn,
            timeout,
        )
        .await
    }
}

/// gotatun IP sink which publishes all packets received from the guest to
/// [`LocalRelay::decrypted_packets`].
#[derive(Clone)]
struct DecryptedPacketSink {
    tx: broadcast::Sender<ParsedPacket>,
}

impl DecryptedPacketSink {
    /// Stream of packets the guest sends inside the tunnel.
    ///
    /// Only packets decrypted after this call are observed.
    fn packets(&self) -> impl Stream<Item = ParsedPacket> + Send + 'static {
        stream::unfold(self.tx.subscribe(), async |mut rx| {
            loop {
                match rx.recv().await {
                    Ok(packet) => return Some((packet, rx)),
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        log::warn!("Tunnel packet monitor skipped {count} packets");
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        })
    }
}

impl IpSend for DecryptedPacketSink {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        if let Some(parsed_packet) = network_monitor::parse_ip_packet(packet.into_bytes().as_ref())
        {
            // Getting a send error just means there are no subscribers, which is fine.
            let _ = self.tx.send(parsed_packet);
        }
        Ok(())
    }
}

/// gotatun IP source which never yields any packets, since the relay never sends anything to
/// the guest.
struct GuestPacketSource {
    mtu: MtuWatcher,
}

impl IpRecv for GuestPacketSource {
    async fn recv<'a>(
        &'a mut self,
        _pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        std::future::pending::<io::Result<std::iter::Empty<_>>>().await
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}
