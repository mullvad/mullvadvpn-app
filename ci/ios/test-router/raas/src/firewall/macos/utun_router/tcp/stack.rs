//! The state machine that bridges smoltcp sockets to real upstream sockets.
//!
//! Nothing here awaits: every step the driver runs on a wakeup is a plain method taking the
//! current time, so the stack can also be stepped synchronously by tests.

use bytes::{Buf, Bytes, BytesMut};
use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    socket::tcp::{self, ListenError},
    time::{Duration as SmoltcpDuration, Instant as SmoltcpInstant},
    wire::Ipv4Packet,
};
use std::{
    collections::{BTreeMap, VecDeque},
    mem,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Notify, mpsc};

use super::{
    IDLE_TIMEOUT, KEEP_ALIVE_INTERVAL, PENDING_PACKET_QUEUE_DEPTH, TCP_BUFFER_SIZE,
    TcpConnectionId, UPSTREAM_READ_CHUNK,
    device::SmoltcpDevice,
    downstream_packet_identifier, tcp_packet_is_syn,
    upstream_socket::{UpstreamEvent, UpstreamMsg, UpstreamSocket},
};

pub struct SmoltcpStack {
    device: SmoltcpDevice,
    interface: Interface,
    sockets: SocketSet<'static>,
    connections: BTreeMap<TcpConnectionId, Connection>,
    event_tx: mpsc::Sender<UpstreamMsg>,
    notify: Arc<Notify>,
}

struct Connection {
    upstream: UpstreamSocket,
    state: ConnectionState,
    /// A payload from upstream that the smoltcp socket has not fully accepted yet.
    ///
    /// Holding at most one chunk is what bounds this direction: until it has been handed over,
    /// nothing new is taken off the channel, so a slow client eventually stalls the upstream read.
    pending_downstream: Option<Bytes>,
    /// The upstream closed its send half; the client is FIN'd once `pending_downstream` drains.
    upstream_eof: bool,
    /// The smoltcp state last logged, so only transitions are reported.
    last_state: tcp::State,
}

enum ConnectionState {
    /// The upstream connection is still being established, so the client's packets are held back.
    ///
    /// They must not go to smoltcp yet: an unmatched SYN sitting in the device queue could be
    /// picked up by another connection's listening socket, since they all listen on the same
    /// endpoint.
    Connecting {
        pending: VecDeque<Bytes>,
    },
    Connected {
        handle: SocketHandle,
    },
}

impl SmoltcpStack {
    pub fn new(
        mtu: u16,
        egress_tx: mpsc::Sender<Bytes>,
        event_tx: mpsc::Sender<UpstreamMsg>,
        notify: Arc<Notify>,
        now: SmoltcpInstant,
    ) -> Self {
        let interface_config = Config::new(smoltcp::wire::HardwareAddress::Ip);
        let mut device = SmoltcpDevice::new(mtu, egress_tx);
        let mut interface = Interface::new(interface_config, &mut device, now);

        // The interface owns no addresses: it answers for whatever destination the client picked,
        // which is what makes the sockets below able to listen on it.
        interface.set_any_ip(true);
        interface
            .routes_mut()
            .add_default_ipv4_route(smoltcp::wire::Ipv4Address::new(0, 0, 0, 1))
            .expect("enough space for default IPv4 route");
        interface
            .routes_mut()
            .add_default_ipv6_route(smoltcp::wire::Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 1))
            .expect("enough space for default IPv6 route");

