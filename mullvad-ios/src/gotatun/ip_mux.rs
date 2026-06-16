//! Multiplexer for [`IpSend`]/[`IpRecv`] that combines traffic from a primary
//! (TUN device) and secondary (smoltcp) source.
//!
//! The [`IpMux`] tracks outbound connections from the secondary interface
//! (TCP by 4-tuple, ICMP by identifier) and routes matching return traffic
//! back to it. All other traffic goes to the primary interface.

use super::connection_tracker::{ConnectionTracker, ConnectionTrackerEvent, outbound_event};
use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use std::io;
use tokio::sync::mpsc;
use zerocopy::IntoBytes;

/// Capacity of the channel carrying connection-tracking events from the receive
/// (writer) half to the send (reader) half. This is control-plane only (a SYN /
/// FIN per smoltcp connection, an echo request every few seconds), so a modest
/// buffer is generous.
const EVENT_CHANNEL_CAPACITY: usize = 128;

/// The receive half of an [`IpMux`]. Merges packets from primary and secondary
/// [`IpRecv`] sources, forwarding secondary outbound connection events to the
/// send half so it can route the return traffic.
pub struct IpMuxRecv<P: IpRecv, S: IpRecv> {
    primary: P,
    secondary: S,
    events_tx: mpsc::Sender<ConnectionTrackerEvent>,
    /// Throwaway pool handed to the secondary's `recv`. The secondary (smoltcp)
    /// ignores the pool and allocates its own buffers, so this exists only to
    /// satisfy the `IpRecv::recv` signature and to keep `pool` from being
    /// borrowed by both branches of the `select!` below.
    secondary_pool: PacketBufPool,
}

/// The send half of an [`IpMux`]. Routes return traffic to the secondary
/// interface if it matches a tracked connection, otherwise to the primary.
///
/// Owns the [`ConnectionTracker`] outright — no shared state. It applies the
/// events streamed from [`IpMuxRecv`] before each lookup.
pub struct IpMuxSend<P: IpSend, S: IpSend> {
    primary: P,
    secondary: S,
    tracker: ConnectionTracker,
    events_rx: mpsc::Receiver<ConnectionTrackerEvent>,
}

/// Create a matched pair of [`IpMuxRecv`] and [`IpMuxSend`].
pub fn ip_mux<PR, SR, PS, SS>(
    primary_recv: PR,
    primary_send: PS,
    secondary_recv: SR,
    secondary_send: SS,
) -> (IpMuxRecv<PR, SR>, IpMuxSend<PS, SS>)
where
    PR: IpRecv,
    SR: IpRecv,
    PS: IpSend,
    SS: IpSend,
{
    let (events_tx, events_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    (
        IpMuxRecv {
            primary: primary_recv,
            secondary: secondary_recv,
            events_tx,
            secondary_pool: PacketBufPool::new(1),
        },
        IpMuxSend {
            primary: primary_send,
            secondary: secondary_send,
            tracker: ConnectionTracker::default(),
            events_rx,
        },
    )
}

impl<P: IpRecv, S: IpRecv> IpRecv for IpMuxRecv<P, S> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        // Race the two sources. The secondary gets its own `secondary_pool`
        // (which it ignores anyway), so neither branch borrows `pool` twice and
        // a plain `select!` suffices.
        //
        // The bias is deliberately toward the secondary, even though the primary
        // (TUN) carries the bulk of the traffic. `biased` polls the branches in
        // order on every call, and each `recv` returns as soon as one branch is
        // ready, so the order only decides the winner when both have data at the
        // same instant; the loser's future is dropped, but its data is not lost
        // (the TUN fd and the smoltcp mpsc are both cancel-safe and keep it
        // buffered for the next call).
        //
        // Given that, polling the primary first would starve the secondary: when
        // the TUN is saturated `self.primary.recv()` is ready on essentially
        // every poll, so it would win every tie and the low-volume smoltcp
        // control traffic (connectivity pings, the negotiation stack) would never
        // get serviced — precisely when we most need the monitor to keep working.
        // Polling the secondary first costs only a single cheap mpsc poll that
        // almost always returns `Pending` (so the primary is not meaningfully
        // delayed), while guaranteeing the scarce control stream is never buried
        // under the bulk data stream.
        let result = tokio::select! {
            biased;
            result = self.secondary.recv(&mut self.secondary_pool) => RecvResult::Secondary(result),
            result = self.primary.recv(pool) => RecvResult::Primary(result),
        };

        match result {
            RecvResult::Primary(result) => result.map(|iter| {
                let packets: Vec<_> = iter.collect();
                MuxIter(packets.into_iter())
            }),
            RecvResult::Secondary(result) => {
                let iter = result?;
                let packets: Vec<_> = iter.collect();
                for pkt in &packets {
                    if let Some(event) = outbound_event((*pkt).as_bytes())
                        && self.events_tx.try_send(event).is_err()
                    {
                        log::warn!(
                            "ip_mux: connection-tracker event channel full or closed, \
                                 dropping event"
                        );
                    }
                }
                Ok(MuxIter(packets.into_iter()))
            }
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.primary.mtu()
    }
}

