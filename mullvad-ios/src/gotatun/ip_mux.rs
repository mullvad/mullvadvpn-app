//! Multiplexer for [`IpSend`]/[`IpRecv`] that combines traffic from a primary
//! (TUN device) and secondary (smoltcp) source.
//!
//! The [`IpMux`] tracks outbound connections from the secondary interface
//! (TCP by 4-tuple, ICMP by identifier) and routes matching return traffic
//! back to it. All other traffic goes to the primary interface.

use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use std::{
    collections::HashSet,
    io,
    net::IpAddr,
    sync::{Arc, Mutex},
};
use zerocopy::IntoBytes;

// ---------------------------------------------------------------------------
// Connection tracker
// ---------------------------------------------------------------------------

/// Tracks connections originating from the secondary interface so that return
/// traffic can be routed back to it.
#[derive(Default)]
pub struct ConnectionTracker {
    /// Active TCP connections from the secondary, identified by 4-tuple.
    tcp: HashSet<FourTuple>,
    /// ICMP identifiers from the secondary, keyed by (dest IP, identifier).
    icmp: HashSet<(IpAddr, u16)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FourTuple {
    src_ip: IpAddr,
    src_port: u16,
    dst_ip: IpAddr,
    dst_port: u16,
}

impl FourTuple {
    fn reverse(&self) -> Self {
        FourTuple {
            src_ip: self.dst_ip,
            src_port: self.dst_port,
            dst_ip: self.src_ip,
            dst_port: self.src_port,
        }
    }
}

impl ConnectionTracker {
    /// Register a TCP connection from the secondary.
    fn track_tcp(&mut self, tuple: FourTuple) {
        self.tcp.insert(tuple);
    }

    /// Remove a TCP connection (on FIN/RST).
    fn untrack_tcp(&mut self, tuple: &FourTuple) {
        self.tcp.remove(tuple);
    }

    /// Register an ICMP identifier from the secondary.
    fn track_icmp(&mut self, dest: IpAddr, ident: u16) {
        self.icmp.insert((dest, ident));
    }

    /// Check if return traffic matches a tracked secondary connection.
    fn is_secondary_return(&self, packet: &[u8]) -> bool {
        // Parse the IP header to determine protocol and extract identifiers
        if packet.len() < 20 {
            return false;
        }
        let version = packet[0] >> 4;
        match version {
            4 => self.is_secondary_return_v4(packet),
            6 => self.is_secondary_return_v6(packet),
            _ => false,
        }
    }