        Self {
            device,
            interface,
            sockets: SocketSet::new(Vec::new()),
            connections: Default::default(),
            event_tx,
            notify,
        }
    }

    /// How long the driver may sleep before smoltcp next needs servicing.
    pub fn poll_delay(&mut self, now: SmoltcpInstant) -> Option<Duration> {
        self.interface
            .poll_delay(now, &self.sockets)
            .map(|delay| Duration::from_millis(delay.total_millis()))
    }

    /// Advance smoltcp: process queued ingress and emit egress.
    pub fn poll(&mut self, now: SmoltcpInstant) {
        let _ = self
            .interface
            .poll(now, &mut self.device, &mut self.sockets);
    }

    /// Route a packet read off the tunnel device.
    pub fn handle_downstream_packet(&mut self, packet: Ipv4Packet<Bytes>) {
        let Some(id) = downstream_packet_identifier(&packet) else {
            return;
        };

        match self.connections.get_mut(&id) {
            Some(connection) => match &mut connection.state {
                ConnectionState::Connecting { pending } => {
                    if pending.len() >= PENDING_PACKET_QUEUE_DEPTH {
                        log::warn!("{id:?} - too many packets buffered while connecting, dropping");
                        return;
                    }
                    pending.push_back(packet.into_inner());
                }
                ConnectionState::Connected { .. } => self.device.enqueue_rx(packet.into_inner()),
            },
            None if tcp_packet_is_syn(&packet) => {
                log::debug!("{id:?} - connecting upstream for a new client connection");
                let upstream =
                    UpstreamSocket::connect(id, self.event_tx.clone(), self.notify.clone());
                self.connections.insert(
                    id,
                    Connection {
                        upstream,
                        state: ConnectionState::Connecting {
                            pending: VecDeque::from([packet.into_inner()]),
                        },
                        pending_downstream: None,
                        upstream_eof: false,
                        last_state: tcp::State::Closed,
                    },
                );
            }
            // Traffic for a connection we know nothing about. Handing it to smoltcp with no
            // socket to accept it is what produces the RST.
            None => self.device.enqueue_rx(packet.into_inner()),
        }
    }

    pub fn handle_upstream_event(&mut self, msg: UpstreamMsg) {
        let mut drop_connection = false;

        let Some(connection) = self.connections.get_mut(&msg.id) else {
            log::debug!("Received an event for the dead connection {:?}", msg.id);
            return;
        };

        match msg.event {
            UpstreamEvent::Connected => {
                let ConnectionState::Connecting { pending } = &mut connection.state else {
                    log::debug!("{:?} - connected more than once", msg.id);
                    return;
                };
                let pending = mem::take(pending);

                match open_local_listener(msg.id.upstream) {
                    Ok(socket) => {
                        log::debug!(
                            "{:?} - upstream connected, answering the client's handshake",
                            msg.id
                        );
                        connection.state = ConnectionState::Connected {
                            handle: self.sockets.add(socket),
                        };
                    }
                    Err(err) => {
                        log::error!("{:?} - failed to listen for the client: {err}", msg.id);
                        drop_connection = true;
                    }
                }

                // With a listening socket these complete the handshake immediately, without
                // waiting for the client to retransmit its SYN. Without one, smoltcp RSTs.
                for packet in pending {
                    self.device.enqueue_rx(packet);
                }
            }
            UpstreamEvent::ConnectFailed => {
                if let ConnectionState::Connecting { pending } = &mut connection.state {
                    for packet in mem::take(pending) {
                        self.device.enqueue_rx(packet);
                    }
                }
                drop_connection = true;
            }
            UpstreamEvent::Error => match connection.state {
                ConnectionState::Connected { handle } => {
                    self.sockets.get_mut::<tcp::Socket>(handle).abort();
                }
                ConnectionState::Connecting { .. } => drop_connection = true,
            },
        }

        if drop_connection {
            self.drop_connection(msg.id);
        }
    }

    /// Move payloads read from upstream into the smoltcp sockets that serve the clients.
    pub fn pump(&mut self) {
        for (id, connection) in self.connections.iter_mut() {
            let ConnectionState::Connected { handle } = connection.state else {
                continue;
            };
            let socket = self.sockets.get_mut::<tcp::Socket>(handle);

            while socket.can_send() {
                if connection.pending_downstream.is_none() {
                    match connection.upstream.poll_downstream_payload() {
                        Ok(payload) => connection.pending_downstream = Some(payload),
                        Err(mpsc::error::TryRecvError::Empty) => break,
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            connection.upstream_eof = true;
                            break;
                        }
                    }
                }

                let Some(payload) = &mut connection.pending_downstream else {
                    break;
                };

                match socket.send_slice(payload) {
                    Ok(0) => break,
                    Ok(written) => {
                        payload.advance(written);
                        if payload.is_empty() {
                            connection.pending_downstream = None;
                        }
                    }
                    Err(err) => {
                        log::warn!("{id:?} - failed to queue payload for the client: {err}");
                        break;
                    }
                }
            }

            if connection.upstream_eof && connection.pending_downstream.is_none() {
                socket.close();
            }
        }
    }

    /// Move payloads received from the clients into their upstream sockets, and drop connections
    /// whose smoltcp socket has finished closing.
    pub fn reap(&mut self) {
        self.connections.retain(|id, connection| {
            let ConnectionState::Connected { handle } = connection.state else {
                return true;
            };
            let socket = self.sockets.get_mut::<tcp::Socket>(handle);

            if socket.state() != connection.last_state {
                log::debug!("{id:?} - {} -> {}", connection.last_state, socket.state());
                connection.last_state = socket.state();
            }

            // Reserve room on the upstream channel *before* draining smoltcp. If there is none,
            // the bytes stay in the receive buffer, the advertised window shrinks, and the client
            // stops sending, rather than us dropping payload it believes was delivered.
            while socket.can_recv() {
                let Some(permit) = connection.upstream.reserve_upstream() else {
                    break;
                };

                let mut payload = BytesMut::zeroed(socket.recv_queue().min(UPSTREAM_READ_CHUNK));
                match socket.recv_slice(&mut payload) {
                    Ok(0) => break,
                    Ok(received) => {
                        payload.truncate(received);
                        permit.send(payload.freeze());
                    }
                    Err(err) => {
                        log::warn!("{id:?} - failed to read the client's payload: {err}");
                        break;
                    }
                }
            }

            if client_closed(socket.state())
                && socket.recv_queue() == 0
                && !connection.upstream.upstream_writes_shut_down()
            {
                connection.upstream.shutdown_upstream_writes();
            }

            if matches!(socket.state(), tcp::State::Closed | tcp::State::TimeWait) {
                log::debug!("{id:?} - connection finished in {}", socket.state());
                self.sockets.remove(handle);
                return false;
            }

            true
        });
    }

    /// Whether smoltcp still holds ingress that it could not accept on the last poll.
    pub fn has_queued_rx(&self) -> bool {
        self.device.has_queued_rx()
    }

    fn drop_connection(&mut self, id: TcpConnectionId) {
        let Some(connection) = self.connections.remove(&id) else {
            return;
        };
        if let ConnectionState::Connected { handle } = connection.state {
            self.sockets.remove(handle);
        }
    }
}

