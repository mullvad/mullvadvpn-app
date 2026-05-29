//! NAT for the UDP traffic arriving on the tunnel device.
//!
//! Each client source endpoint gets one upstream socket, reused for every destination it talks to,
//! and a task forwarding the replies back. Flows are evicted once idle so neither the table nor
//! the file descriptors it holds can grow without bound.

use bytes::{Bytes, BytesMut};
use smoltcp::{
    phy::ChecksumCapabilities,
    wire::{
        IPV4_HEADER_LEN, IpProtocol, Ipv4Packet, Ipv4Repr, UDP_HEADER_LEN, UdpPacket, UdpRepr,
    },
};
use std::{
    collections::BTreeMap,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, sync::mpsc};
use tokio_util::task::AbortOnDropHandle;

/// Largest datagram we will accept from an upstream socket.
const MAX_DATAGRAM_SIZE: usize = u16::MAX as usize;
/// How long a flow may sit unused before its socket is closed.
const FLOW_TIMEOUT: Duration = Duration::from_secs(60);
/// How often idle flows are looked for.
const SWEEP_INTERVAL: Duration = Duration::from_secs(10);
/// Upper bound on concurrent flows, enforced by evicting the least recently used.
const MAX_FLOWS: usize = 512;
/// Datagrams buffered per flow on their way upstream.
const OUTBOUND_QUEUE_DEPTH: usize = 64;
const HOP_LIMIT: u8 = 64;

pub struct UdpRouter {
    egress_tx: mpsc::Sender<Bytes>,
    flows: BTreeMap<SocketAddrV4, Flow>,
    /// The clock every flow timestamp is measured against.
    reference: Instant,
    last_sweep: Instant,
    max_flows: usize,
    mtu: u16,
}

struct Flow {
    /// Datagrams for the sending task, which owns the waiting that the read loop must not do.
    outbound_tx: mpsc::Sender<(Bytes, SocketAddr)>,
    /// Milliseconds on the router's clock at the last packet in either direction.
    last_used: Arc<AtomicU64>,
    _sender: AbortOnDropHandle<()>,
    _receiver: AbortOnDropHandle<()>,
}

impl UdpRouter {
    pub fn new(egress_tx: mpsc::Sender<Bytes>, mtu: u16) -> Self {
        let reference = Instant::now();

        Self {
            egress_tx,
            flows: BTreeMap::new(),
            reference,
            last_sweep: reference,
            max_flows: MAX_FLOWS,
            mtu,
        }
    }

    pub async fn route(&mut self, source: Ipv4Addr, destination: Ipv4Addr, datagram: Bytes) {
        if let Err(err) = self.route_inner(source, destination, datagram).await {
            log::debug!("Failed to route a UDP packet from {source} to {destination}: {err}");
        }
    }

    async fn route_inner(
        &mut self,
        source: Ipv4Addr,
        destination: Ipv4Addr,
        datagram: Bytes,
    ) -> io::Result<()> {
        let header = UdpPacket::new_checked(datagram.as_ref())
            .map_err(|err| io::Error::other(format!("malformed UDP packet: {err}")))?;
        let client = SocketAddrV4::new(source, header.src_port());
        let upstream = SocketAddrV4::new(destination, header.dst_port());
        let payload_len = header.payload().len();

        let now = self.now();
        let flow = self.flow_for(client).await?;
        flow.last_used.store(now, Ordering::Relaxed);

        // Handing the datagram to the flow's sending task rather than sending it here keeps the
        // tunnel read loop from ever waiting on one upstream socket. `try_send_to` cannot be used
        // for that: tokio answers it with `WouldBlock` without even attempting the syscall
        // whenever it has not yet observed the socket as writable, which is exactly the state a
        // freshly bound socket is in, so the first datagram of every flow would be dropped.
        //
        // Dropping when that queue is full is the intended response, not a fault. This queue and
        // the socket buffer behind it are the only congestion signal the tunnelled sender gets,
        // so making either deeper just converts loss into latency and hurts the traffic inside.
        let payload = datagram.slice(UDP_HEADER_LEN..UDP_HEADER_LEN + payload_len);
        if flow.outbound_tx.try_send((payload, upstream.into())).is_err() {
            log::debug!("{client} - upstream is congested, shedding a datagram");
        }

        Ok(())
    }

    /// Milliseconds on the router's clock.
    fn now(&self) -> u64 {
        self.reference.elapsed().as_millis() as u64
    }

