//! Inline LWO obfuscation for GotaTun via [`UdpTransportFactory`] wrappers.
//!
//! Instead of routing packets through a localhost proxy, obfuscation is applied
//! in-process: [`LwoSend`] XOR-obfuscates each outgoing packet and [`LwoRecv`]
//! de-obfuscates each incoming packet.  [`LwoUdpTransportFactory`] wires them together.

use std::{io, net::SocketAddr};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams},
};
use tunnel_obfuscation::lwo;

/// A [`UdpSend`] wrapper that LWO-obfuscates every outgoing packet before forwarding it to the
/// inner sender.
///
/// `tx_key` must be the **server** public key (the key used by the relay to deobfuscate).
#[derive(Clone)]
pub struct LwoSend<S: UdpSend + Clone> {
    inner: S,
    tx_key: [u8; 32],
}

impl<S: UdpSend + Clone> UdpSend for LwoSend<S> {
    type SendManyBuf = S::SendManyBuf;

    async fn send_to(&self, mut packet: Packet, destination: SocketAddr) -> io::Result<()> {
        lwo::obfuscate_thread_local(&mut packet, &self.tx_key);
        self.inner.send_to(packet, destination).await
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        self.inner.max_number_of_packets_to_send()
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        // Obfuscate every packet before handing the batch to the inner sender. The caller
        // (`BufferedUdpSend`) drops any packets remaining in `packets` on error, so there is no
        // risk of an obfuscated packet being re-submitted and double-XORed.
        obfuscate_all(packets, &self.tx_key);
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
/// inner receiver.
///
/// `rx_key` must be the **client** public key (the key used by the client to deobfuscate).
pub struct LwoRecv<R: UdpRecv> {
    inner: R,
    rx_key: [u8; 32],
}

impl<R: UdpRecv> UdpRecv for LwoRecv<R> {
    type RecvManyBuf = R::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let (mut packet, addr) = self.inner.recv_from(pool).await?;
        lwo::deobfuscate(&mut packet, &self.rx_key);
        Ok((packet, addr))
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        // The trait contract appends to `packets`; only deobfuscate the entries the inner call
        // actually added so we don't touch packets the caller already owned.
        let start = packets.len();
        self.inner.recv_many_from(recv_buf, pool, packets).await?;
        deobfuscate_all(&mut packets[start..], &self.rx_key);
        Ok(())
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        self.inner.enable_udp_gro()
    }
}

/// Apply LWO obfuscation to every packet in `packets`.
fn obfuscate_all(packets: &mut [(Packet, SocketAddr)], tx_key: &[u8; 32]) {
    for (packet, _) in packets.iter_mut() {
        lwo::obfuscate_thread_local(packet, tx_key);
    }
}

/// Apply LWO deobfuscation to every packet in `packets`.
fn deobfuscate_all(packets: &mut [(Packet, SocketAddr)], rx_key: &[u8; 32]) {
    for (packet, _) in packets.iter_mut() {
        lwo::deobfuscate(packet, rx_key);
    }
}

/// A [`UdpTransportFactory`] that wraps another factory and applies LWO obfuscation inline.
///
/// * `tx_key` - server public key bytes, used to obfuscate outgoing packets.
/// * `rx_key` - client public key bytes, used to deobfuscate incoming packets.
pub struct LwoUdpTransportFactory<F: UdpTransportFactory> {
    pub inner: F,
    pub tx_key: [u8; 32],
    pub rx_key: [u8; 32],
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
                    tx_key: self.tx_key,
                },
                LwoRecv {
                    inner: recv_v4,
                    rx_key: self.rx_key,
                },
            ),
            (
                LwoSend {
                    inner: send_v6,
                    tx_key: self.tx_key,
                },
                LwoRecv {
                    inner: recv_v6,
                    rx_key: self.rx_key,
                },
            ),
        ))
    }
}