enum RecvResult<P, S> {
    Primary(P),
    Secondary(S),
}

impl<P: IpSend, S: IpSend> IpSend for IpMuxSend<P, S> {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        // Apply any pending connection events first. A connection is registered
        // before its SYN is transmitted, so by the time its return traffic
        // arrives here (a round trip later) the event is already queued.
        while let Ok(event) = self.events_rx.try_recv() {
            self.tracker.apply(event);
        }

        let is_secondary = self.tracker.is_secondary_return((*packet).as_bytes());

        if is_secondary {
            self.secondary.send(packet).await
        } else {
            self.primary.send(packet).await
        }
    }
}

/// Wrapper iterator over collected packets from either source.
struct MuxIter(std::vec::IntoIter<Packet<Ip>>);

impl Iterator for MuxIter {
    type Item = Packet<Ip>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use gotatun::packet::PacketBufPool;
    use tokio::sync::mpsc;

    // --- Test helpers: channel-backed IpSend/IpRecv ---

    struct ChannelIpRecv {
        rx: mpsc::Receiver<Vec<u8>>,
        mtu: MtuWatcher,
    }

    impl IpRecv for ChannelIpRecv {
        async fn recv<'a>(
            &'a mut self,
            _pool: &mut PacketBufPool,
        ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
            let raw = self
                .rx
                .recv()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "closed"))?;
            let pkt = Packet::from_bytes(BytesMut::from(raw.as_slice()));
            let ip = pkt
                .try_into_ip()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            Ok(std::iter::once(ip))
        }
        fn mtu(&self) -> MtuWatcher {
            self.mtu.clone()
        }
    }

    struct ChannelIpSend {
        tx: mpsc::Sender<Vec<u8>>,
    }

    impl IpSend for ChannelIpSend {
        async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
            let raw: Packet<[u8]> = packet.into_bytes();
            let bytes: &[u8] = &*raw;
            self.tx
                .send(bytes.to_vec())
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
    }

    fn channel_pair() -> (mpsc::Sender<Vec<u8>>, ChannelIpRecv) {
        let (tx, rx) = mpsc::channel(64);
        (
            tx,
            ChannelIpRecv {
                rx,
                mtu: MtuWatcher::new(1420),
            },
        )
    }

    fn channel_send_pair() -> (ChannelIpSend, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(64);
        (ChannelIpSend { tx }, rx)
    }

    // --- Packet construction helpers ---

    /// Build a minimal IPv4 TCP packet with given addresses, ports, and flags.
    fn make_tcp_packet(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        flags: u8,
    ) -> Vec<u8> {
        let mut pkt = vec![0u8; 40]; // 20 IPv4 + 20 TCP
        // IPv4 header
        pkt[0] = 0x45; // version=4, ihl=5
        pkt[2..4].copy_from_slice(&40u16.to_be_bytes()); // total length
        pkt[8] = 64; // TTL
        pkt[9] = 6; // protocol: TCP
        pkt[12..16].copy_from_slice(&src_ip);
        pkt[16..20].copy_from_slice(&dst_ip);
        // TCP header
        pkt[20..22].copy_from_slice(&src_port.to_be_bytes());
        pkt[22..24].copy_from_slice(&dst_port.to_be_bytes());
        pkt[32] = 0x50; // data offset = 5 words
        pkt[33] = flags;
        pkt
    }

    /// Build a minimal IPv4 ICMP echo request/reply.
    fn make_icmp_packet(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        icmp_type: u8,
        ident: u16,
        seq: u16,
    ) -> Vec<u8> {
        let mut pkt = vec![0u8; 28]; // 20 IPv4 + 8 ICMP
        pkt[0] = 0x45;
        pkt[2..4].copy_from_slice(&28u16.to_be_bytes());
        pkt[8] = 64;
        pkt[9] = 1; // protocol: ICMP
        pkt[12..16].copy_from_slice(&src_ip);
        pkt[16..20].copy_from_slice(&dst_ip);
        // ICMP
        pkt[20] = icmp_type;
        pkt[21] = 0; // code
        pkt[24..26].copy_from_slice(&ident.to_be_bytes());
        pkt[26..28].copy_from_slice(&seq.to_be_bytes());
        pkt
    }

    /// Send raw bytes into a channel, to be received by a ChannelIpRecv.
    async fn inject(tx: &mpsc::Sender<Vec<u8>>, pkt: Vec<u8>) {
        tx.send(pkt).await.unwrap();
    }

    /// Receive raw bytes from a channel (output of ChannelIpSend).
    async fn drain(rx: &mut mpsc::Receiver<Vec<u8>>) -> Vec<u8> {
        tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout")
            .expect("channel closed")
    }

    // --- Tests ---

    /// Secondary TCP SYN should be tracked, and the corresponding SYN-ACK
    /// (return traffic) should be routed to the secondary IpSend.
    #[tokio::test]
    async fn tcp_return_traffic_routes_to_secondary() {
        let (_primary_inject, primary_recv) = channel_pair();
        let (secondary_inject, secondary_recv) = channel_pair();
        let (primary_send, mut primary_out) = channel_send_pair();
        let (secondary_send, mut secondary_out) = channel_send_pair();

        let (mut mux_recv, mut mux_send) =
            ip_mux(primary_recv, primary_send, secondary_recv, secondary_send);

        let mut pool = PacketBufPool::new(4);

        // 1) Secondary sends a TCP SYN
        let syn = make_tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        inject(&secondary_inject, syn).await;

        // Receive it through the mux (this registers the connection)
        let iter = mux_recv.recv(&mut pool).await.unwrap();
        let _: Vec<_> = iter.collect();

        // 2) A SYN-ACK comes back (from 1.1.1.1:1337 to 10.0.0.1:49152)
        let syn_ack = make_tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x12);
        let pkt = Packet::from_bytes(BytesMut::from(syn_ack.as_slice()))
            .try_into_ip()
            .unwrap();
        mux_send.send(pkt).await.unwrap();

        // It should be routed to secondary, not primary
        let routed = drain(&mut secondary_out).await;
        assert_eq!(routed[9], 6, "Should be TCP");
        assert_eq!(&routed[12..16], &[1, 1, 1, 1]);
        assert_eq!(&routed[16..20], &[10, 0, 0, 1]);

        // Primary should have received nothing
        assert!(primary_out.try_recv().is_err());
    }

    /// Traffic not matching any secondary connection should go to primary.
    #[tokio::test]
    async fn untracked_traffic_routes_to_primary() {
        let (_primary_inject, primary_recv) = channel_pair();
        let (_secondary_inject, secondary_recv) = channel_pair();
        let (primary_send, mut primary_out) = channel_send_pair();
        let (secondary_send, mut secondary_out) = channel_send_pair();

        let (_mux_recv, mut mux_send) =
            ip_mux(primary_recv, primary_send, secondary_recv, secondary_send);

        // Send a random TCP packet that wasn't tracked
        let pkt_bytes = make_tcp_packet([8, 8, 8, 8], [10, 0, 0, 1], 443, 12345, 0x10);
        let pkt = Packet::from_bytes(BytesMut::from(pkt_bytes.as_slice()))
            .try_into_ip()
            .unwrap();
        mux_send.send(pkt).await.unwrap();

        // Should go to primary
        let routed = drain(&mut primary_out).await;
        assert_eq!(&routed[12..16], &[8, 8, 8, 8]);

        // Secondary should be empty
        assert!(secondary_out.try_recv().is_err());
    }

    /// ICMP echo requests from secondary should be tracked by identifier,
    /// and echo replies routed back to secondary.
    #[tokio::test]
    async fn icmp_echo_reply_routes_to_secondary() {
        let (_primary_inject, primary_recv) = channel_pair();
        let (secondary_inject, secondary_recv) = channel_pair();
        let (primary_send, mut primary_out) = channel_send_pair();
        let (secondary_send, mut secondary_out) = channel_send_pair();

        let (mut mux_recv, mut mux_send) =
            ip_mux(primary_recv, primary_send, secondary_recv, secondary_send);

        let mut pool = PacketBufPool::new(4);

        // Secondary sends ICMP echo request (type 8) with ident=0xABCD
        let echo_req = make_icmp_packet([10, 0, 0, 1], [1, 1, 1, 1], 8, 0xABCD, 1);
        inject(&secondary_inject, echo_req).await;

        // Receive through mux to register
        let iter = mux_recv.recv(&mut pool).await.unwrap();
        let _: Vec<_> = iter.collect();

        // ICMP echo reply comes back (type 0, same ident)
        let echo_reply = make_icmp_packet([1, 1, 1, 1], [10, 0, 0, 1], 0, 0xABCD, 1);
        let pkt = Packet::from_bytes(BytesMut::from(echo_reply.as_slice()))
            .try_into_ip()
            .unwrap();
        mux_send.send(pkt).await.unwrap();

        // Should go to secondary
        let routed = drain(&mut secondary_out).await;
        assert_eq!(routed[9], 1, "Should be ICMP");
        assert_eq!(routed[20], 0, "Should be echo reply");

        assert!(primary_out.try_recv().is_err());
    }

    /// After a TCP FIN from secondary, the connection should be untracked
    /// and subsequent return traffic goes to primary.
    #[tokio::test]
    async fn tcp_fin_untracks_connection() {
        let (_primary_inject, primary_recv) = channel_pair();
        let (secondary_inject, secondary_recv) = channel_pair();
        let (primary_send, mut primary_out) = channel_send_pair();
        let (secondary_send, mut secondary_out) = channel_send_pair();

        let (mut mux_recv, mut mux_send) =
            ip_mux(primary_recv, primary_send, secondary_recv, secondary_send);

        let mut pool = PacketBufPool::new(4);

        // SYN
        let syn = make_tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 50000, 80, 0x02);
        inject(&secondary_inject, syn).await;
        let _: Vec<_> = mux_recv.recv(&mut pool).await.unwrap().collect();

        // FIN
        let fin = make_tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 50000, 80, 0x01);
        inject(&secondary_inject, fin).await;
        let _: Vec<_> = mux_recv.recv(&mut pool).await.unwrap().collect();

        // Return traffic should now go to primary (connection untracked)
        let reply = make_tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 80, 50000, 0x10);
        let pkt = Packet::from_bytes(BytesMut::from(reply.as_slice()))
            .try_into_ip()
            .unwrap();
        mux_send.send(pkt).await.unwrap();

        let routed = drain(&mut primary_out).await;
        assert_eq!(&routed[12..16], &[1, 1, 1, 1]);
        assert!(secondary_out.try_recv().is_err());
    }

    /// After a TCP RST from secondary, the connection should be untracked.
    #[tokio::test]
    async fn tcp_rst_untracks_connection() {
        let (_primary_inject, primary_recv) = channel_pair();
        let (secondary_inject, secondary_recv) = channel_pair();
        let (primary_send, mut primary_out) = channel_send_pair();
        let (secondary_send, mut secondary_out) = channel_send_pair();

        let (mut mux_recv, mut mux_send) =
            ip_mux(primary_recv, primary_send, secondary_recv, secondary_send);

        let mut pool = PacketBufPool::new(4);

        // SYN
        let syn = make_tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 50000, 80, 0x02);
        inject(&secondary_inject, syn).await;
        let _: Vec<_> = mux_recv.recv(&mut pool).await.unwrap().collect();

        // RST
        let rst = make_tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 50000, 80, 0x04);
        inject(&secondary_inject, rst).await;
        let _: Vec<_> = mux_recv.recv(&mut pool).await.unwrap().collect();

        // Return traffic should go to primary
        let reply = make_tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 80, 50000, 0x10);
        let pkt = Packet::from_bytes(BytesMut::from(reply.as_slice()))
            .try_into_ip()
            .unwrap();
        mux_send.send(pkt).await.unwrap();

        let routed = drain(&mut primary_out).await;
        assert_eq!(&routed[12..16], &[1, 1, 1, 1]);
        assert!(secondary_out.try_recv().is_err());
    }
}