    async fn flow_for(&mut self, client: SocketAddrV4) -> io::Result<&Flow> {
        if !self.flows.contains_key(&client) {
            if self.flows.len() >= self.max_flows {
                self.evict_least_recently_used();
            }

            let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
            log::debug!(
                "{client} - new UDP flow via {}",
                socket
                    .local_addr()
                    .map_or_else(|err| err.to_string(), |address| address.to_string())
            );
            let last_used = Arc::new(AtomicU64::new(self.now()));
            let (outbound_tx, outbound_rx) = mpsc::channel(OUTBOUND_QUEUE_DEPTH);

            let sender = tokio::spawn(send_upstream(socket.clone(), client, outbound_rx));
            let receiver = tokio::spawn(forward_replies(
                socket,
                client,
                last_used.clone(),
                self.reference,
                self.egress_tx.clone(),
                self.mtu,
            ));

            self.flows.insert(
                client,
                Flow {
                    outbound_tx,
                    last_used,
                    _sender: AbortOnDropHandle::new(sender),
                    _receiver: AbortOnDropHandle::new(receiver),
                },
            );
        }

        Ok(&self.flows[&client])
    }

    /// Close flows that have not carried a packet recently.
    ///
    /// This runs off the tunnel read loop rather than a timer, so an idle router keeps its flows
    /// until traffic resumes. [`MAX_FLOWS`] is what bounds them in the meantime.
    pub fn sweep_idle_flows(&mut self) {
        if self.last_sweep.elapsed() < SWEEP_INTERVAL {
            return;
        }
        self.last_sweep = Instant::now();

        let now = self.now();
        let timeout = FLOW_TIMEOUT.as_millis() as u64;
        self.flows.retain(|client, flow| {
            let idle = now.saturating_sub(flow.last_used.load(Ordering::Relaxed));
            if idle < timeout {
                return true;
            }
            log::debug!("Closing the idle UDP flow for {client}");
            false
        });
    }

    fn evict_least_recently_used(&mut self) {
        let oldest = self
            .flows
            .iter()
            .min_by_key(|(_, flow)| flow.last_used.load(Ordering::Relaxed))
            .map(|(client, _)| *client);

        if let Some(client) = oldest {
            log::debug!("Closing the UDP flow for {client} to make room for a new one");
            self.flows.remove(&client);
        }
    }
}

/// Send the client's datagrams upstream, waiting for the socket rather than dropping them.
async fn send_upstream(
    socket: Arc<UdpSocket>,
    client: SocketAddrV4,
    mut outbound_rx: mpsc::Receiver<(Bytes, SocketAddr)>,
) {
    let mut batch = Vec::with_capacity(OUTBOUND_QUEUE_DEPTH);

    while outbound_rx.recv_many(&mut batch, OUTBOUND_QUEUE_DEPTH).await > 0 {
        for (payload, destination) in batch.drain(..) {
            if let Err(err) = socket.send_to(&payload, destination).await {
                log::debug!("{client} - failed to send a datagram to {destination}: {err}");
            }
        }
    }
}

/// Forward everything the upstream socket receives back to the client.
async fn forward_replies(
    socket: Arc<UdpSocket>,
    client: SocketAddrV4,
    last_used: Arc<AtomicU64>,
    reference: Instant,
    egress_tx: mpsc::Sender<Bytes>,
    mtu: u16,
) {
    let mut buffer = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut ident = 0u16;

    loop {
        let (received, upstream) = match socket.recv_from(&mut buffer).await {
            Ok(reply) => reply,
            Err(err) => {
                log::debug!("{client} - upstream socket failed: {err}");
                return;
            }
        };

        let IpAddr::V4(upstream_ip) = upstream.ip() else {
            continue;
        };
        last_used.store(reference.elapsed().as_millis() as u64, Ordering::Relaxed);

        let source = SocketAddrV4::new(upstream_ip, upstream.port());
        let payload = &buffer[..received];
        ident = ident.wrapping_add(1);

        let send = |packet: Bytes| {
            if egress_tx.try_send(packet).is_err() {
                log::warn!("{client} - tunnel is congested, dropping a datagram");
                return false;
            }
            true
        };

        // A reply that fits is built straight into its packet. Only one too large for the tunnel
        // has to be assembled and then split, which is what costs the second pass over it.
        if IPV4_HEADER_LEN + UDP_HEADER_LEN + received <= usize::from(mtu) {
            send(reply_packet(source, client, payload, ident));
        } else {
            let datagram = build_datagram(source, client, payload);
            for packet in fragments(*source.ip(), *client.ip(), &datagram, mtu, ident) {
                if !send(packet) {
                    break;
                }
            }
        }
    }
}