    fn is_secondary_return_v4(&self, packet: &[u8]) -> bool {
        let ihl = (packet[0] & 0x0f) as usize * 4;
        if packet.len() < ihl {
            return false;
        }
        let protocol = packet[9];
        let src_ip = IpAddr::from([packet[12], packet[13], packet[14], packet[15]]);
        let dst_ip = IpAddr::from([packet[16], packet[17], packet[18], packet[19]]);
        let payload = &packet[ihl..];

        match protocol {
            // TCP
            6 if payload.len() >= 4 => {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                let tuple = FourTuple {
                    src_ip,
                    src_port,
                    dst_ip,
                    dst_port,
                };
                // Return traffic has reversed src/dst compared to what we tracked
                self.tcp.contains(&tuple.reverse())
            }
            // ICMP
            1 if payload.len() >= 8 => {
                let icmp_type = payload[0];
                // Echo reply (type 0) — check if we tracked the corresponding request
                if icmp_type == 0 {
                    let ident = u16::from_be_bytes([payload[4], payload[5]]);
                    // The reply comes FROM src_ip, which was the destination of our request
                    self.icmp.contains(&(src_ip, ident))
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_secondary_return_v6(&self, packet: &[u8]) -> bool {
        if packet.len() < 40 {
            return false;
        }
        let next_header = packet[6];
        let src_ip: [u8; 16] = packet[8..24].try_into().unwrap();
        let dst_ip: [u8; 16] = packet[24..40].try_into().unwrap();
        let src_ip = IpAddr::from(src_ip);
        let dst_ip = IpAddr::from(dst_ip);
        let payload = &packet[40..];

        match next_header {
            // TCP
            6 if payload.len() >= 4 => {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                let tuple = FourTuple {
                    src_ip,
                    src_port,
                    dst_ip,
                    dst_port,
                };
                self.tcp.contains(&tuple.reverse())
            }
            // ICMPv6
            58 if payload.len() >= 8 => {
                let icmp_type = payload[0];
                // Echo reply (type 129)
                if icmp_type == 129 {
                    let ident = u16::from_be_bytes([payload[4], payload[5]]);
                    self.icmp.contains(&(src_ip, ident))
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Inspect an outbound packet from the secondary and register it in the tracker.
    fn register_secondary_outbound(&mut self, packet: &[u8]) {
        if packet.len() < 20 {
            return;
        }
        let version = packet[0] >> 4;
        match version {
            4 => self.register_secondary_outbound_v4(packet),
            6 => self.register_secondary_outbound_v6(packet),
            _ => {}
        }
    }

    fn register_secondary_outbound_v4(&mut self, packet: &[u8]) {
        let ihl = (packet[0] & 0x0f) as usize * 4;
        if packet.len() < ihl {
            return;
        }
        let protocol = packet[9];
        let src_ip = IpAddr::from([packet[12], packet[13], packet[14], packet[15]]);
        let dst_ip = IpAddr::from([packet[16], packet[17], packet[18], packet[19]]);
        let payload = &packet[ihl..];

        match protocol {
            // TCP
            6 if payload.len() >= 14 => {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                let flags = payload[13];
                let syn = flags & 0x02 != 0;
                let fin = flags & 0x01 != 0;
                let rst = flags & 0x04 != 0;

                let tuple = FourTuple {
                    src_ip,
                    src_port,
                    dst_ip,
                    dst_port,
                };

                if syn && !fin && !rst {
                    self.track_tcp(tuple);
                } else if fin || rst {
                    self.untrack_tcp(&tuple);
                }
            }
            // ICMP echo request (type 8)
            1 if payload.len() >= 8 && payload[0] == 8 => {
                let ident = u16::from_be_bytes([payload[4], payload[5]]);
                self.track_icmp(dst_ip, ident);
            }
            _ => {}
        }
    }

    fn register_secondary_outbound_v6(&mut self, packet: &[u8]) {
        if packet.len() < 40 {
            return;
        }
        let next_header = packet[6];
        let src_ip: [u8; 16] = packet[8..24].try_into().unwrap();
        let dst_ip: [u8; 16] = packet[24..40].try_into().unwrap();
        let src_ip = IpAddr::from(src_ip);
        let dst_ip = IpAddr::from(dst_ip);
        let payload = &packet[40..];

        match next_header {
            // TCP
            6 if payload.len() >= 14 => {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                let flags = payload[13];
                let syn = flags & 0x02 != 0;
                let fin = flags & 0x01 != 0;
                let rst = flags & 0x04 != 0;

                let tuple = FourTuple {
                    src_ip,
                    src_port,
                    dst_ip,
                    dst_port,
                };

                if syn && !fin && !rst {
                    self.track_tcp(tuple);
                } else if fin || rst {
                    self.untrack_tcp(&tuple);
                }
            }
            // ICMPv6 echo request (type 128)
            58 if payload.len() >= 8 && payload[0] == 128 => {
                let ident = u16::from_be_bytes([payload[4], payload[5]]);
                self.track_icmp(dst_ip, ident);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// IpMux
// ---------------------------------------------------------------------------

type SharedTracker = Arc<Mutex<ConnectionTracker>>;

/// The receive half of an [`IpMux`]. Merges packets from primary and secondary
/// [`IpRecv`] sources, registering secondary outbound traffic for routing.
pub struct IpMuxRecv<P: IpRecv, S: IpRecv> {
    primary: P,
    secondary: S,
    tracker: SharedTracker,
}

/// The send half of an [`IpMux`]. Routes return traffic to the secondary
/// interface if it matches a tracked connection, otherwise to the primary.
pub struct IpMuxSend<P: IpSend, S: IpSend> {
    primary: P,
    secondary: S,
    tracker: SharedTracker,
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
    let tracker = Arc::new(Mutex::new(ConnectionTracker::default()));
    (
        IpMuxRecv {
            primary: primary_recv,
            secondary: secondary_recv,
            tracker: tracker.clone(),
        },
        IpMuxSend {
            primary: primary_send,
            secondary: secondary_send,
            tracker,
        },
    )
}

impl<P: IpRecv, S: IpRecv> IpRecv for IpMuxRecv<P, S> {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        // We can't use tokio::select! because `pool` would be borrowed mutably
        // in both branches. Instead, use a biased poll: try secondary first
        // (non-blocking via try_recv semantics if possible), then block on
        // primary. But since IpRecv::recv is async and we need to race, we
        // use futures::select with pinned futures and pass pool to only one
        // at a time. We use a simple loop that alternates.
        //
        // A practical approach: since the secondary (smoltcp) is expected to
        // produce much less traffic, we first try secondary with a brief
        // poll, then fall through to primary.
        use std::future::poll_fn;
        use std::pin::pin;
        use std::task::Poll;

        // Create both futures but only give pool to the one that resolves
        let mut primary_fut = pin!(self.primary.recv(pool));
        let mut secondary_pool = PacketBufPool::new(1);
        let mut secondary_fut = pin!(self.secondary.recv(&mut secondary_pool));

        // Race the two futures
        let result = poll_fn(|cx| {
            // Check secondary first (biased toward control traffic)
            if let Poll::Ready(result) = secondary_fut.as_mut().poll(cx) {
                return Poll::Ready(RecvResult::Secondary(result));
            }
            if let Poll::Ready(result) = primary_fut.as_mut().poll(cx) {
                return Poll::Ready(RecvResult::Primary(result));
            }
            Poll::Pending
        })
        .await;

        match result {
            RecvResult::Primary(result) => result.map(|iter| {
                let packets: Vec<_> = iter.collect();
                MuxIter(packets.into_iter())
            }),
            RecvResult::Secondary(result) => {
                let iter = result?;
                let packets: Vec<_> = iter.collect();
                {
                    let mut tracker = self.tracker.lock().unwrap();
                    for pkt in &packets {
                        tracker.register_secondary_outbound((*pkt).as_bytes());
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
        let is_secondary = {
            let tracker = self.tracker.lock().unwrap();
            tracker.is_secondary_return((*packet).as_bytes())
        };

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
