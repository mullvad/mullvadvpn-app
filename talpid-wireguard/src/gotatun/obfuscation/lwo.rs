//! LWO obfuscation/deobfuscation wrappers for GotaTun UDP transports.
//!
//! LWO v2 also changes the timing of some WireGuard timers; see [`lwo_timer_params`]. These must
//! be applied to the [`gotatun::device::Peer`] of LWO v2 peers.
// TODO: v1 support can be removed when all servers support v2

use std::{io, net::SocketAddr, time::Duration};

use gotatun::{
    noise::TimerParams,
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use talpid_types::net::obfuscation::{LwoVersion, ObfuscatorConfig, Obfuscators};
use tunnel_obfuscation::lwo::{
    self,
    v2::{self, Verdict},
};

use crate::config::Config;

/// The keys used to obfuscate traffic.
#[derive(Clone, Copy)]
pub enum LwoKeys {
    /// v1 obfuscates each direction with a different key.
    V1 {
        /// The server public key, used to obfuscate outgoing packets.
        tx_key: [u8; 32],
        /// The client public key, used to deobfuscate incoming packets.
        rx_key: [u8; 32],
    },
    /// v2 obfuscates both directions with the server public key.
    V2 { key: [u8; 32] },
}

/// WireGuard timer tuning for LWO v2 peers.
pub fn lwo_timer_params() -> TimerParams {
    TimerParams {
        // Passive keepalive: 10 s +/- 2 s
        keepalive_timeout: Duration::from_secs(8)..=Duration::from_secs(12),
        // New handshake after silence: 15 s +/- 2 s
        new_handshake_timeout: Duration::from_secs(13)..=Duration::from_secs(17),
        // Handshake retransmit: 5 s +/- 250 ms
        rekey_timeout: Duration::from_millis(4750)..=Duration::from_millis(5250),
        // Rekey-after-time: only ever moved earlier
        rekey_after_time: Duration::from_secs(100)..=Duration::from_secs(120),
    }
}

/// The LWO version the tunnel config connects with, if it uses LWO at all.
///
/// [`LwoVersion::V2`] peers must have [`lwo_timer_params`] applied.
pub fn lwo_version(config: &Config) -> Option<LwoVersion> {
    match &config.obfuscator_config {
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { version, .. })) => Some(*version),
        _ => None,
    }
}

/// Pad (v2 handshakes only) and obfuscate an outgoing packet in place.
fn pad_and_obfuscate(packet: &mut Packet, keys: &LwoKeys) {
    match keys {
        LwoKeys::V1 { tx_key, .. } => lwo::obfuscate_thread_local(packet, tx_key),
        LwoKeys::V2 { key } => {
            if let Some(padded) = v2::pad(packet) {
                let buf = packet.buf_mut();
                buf.clear();
                buf.extend_from_slice(&padded);
            }
            v2::obfuscate(packet, key)
        }
    }
}

/// Deobfuscate an incoming packet in place, trimming v2 handshake padding.
///
/// Returns `false` if the packet is invalid and must be dropped. Only v2 validates packets;
/// v1 accepts everything.
fn deobfuscate_and_trim(packet: &mut Packet, keys: &LwoKeys) -> bool {
    match keys {
        LwoKeys::V1 { rx_key, .. } => {
            lwo::deobfuscate(packet, rx_key);
            true
        }
        LwoKeys::V2 { key } => match v2::deobfuscate(packet, key) {
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
        },
    }
}

/// A [`UdpSend`] wrapper that LWO-obfuscates every outgoing packet before forwarding it to the
/// inner sender.
#[derive(Clone)]
pub struct LwoSend<S: UdpSend> {
    inner: S,
    keys: LwoKeys,
    endpoint: SocketAddr,
}

impl<S: UdpSend> UdpSend for LwoSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, mut packet: Packet, _destination: SocketAddr) -> io::Result<()> {
        pad_and_obfuscate(&mut packet, &self.keys);
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
            pad_and_obfuscate(packet, &self.keys);
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
    keys: LwoKeys,
}

impl<R: UdpRecv> UdpRecv for LwoRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        loop {
            let (mut packet, addr) = self.inner.recv_from(pool).await?;
            if deobfuscate_and_trim(&mut packet, &self.keys) {
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
            let keep = index < start || deobfuscate_and_trim(packet, &self.keys);
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
/// * `keys` - the obfuscation keys, which also select the protocol version.
/// * `endpoint` - endpoint to forward traffic to.
pub struct LwoUdpTransportFactory<F: UdpTransportFactory> {
    pub inner: F,
    pub keys: LwoKeys,
    pub endpoint: SocketAddr,
}

impl<F: UdpTransportFactory> UdpTransportFactory for LwoUdpTransportFactory<F> {
    type Send = LwoSend<F::Send>;
    type Recv = LwoRecv<F::Recv>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let (send, recv) = self.inner.bind(params).await?;
        Ok((
            LwoSend {
                inner: send,
                keys: self.keys,
                endpoint: self.endpoint,
            },
            LwoRecv {
                inner: recv,
                keys: self.keys,
            },
        ))
    }
}