#[cfg(test)]
impl SmoltcpStack {
    /// Register a connection whose upstream is driven by the test rather than a real socket.
    ///
    /// The connection starts out connecting, exactly as if a SYN had just arrived, so tests still
    /// go through the handshake and the buffered-packet flush.
    fn insert_connecting(
        &mut self,
        id: TcpConnectionId,
    ) -> super::upstream_socket::DetachedUpstream {
        let (upstream, detached) = UpstreamSocket::detached();
        self.connections.insert(
            id,
            Connection {
                upstream,
                state: ConnectionState::Connecting {
                    pending: VecDeque::new(),
                },
                pending_downstream: None,
                upstream_eof: false,
                last_state: tcp::State::Closed,
            },
        );

        detached
    }

    fn is_tracking(&self, id: &TcpConnectionId) -> bool {
        self.connections.contains_key(id)
    }
}

/// Whether the client has closed its send half.
fn client_closed(state: tcp::State) -> bool {
    matches!(
        state,
        tcp::State::CloseWait | tcp::State::LastAck | tcp::State::Closing | tcp::State::TimeWait
    )
}

/// Create the socket that impersonates the client's original destination.
///
/// This only works because the interface has `any_ip` set, which lets it answer for an address it
/// does not own.
fn open_local_listener(destination: SocketAddr) -> Result<tcp::Socket<'static>, ListenError> {
    let rx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
    let tx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
    let mut socket = tcp::Socket::new(rx_buffer, tx_buffer);

    // Without these, a client that disappears mid-connection leaves the socket and its upstream
    // connection around forever.
    socket.set_keep_alive(Some(SmoltcpDuration::from_secs(KEEP_ALIVE_INTERVAL)));
    socket.set_timeout(Some(SmoltcpDuration::from_secs(IDLE_TIMEOUT)));
    socket.listen(destination)?;

    Ok(socket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firewall::macos::utun_router::tcp::{
        TCP_DATA_CHANNEL_CAPACITY, device::SmoltcpDevice, tcp_packet_from_ip_packet,
        upstream_socket::DetachedUpstream,
    };
    use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, IpEndpoint, Ipv4Address};
    use std::net::Ipv4Addr;

    const MTU: u16 = 1500;
    const CLIENT_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
    const CLIENT_PORT: u16 = 49152;
    const SERVER_IP: Ipv4Addr = Ipv4Addr::new(1, 2, 3, 4);
    const SERVER_PORT: u16 = 80;

    fn connection_id() -> TcpConnectionId {
        TcpConnectionId {
            downstream: SocketAddr::new(CLIENT_IP.into(), CLIENT_PORT),
            upstream: SocketAddr::new(SERVER_IP.into(), SERVER_PORT),
        }
    }

    /// A real smoltcp interface standing in for the client behind the tunnel.
    struct Client {
        interface: Interface,
        device: SmoltcpDevice,
        sockets: SocketSet<'static>,
        handle: SocketHandle,
        egress_rx: mpsc::Receiver<Bytes>,
    }

    impl Client {
        fn new(now: SmoltcpInstant) -> Self {
            let (egress_tx, egress_rx) = mpsc::channel(2048);
            let mut device = SmoltcpDevice::new(MTU, egress_tx);
            let mut interface = Interface::new(Config::new(HardwareAddress::Ip), &mut device, now);

            interface.update_ip_addrs(|addrs| {
                addrs
                    .push(IpCidr::new(IpAddress::Ipv4(CLIENT_IP), 32))
                    .expect("enough space for the client address");
            });
            interface
                .routes_mut()
                .add_default_ipv4_route(Ipv4Address::new(0, 0, 0, 1))
                .expect("enough space for the default route");

            let mut socket = tcp::Socket::new(
                tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]),
                tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]),
            );
            // Without this a run of one-byte writes coalesces into a single segment.
            socket.set_nagle_enabled(false);
            socket
                .connect(
                    interface.context(),
                    IpEndpoint::from(SocketAddr::new(SERVER_IP.into(), SERVER_PORT)),
                    CLIENT_PORT,
                )
                .expect("the client can start connecting");

            let mut sockets = SocketSet::new(Vec::new());
            let handle = sockets.add(socket);

            Self {
                interface,
                device,
                sockets,
                handle,
                egress_rx,
            }
        }

        fn socket(&mut self) -> &mut tcp::Socket<'static> {
            self.sockets.get_mut::<tcp::Socket>(self.handle)
        }

        fn poll(&mut self, now: SmoltcpInstant) {
            let _ = self
                .interface
                .poll(now, &mut self.device, &mut self.sockets);
        }
    }

    /// The stack under test, plus the ends of the channels a test needs to observe.
    struct Harness {
        stack: SmoltcpStack,
        egress_rx: mpsc::Receiver<Bytes>,
        /// Kept alive so the stack's event sender never sees a closed channel.
        _event_rx: mpsc::Receiver<UpstreamMsg>,
        client: Client,
        now_ms: i64,
    }

    impl Harness {
        fn new() -> Self {
            let now = SmoltcpInstant::from_millis(0);
            let (egress_tx, egress_rx) = mpsc::channel(2048);
            let (event_tx, event_rx) = mpsc::channel(64);
            let stack = SmoltcpStack::new(MTU, egress_tx, event_tx, Arc::new(Notify::new()), now);

            Self {
                stack,
                egress_rx,
                _event_rx: event_rx,
                client: Client::new(now),
                now_ms: 0,
            }
        }

        fn now(&self) -> SmoltcpInstant {
            SmoltcpInstant::from_millis(self.now_ms)
        }

        /// One full servicing pass, in the same order the driver runs them.
        fn step(&mut self) {
            self.stack.pump();
            self.stack.poll(self.now());
            self.stack.reap();
        }

        /// Move everything the client emitted into the stack, and vice versa.
        fn exchange(&mut self) {
            for _ in 0..8 {
                self.client.poll(self.now());
                while let Ok(packet) = self.client.egress_rx.try_recv() {
                    self.stack
                        .handle_downstream_packet(Ipv4Packet::new_unchecked(packet));
                }

                self.step();
                while let Ok(packet) = self.egress_rx.try_recv() {
                    self.client.device.enqueue_rx(packet);
                }
            }
            self.client.poll(self.now());
        }

        /// Bring the connection up: the client's SYN is buffered until the upstream connects.
        fn establish(&mut self) -> DetachedUpstream {
            let id = connection_id();
            let detached = self.stack.insert_connecting(id);

            self.exchange();
            assert_eq!(
                self.client.socket().state(),
                tcp::State::SynSent,
                "the client must not be answered before the upstream connects"
            );

            self.stack.handle_upstream_event(UpstreamMsg {
                id,
                event: UpstreamEvent::Connected,
            });
            self.exchange();

            detached
        }

        fn egress_packets(&mut self) -> Vec<Bytes> {
            let mut packets = Vec::new();
            while let Ok(packet) = self.egress_rx.try_recv() {
                packets.push(packet);
            }
            packets
        }
    }

    fn is_reset(packet: &Bytes) -> bool {
        tcp_packet_from_ip_packet(&Ipv4Packet::new_unchecked(packet.clone()))
            .is_some_and(|tcp| tcp.rst())
    }

    fn is_syn_ack(packet: &Bytes) -> bool {
        tcp_packet_from_ip_packet(&Ipv4Packet::new_unchecked(packet.clone()))
            .is_some_and(|tcp| tcp.syn() && tcp.ack())
    }

    #[tokio::test]
    async fn handshake_completes_once_the_upstream_connects() {
        let mut harness = Harness::new();
        harness.establish();

        assert_eq!(
            harness.client.socket().state(),
            tcp::State::Established,
            "the buffered SYN should be answered as soon as the upstream is up"
        );
    }

    #[tokio::test]
    async fn connect_failure_resets_the_client() {
        let mut harness = Harness::new();
        let id = connection_id();
        let _detached = harness.stack.insert_connecting(id);

        harness.exchange();
        let _ = harness.egress_packets();

        harness.stack.handle_upstream_event(UpstreamMsg {
            id,
            event: UpstreamEvent::ConnectFailed,
        });
        harness.step();

        assert!(
            harness.egress_packets().iter().any(is_reset),
            "a client whose upstream is unreachable must be reset, not left hanging"
        );
        assert!(
            !harness.stack.is_tracking(&id),
            "the failed connection must not be left behind"
        );
    }

    #[tokio::test]
    async fn synack_is_retransmitted_on_the_virtual_clock() {
        let mut harness = Harness::new();
        let id = connection_id();
        harness.stack.insert_connecting(id);

        // Deliver the SYN, then answer it, but never let the client's ACK back through.
        harness.client.poll(harness.now());
        while let Ok(packet) = harness.client.egress_rx.try_recv() {
            harness
                .stack
                .handle_downstream_packet(Ipv4Packet::new_unchecked(packet));
        }
        harness.stack.handle_upstream_event(UpstreamMsg {
            id,
            event: UpstreamEvent::Connected,
        });
        harness.step();

        let mut syn_acks = harness
            .egress_packets()
            .iter()
            .filter(|p| is_syn_ack(p))
            .count();
        assert_eq!(syn_acks, 1, "the handshake should be answered once");

        for _ in 0..5 {
            harness.now_ms += 2_000;
            harness.step();
            syn_acks += harness
                .egress_packets()
                .iter()
                .filter(|p| is_syn_ack(p))
                .count();
        }

        assert!(
            syn_acks >= 2,
            "timer-driven retransmission must happen, got {syn_acks} SYN-ACKs"
        );
    }

    #[tokio::test]
    async fn client_payload_reaches_the_upstream() {
        let mut harness = Harness::new();
        let mut detached = harness.establish();

        harness
            .client
            .socket()
            .send_slice(b"GET / HTTP/1.1\r\n")
            .expect("the client can send");
        harness.exchange();

        let payload = detached
            .to_upstream_rx
            .try_recv()
            .expect("the payload should have been forwarded upstream");
        assert_eq!(&payload[..], b"GET / HTTP/1.1\r\n");
    }

    #[tokio::test]
    async fn upstream_payload_reaches_the_client() {
        let mut harness = Harness::new();
        let detached = harness.establish();

        detached
            .from_upstream_tx
            .try_send(Bytes::from_static(b"HTTP/1.1 200 OK\r\n"))
            .expect("the upstream channel has room");
        harness.exchange();

        let mut received = vec![0u8; 64];
        let read = harness
            .client
            .socket()
            .recv_slice(&mut received)
            .expect("the client can receive");
        assert_eq!(&received[..read], b"HTTP/1.1 200 OK\r\n");
    }

    /// Regression test for the reason payloads go through smoltcp at all.
    ///
    /// The client sends far more distinct segments than the upstream channel can hold while
    /// nothing drains that channel. Every byte must survive in smoltcp's receive buffer and come
    /// out in order once the upstream starts accepting again.
    #[tokio::test]
    async fn reap_does_not_drop_bytes_under_backpressure() {
        const TOTAL: usize = 200;
        const _: () = assert!(TOTAL > TCP_DATA_CHANNEL_CAPACITY);

        let mut harness = Harness::new();
        let mut detached = harness.establish();

        // One byte per segment, and one reap per byte, so the bounded channel fills long before
        // the client is done sending.
        for i in 0..TOTAL {
            assert_eq!(
                harness
                    .client
                    .socket()
                    .send_slice(&[i as u8])
                    .expect("the client can send"),
                1
            );
            harness.exchange();
        }

        let mut received: Vec<u8> = Vec::new();
        for _ in 0..(TOTAL * 4) {
            while let Ok(payload) = detached.to_upstream_rx.try_recv() {
                received.extend_from_slice(&payload);
            }
            if received.len() >= TOTAL {
                break;
            }
            harness.exchange();
        }

        let expected: Vec<u8> = (0..TOTAL).map(|i| i as u8).collect();
        assert_eq!(
            received, expected,
            "every byte the client sent must reach the upstream, in order and none dropped"
        );
    }

    #[tokio::test]
    async fn client_fin_half_closes_the_upstream() {
        let mut harness = Harness::new();
        let mut detached = harness.establish();

        harness
            .client
            .socket()
            .send_slice(b"trailing")
            .expect("the client can send");
        harness.client.socket().close();
        harness.exchange();

        let payload = detached
            .to_upstream_rx
            .try_recv()
            .expect("the last payload must be forwarded before the FIN is");
        assert_eq!(&payload[..], b"trailing");

        assert!(
            matches!(
                detached.to_upstream_rx.try_recv(),
                Err(mpsc::error::TryRecvError::Disconnected)
            ),
            "the client's FIN must shut down the upstream write half"
        );
    }

    #[tokio::test]
    async fn upstream_eof_closes_the_client() {
        let mut harness = Harness::new();
        let detached = harness.establish();

        detached
            .from_upstream_tx
            .try_send(Bytes::from_static(b"last"))
            .expect("the upstream channel has room");
        drop(detached.from_upstream_tx);
        harness.exchange();

        let mut received = vec![0u8; 64];
        let read = harness
            .client
            .socket()
            .recv_slice(&mut received)
            .expect("the client can receive");
        assert_eq!(
            &received[..read],
            b"last",
            "buffered data must be delivered before the FIN"
        );
        assert!(
            !harness.client.socket().may_recv(),
            "the upstream's EOF must be forwarded to the client as a FIN"
        );
    }
}