/// Build the IPv4 packet carrying a reply, headers and payload in one allocation.
fn reply_packet(
    source: SocketAddrV4,
    destination: SocketAddrV4,
    payload: &[u8],
    ident: u16,
) -> Bytes {
    let mut packet =
        BytesMut::zeroed(IPV4_HEADER_LEN + UDP_HEADER_LEN + payload.len());
    let mut view = Ipv4Packet::new_unchecked(&mut packet[..]);

    Ipv4Repr {
        src_addr: *source.ip(),
        dst_addr: *destination.ip(),
        next_header: IpProtocol::Udp,
        payload_len: UDP_HEADER_LEN + payload.len(),
        hop_limit: HOP_LIMIT,
    }
    .emit(&mut view, &ChecksumCapabilities::default());

    UdpRepr {
        src_port: source.port(),
        dst_port: destination.port(),
    }
    .emit(
        &mut UdpPacket::new_unchecked(view.payload_mut()),
        &(*source.ip()).into(),
        &(*destination.ip()).into(),
        payload.len(),
        |buffer| buffer.copy_from_slice(payload),
        &ChecksumCapabilities::default(),
    );

    // Matching what `fragments` emits keeps a reply's framing the same whether or not it was split.
    view.set_dont_frag(false);
    view.set_ident(ident);
    view.fill_checksum();

    packet.freeze()
}

/// Build the UDP datagram carrying `payload`, checksum included.
fn build_datagram(source: SocketAddrV4, destination: SocketAddrV4, payload: &[u8]) -> Vec<u8> {
    let mut datagram = vec![0u8; UDP_HEADER_LEN + payload.len()];
    let repr = UdpRepr {
        src_port: source.port(),
        dst_port: destination.port(),
    };

    repr.emit(
        &mut UdpPacket::new_unchecked(&mut datagram[..]),
        &(*source.ip()).into(),
        &(*destination.ip()).into(),
        payload.len(),
        |buffer| buffer.copy_from_slice(payload),
        &ChecksumCapabilities::default(),
    );

    datagram
}

