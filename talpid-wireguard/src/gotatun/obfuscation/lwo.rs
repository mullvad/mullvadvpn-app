//! LWO v2 obfuscation/deobfuscation wrappers for GotaTun UDP transports.
//!
//! LWO also changes the timing of some WireGuard timers; see [`lwo_timer_params`]. These must
//! be applied to the [`gotatun::device::Peer`] of LWO peers.

use std::{io, net::SocketAddr};

use gotatun::{
    noise::TimerParams,
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use rand::RngCore;
use talpid_types::net::obfuscation::{ObfuscatorConfig, Obfuscators};
use tunnel_obfuscation::lwo::v2::{self, Verdict, timers};

use crate::config::Config;

/// WireGuard timer tuning for LWO V2 peers.
pub fn lwo_timer_params() -> TimerParams {
    TimerParams {
        keepalive_timeout: timers::KEEPALIVE_TIMEOUT,
        new_handshake_timeout: timers::NEW_HANDSHAKE_TIMEOUT,
        rekey_timeout: timers::REKEY_TIMEOUT,
        rekey_after_time: timers::REKEY_AFTER_TIME,
    }
}

/// Extract LWO obfuscation settings from the tunnel config.
///
/// Returns the obfuscation key (the **server** public key, used in both directions in LWO v2)
/// and the endpoint to forward traffic to.
pub fn lwo_config(config: &Config) -> Option<([u8; 32], SocketAddr)> {
    match &config.obfuscator_config {
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { endpoint })) => {
            let key = *config.entry_peer.public_key.as_bytes();
            Some((key, *endpoint))
        }
        _ => None,
    }
}

/// Pad (handshakes only) and obfuscate an outgoing packet in place.
fn pad_and_obfuscate(packet: &mut Packet, key: &[u8; 32]) {
    let mut rng = rand::rng();
    let padding = v2::padding_len(packet, &mut rng);
    if padding > 0 {
        let buf = packet.buf_mut();
        let packet_len = buf.len();
        buf.resize(packet_len + padding, 0);
        rng.fill_bytes(&mut buf[packet_len..]);
    }
    v2::obfuscate(packet, key);
}

/// Validate and deobfuscate an incoming packet in place, trimming handshake padding.
///
/// Returns `false` if the packet is invalid and must be dropped.
fn deobfuscate_and_trim(packet: &mut Packet, key: &[u8; 32]) -> bool {
    match v2::deobfuscate(packet, key) {
        Verdict::Plain => true,
        Verdict::Lwo { trim_to } => {
            if let Some(len) = trim_to {
                packet.truncate(len);
            }
            true
        }
        Verdict::Invalid => {
            if cfg!(debug_assertions) {
                log::trace!("Dropping invalid LWO packet");
            }
            false
        }
    }
}

/// A [`UdpSend`] wrapper that LWO-obfuscates every outgoing packet before forwarding it to the
/// inner sender.
#[derive(Clone)]
pub struct LwoSend<S: UdpSend> {
    inner: S,
    /// The server public key.
    key: [u8; 32],
    endpoint: SocketAddr,
}

impl<S: UdpSend> UdpSend for LwoSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, mut packet: Packet, _destination: SocketAddr) -> io::Result<()> {
        pad_and_obfuscate(&mut packet, &self.key);
        self.inner.send_to(packet, self.endpoint).await
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        self.inner.max_number_of_packets_to_send()
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        for (packet, _) in packets.iter_mut() {
            pad_and_obfuscate(packet, &self.key);
        }
        self.inner.send_many_to(send_buf, packets).await
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        self.inner.local_addr()
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        self.inner.set_fwmark(mark)
    }
}

/// A [`UdpRecv`] wrapper that LWO-deobfuscates every incoming packet after receiving it from the
/// inner receiver. Packets that fail LWO validation are dropped.
pub struct LwoRecv<R: UdpRecv> {
    inner: R,
    /// The server public key.
    key: [u8; 32],
}

impl<R: UdpRecv> UdpRecv for LwoRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        loop {
            let (mut packet, addr) = self.inner.recv_from(pool).await?;
            if deobfuscate_and_trim(&mut packet, &self.key) {
                return Ok((packet, addr));
            }
        }
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        // The trait contract appends to `packets`; only touch the entries the inner call
        // actually added so we don't process packets the caller already owned.
        let start = packets.len();
        self.inner.recv_many_from(recv_buf, pool, packets).await?;

        let mut index = 0;
        packets.retain_mut(|(packet, _addr)| {
            let keep = index < start || deobfuscate_and_trim(packet, &self.key);
            index += 1;
            keep
        });

        Ok(())
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        self.inner.enable_udp_gro()
    }
}

/// A [`UdpTransportFactory`] that wraps another factory and applies LWO obfuscation inline.
///
/// * `key` - server public key bytes, used to obfuscate and deobfuscate in both directions.
/// * `endpoint` - endpoint to forward traffic to.
pub struct LwoUdpTransportFactory<F: UdpTransportFactory> {
    pub inner: F,
    pub key: [u8; 32],
    pub endpoint: SocketAddr,
}

impl<F: UdpTransportFactory> UdpTransportFactory for LwoUdpTransportFactory<F> {
    type SendV4 = LwoSend<F::SendV4>;
    type SendV6 = LwoSend<F::SendV6>;
    type RecvV4 = LwoRecv<F::RecvV4>;
    type RecvV6 = LwoRecv<F::RecvV6>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        let ((send_v4, recv_v4), (send_v6, recv_v6)) = self.inner.bind(params).await?;
        Ok((
            (
                LwoSend {
                    inner: send_v4,
                    key: self.key,
                    endpoint: self.endpoint,
                },
                LwoRecv {
                    inner: recv_v4,
                    key: self.key,
                },
            ),
            (
                LwoSend {
                    inner: send_v6,
                    key: self.key,
                    endpoint: self.endpoint,
                },
                LwoRecv {
                    inner: recv_v6,
                    key: self.key,
                },
            ),
        ))
    }
}