/// Wrap a datagram in as many IPv4 packets as the MTU requires.
///
/// Upstream sockets hand us datagrams already reassembled by the OS, so anything larger than the
/// tunnel's MTU has to be split again before the client can receive it.
fn fragments(
    source: Ipv4Addr,
    destination: Ipv4Addr,
    datagram: &[u8],
    mtu: u16,
    ident: u16,
) -> impl Iterator<Item = Bytes> + '_ {
    // Every fragment but the last has to end on an 8 octet boundary.
    let max_payload = ((usize::from(mtu).saturating_sub(IPV4_HEADER_LEN) / 8) * 8).max(8);

    datagram
        .chunks(max_payload)
        .enumerate()
        .map(move |(index, chunk)| {
            let offset = index * max_payload;
            let more_fragments = offset + chunk.len() < datagram.len();

            let mut packet = vec![0u8; IPV4_HEADER_LEN + chunk.len()];
            let mut view = Ipv4Packet::new_unchecked(&mut packet[..]);
            Ipv4Repr {
                src_addr: source,
                dst_addr: destination,
                next_header: IpProtocol::Udp,
                payload_len: chunk.len(),
                hop_limit: HOP_LIMIT,
            }
            .emit(&mut view, &ChecksumCapabilities::default());

            view.payload_mut().copy_from_slice(chunk);
            // `emit` assumes a whole datagram, so the fragment fields are set afterwards and the
            // checksum recomputed over them.
            view.set_dont_frag(false);
            view.set_ident(ident);
            view.set_more_frags(more_fragments);
            view.set_frag_offset(offset as u16);
            view.fill_checksum();

            Bytes::from(packet)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 2), 5353);
    const SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 53);
    const MTU: u16 = 1500;

    /// Reassemble fragments the way a receiving host would, checking the offsets and flags.
    fn reassemble(packets: &[Bytes]) -> Vec<u8> {
        let mut datagram = Vec::new();

        for (index, packet) in packets.iter().enumerate() {
            let view = Ipv4Packet::new_checked(packet.as_ref()).expect("a well formed IPv4 packet");
            let last = index == packets.len() - 1;

            assert_eq!(view.more_frags(), !last, "only the last fragment may end it");
            assert_eq!(
                usize::from(view.frag_offset()),
                datagram.len(),
                "fragments must be contiguous"
            );
            assert!(!view.dont_frag(), "fragments must not be marked don't fragment");
            assert!(
                packet.len() <= usize::from(MTU),
                "a fragment must fit the MTU"
            );

            datagram.extend_from_slice(view.payload());
        }

        datagram
    }

    fn payload_of(datagram: &[u8]) -> Vec<u8> {
        let packet = UdpPacket::new_checked(datagram).expect("a well formed UDP datagram");
        assert_eq!(packet.src_port(), SERVER.port());
        assert_eq!(packet.dst_port(), CLIENT.port());
        packet.payload().to_vec()
    }

    #[test]
    fn a_small_reply_is_one_packet() {
        let datagram = build_datagram(SERVER, CLIENT, b"pong");
        let packets: Vec<_> = fragments(*SERVER.ip(), *CLIENT.ip(), &datagram, MTU, 1).collect();

        assert_eq!(packets.len(), 1);
        assert_eq!(payload_of(&reassemble(&packets)), b"pong");
    }

    /// Upstream sockets hand us datagrams the OS already reassembled, so replies larger than the
    /// tunnel's MTU have to be split again rather than dropped or truncated.
    #[test]
    fn an_oversized_reply_is_fragmented() {
        let payload: Vec<u8> = (0..5000u32).map(|byte| byte as u8).collect();
        let datagram = build_datagram(SERVER, CLIENT, &payload);
        let packets: Vec<_> = fragments(*SERVER.ip(), *CLIENT.ip(), &datagram, MTU, 7).collect();

        assert!(packets.len() > 1, "5000 bytes cannot fit a 1500 byte MTU");
        assert!(
            packets
                .iter()
                .all(|packet| Ipv4Packet::new_unchecked(packet.as_ref()).ident() == 7),
            "every fragment of a datagram shares its identifier"
        );
        assert_eq!(payload_of(&reassemble(&packets)), payload);
    }

    #[test]
    fn a_reply_that_exactly_fills_the_mtu_is_not_split() {
        let payload = vec![0xab; usize::from(MTU) - IPV4_HEADER_LEN - UDP_HEADER_LEN];
        let datagram = build_datagram(SERVER, CLIENT, &payload);
        let packets: Vec<_> = fragments(*SERVER.ip(), *CLIENT.ip(), &datagram, MTU, 1).collect();

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].len(), usize::from(MTU));
        assert_eq!(payload_of(&reassemble(&packets)), payload);
    }

    /// The one-allocation path and the assemble-then-split path must agree, so that a reply's
    /// framing does not depend on which of them produced it.
    #[test]
    fn the_single_packet_path_matches_the_fragmenting_one() {
        let payload = b"a reply that comfortably fits the tunnel";
        let datagram = build_datagram(SERVER, CLIENT, payload);
        let split: Vec<_> = fragments(*SERVER.ip(), *CLIENT.ip(), &datagram, MTU, 9).collect();

        assert_eq!(split.len(), 1, "this reply should not have been split");
        assert_eq!(reply_packet(SERVER, CLIENT, payload, 9), split[0]);
    }

    #[tokio::test]
    async fn flows_are_evicted_once_idle() {
        let (egress_tx, _egress_rx) = mpsc::channel(16);
        let mut router = UdpRouter::new(egress_tx, MTU);
        router.route(*CLIENT.ip(), *SERVER.ip(), Bytes::from(build_datagram(CLIENT, SERVER, b"ping"))).await;
        assert_eq!(router.flows.len(), 1);

        // Pretend the flow last carried a packet long enough ago to have timed out.
        router.last_sweep -= SWEEP_INTERVAL;
        for flow in router.flows.values() {
            flow.last_used.store(0, Ordering::Relaxed);
        }
        router.reference -= FLOW_TIMEOUT + Duration::from_secs(1);

        router.sweep_idle_flows();
        assert!(router.flows.is_empty(), "an idle flow must not hold its socket forever");
    }

    #[tokio::test]
    async fn the_flow_table_is_capped() {
        let (egress_tx, _egress_rx) = mpsc::channel(16);
        let mut router = UdpRouter::new(egress_tx, MTU);
        router.max_flows = 4;

        for port in 0..u16::try_from(router.max_flows).unwrap() + 6 {
            let client = SocketAddrV4::new(*CLIENT.ip(), 1024 + port);
            router
                .route(*client.ip(), *SERVER.ip(), Bytes::from(build_datagram(client, SERVER, b"ping")))
                .await;
        }

        assert_eq!(
            router.flows.len(),
            4,
            "a client opening endless flows must not exhaust our file descriptors"
        );
    }
}
