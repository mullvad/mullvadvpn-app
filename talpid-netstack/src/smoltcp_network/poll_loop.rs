//! The async poll loop and the [`SmoltcpStack`] state machine it drives.
//!
//! The loop owns the wall-clock and feeds timestamps into the stack, which is a
//! pure state machine. Each step the loop runs on every wakeup is a separate
//! method on [`SmoltcpStack`], so it can also be driven synchronously in tests.

use super::SmoltcpNetworkConfig;
use super::device::SmoltcpDevice;
use super::icmp_socket::SmoltcpIcmpSocket;
use super::tcp_stream::SmoltcpTcpStream;
use bytes::BytesMut;
use gotatun::packet::{Ip, Packet};
use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet},
    socket::{icmp, tcp},
    time::Instant as SmoltcpInstant,
    wire::{IpAddress, IpCidr, IpEndpoint, IpListenEndpoint},
};
use std::{
    collections::VecDeque,
    io,
    iter::Cycle,
    net::SocketAddr,
    ops::Range,
    sync::Arc,
    time::{Duration, Instant as StdInstant},
};
use tokio::sync::{Notify, mpsc, oneshot};

/// Channel capacity for TCP data between socket handles and the poll loop.
const TCP_DATA_CHANNEL_CAPACITY: usize = 64;
/// Channel capacity for ICMP between socket handles and the poll loop.
const ICMP_CHANNEL_CAPACITY: usize = 16;

/// Size of TCP socket receive and send buffers within smoltcp.
const TCP_BUFFER_SIZE: usize = 65535;
/// Number of ICMP packet metadata slots.
const ICMP_METADATA_SLOTS: usize = 8;
/// Size of ICMP packet buffer.
const ICMP_BUFFER_SIZE: usize = 2048;

pub(super) enum SocketCmd {
    TcpConnect {
        addr: SocketAddr,
        response: oneshot::Sender<io::Result<SmoltcpTcpStream>>,
    },
    CreateIcmpSocket {
        ident: u16,
        response: oneshot::Sender<io::Result<SmoltcpIcmpSocket>>,
    },
}

struct ActiveTcpSocket {
    socket_handle: SocketHandle,
    read_tx: mpsc::Sender<io::Result<BytesMut>>,
    write_rx: mpsc::Receiver<Vec<u8>>,
    write_buf: VecDeque<u8>,
}

struct ActiveIcmpSocket {
    socket_handle: SocketHandle,
    send_rx: mpsc::Receiver<(Vec<u8>, IpAddress)>,
}

fn smoltcp_now(reference: &StdInstant) -> SmoltcpInstant {
    SmoltcpInstant::from_millis(reference.elapsed().as_millis() as i64)
}

/// Sleep until the smoltcp poll deadline, or forever if there is none. An idle
/// stack needs no timer wakeups: commands, inbound packets, and `notify` all
/// wake the poll loop on their own.
async fn sleep_until(deadline: Option<Duration>) {
    match deadline {
        Some(delay) => tokio::time::sleep(delay).await,
        None => std::future::pending().await,
    }
}

fn to_smoltcp_endpoint(addr: SocketAddr) -> IpEndpoint {
    IpEndpoint {
        addr: match addr {
            SocketAddr::V4(v4) => IpAddress::Ipv4(*v4.ip()),
            SocketAddr::V6(v6) => IpAddress::Ipv6(*v6.ip()),
        },
        port: addr.port(),
    }
}

pub(super) async fn poll_loop(
    config: SmoltcpNetworkConfig,
    mut from_gotatun_rx: mpsc::Receiver<Packet<Ip>>,
    to_gotatun_tx: mpsc::Sender<Packet<Ip>>,
    mut cmd_rx: mpsc::Receiver<SocketCmd>,
    notify: Arc<Notify>,
) {
    // The loop owns the wall-clock; the stack is a pure state machine driven by
    // the timestamps we feed it.
    let reference = StdInstant::now();
    let now = || smoltcp_now(&reference);

    let mut stack = SmoltcpStack::new(&config, now(), to_gotatun_tx);

    loop {
        // Block until there's something to do: a socket command, an inbound
        // packet, an explicit wakeup, or smoltcp's next timer deadline (if any).
        let deadline = stack.poll_delay(now());
        tokio::select! {
            biased;
            Some(cmd) = cmd_rx.recv() => stack.handle_cmd(cmd, &notify),
            Some(pkt) = from_gotatun_rx.recv() => stack.enqueue_rx(pkt),
            _ = notify.notified() => {}
            _ = sleep_until(deadline) => {}
        }

        // Drain whatever else is already queued so a burst of commands or
        // packets is serviced by a single poll below, rather than one wakeup
        // (and one full `iface.poll`) per item.
        while let Ok(cmd) = cmd_rx.try_recv() {
            stack.handle_cmd(cmd, &notify);
        }
        while let Ok(pkt) = from_gotatun_rx.try_recv() {
            stack.enqueue_rx(pkt);
        }

        // One full servicing pass over the stack.
        stack.pump_tcp_writes();
        stack.pump_icmp_sends();
        stack.poll(now());
        stack.reap_tcp();
        stack.reap_icmp();
    }
}

/// Owns the smoltcp interface and the sockets created on it, and exposes each
/// step that [`poll_loop`] runs on every wakeup as a separate method.
struct SmoltcpStack {
    device: SmoltcpDevice,
    iface: Interface,
    sockets: SocketSet<'static>,
    active_tcp: Vec<ActiveTcpSocket>,
    active_icmp: Vec<ActiveIcmpSocket>,
    /// Next ephemeral source port to hand out for outgoing TCP connections.
    next_local_port: Cycle<Range<u16>>,
}

const LINUX_HIGH_PORT_RANGE: Range<u16> = 32768..61000;

impl SmoltcpStack {
    fn new(
        config: &SmoltcpNetworkConfig,
        now: SmoltcpInstant,
        to_gotatun_tx: mpsc::Sender<Packet<Ip>>,
    ) -> Self {
        let mut device = SmoltcpDevice::new(config.mtu, to_gotatun_tx);
        let iface_config = IfaceConfig::new(smoltcp::wire::HardwareAddress::Ip);
        let mut iface = Interface::new(iface_config, &mut device, now);

        iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::new(IpAddress::Ipv4(config.ipv4_addr), 32))
                .expect("enough space for IPv4 address");
            if let Some(v6) = config.ipv6_addr {
                addrs
                    .push(IpCidr::new(IpAddress::Ipv6(v6), 128))
                    .expect("enough space for IPv6 address");
            }
        });

        iface
            .routes_mut()
            .add_default_ipv4_route(smoltcp::wire::Ipv4Address::new(0, 0, 0, 1))
            .expect("enough space for default IPv4 route");
        if config.ipv6_addr.is_some() {
            iface
                .routes_mut()
                .add_default_ipv6_route(smoltcp::wire::Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 1))
                .expect("enough space for default IPv6 route");
        }

        Self {
            device,
            iface,
            sockets: SocketSet::new(Vec::new()),
            active_tcp: Vec::new(),
            active_icmp: Vec::new(),
            next_local_port: LINUX_HIGH_PORT_RANGE.cycle(),
        }
    }

    /// How long the poll loop may sleep before smoltcp next needs servicing,
    /// or `None` if no timer is pending.
    fn poll_delay(&mut self, now: SmoltcpInstant) -> Option<Duration> {
        self.iface
            .poll_delay(now, &self.sockets)
            .map(|d| Duration::from_millis(d.total_millis()))
    }

    /// Queue an inbound (decrypted) IP packet for smoltcp to process.
    fn enqueue_rx(&mut self, packet: Packet<Ip>) {
        self.device.enqueue_rx(packet.into_bytes());
    }

    /// Advance smoltcp: process all queued ingress and generate egress.
    fn poll(&mut self, now: SmoltcpInstant) {
        let _ = self.iface.poll(now, &mut self.device, &mut self.sockets);
    }

    /// Feed buffered writes from the TCP stream handles into their sockets.
    fn pump_tcp_writes(&mut self) {
        let sockets = &mut self.sockets;
        for tcp_sock in self.active_tcp.iter_mut() {
            // Drain channel into per-socket write buffer
            while let Ok(data) = tcp_sock.write_rx.try_recv() {
                tcp_sock.write_buf.extend(data.iter());
            }

            // Write from buffer into smoltcp socket
            let socket = sockets.get_mut::<tcp::Socket<'_>>(tcp_sock.socket_handle);
            while !tcp_sock.write_buf.is_empty() && socket.can_send() {
                let data = tcp_sock.write_buf.make_contiguous();
                match socket.send_slice(data) {
                    Ok(0) => break,
                    Ok(n) => {
                        tcp_sock.write_buf.drain(..n);
                    }
                    Err(e) => {
                        log::warn!("smoltcp tcp send error: {e}");
                        break;
                    }
                }
            }
        }
    }

    /// Feed buffered ICMP send requests into their sockets.
    fn pump_icmp_sends(&mut self) {
        let sockets = &mut self.sockets;
        for icmp_sock in self.active_icmp.iter_mut() {
            let socket = sockets.get_mut::<icmp::Socket<'_>>(icmp_sock.socket_handle);
            while socket.can_send() {
                match icmp_sock.send_rx.try_recv() {
                    Ok((data, dest)) => {
                        if let Err(e) = socket.send_slice(&data, dest) {
                            log::warn!("smoltcp icmp send error: {e}");
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    /// Push received TCP data up to the stream handles and drop dead sockets.
    fn reap_tcp(&mut self) {
        let sockets = &mut self.sockets;
        self.active_tcp.retain_mut(|tcp_sock| {
            let socket = sockets.get_mut::<tcp::Socket<'_>>(tcp_sock.socket_handle);

            // Push received data to the stream handle. Reserve a channel slot *before* draining
            // smoltcp.
            if socket.can_recv() && socket.recv_queue() > 0 {
                match tcp_sock.read_tx.try_reserve() {
                    Ok(permit) => {
                        let queue_len = socket.recv_queue();
                        let mut buf = BytesMut::zeroed(queue_len);
                        match socket.recv_slice(&mut buf) {
                            Ok(n) => {
                                if n > 0 {
                                    buf.truncate(n);
                                    permit.send(Ok(buf));
                                }
                            }
                            Err(e) => {
                                permit.send(Err(io::Error::other(e.to_string())));
                            }
                        }
                    }
                    // Channel full: leave the bytes in smoltcp and retry on the
                    // next poll, once the consumer has drained the channel.
                    Err(mpsc::error::TrySendError::Full(())) => {}
                    // Stream dropped: the cleanup below removes the socket.
                    Err(mpsc::error::TrySendError::Closed(())) => {}
                }
            }

            // Clean up closed sockets or sockets whose stream was dropped
            let state = socket.state();
            let stream_dropped = tcp_sock.read_tx.is_closed();
            if state == tcp::State::Closed || state == tcp::State::TimeWait || stream_dropped {
                if stream_dropped {
                    socket.close();
                }
                sockets.remove(tcp_sock.socket_handle);
                false
            } else {
                true
            }
        });
    }

    /// Drop ICMP sockets whose handle has been dropped.
    fn reap_icmp(&mut self) {
        self.active_icmp.retain(|s| !s.send_rx.is_closed());
    }

    fn handle_cmd(&mut self, cmd: SocketCmd, notify: &Arc<Notify>) {
        match cmd {
            SocketCmd::TcpConnect { addr, response } => {
                let rx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
                let tx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
                let mut tcp_socket = tcp::Socket::new(rx_buffer, tx_buffer);

                let local_port = self
                    .next_local_port
                    .next()
                    .expect("Cycle<Range> is expected to repeat forever");

                let local_endpoint = IpListenEndpoint {
                    addr: None,
                    port: local_port,
                };
                let remote_endpoint = to_smoltcp_endpoint(addr);

                let cx = self.iface.context();
                if let Err(e) = tcp_socket.connect(cx, remote_endpoint, local_endpoint) {
                    let _ =
                        response.send(Err(io::Error::other(format!("smoltcp connect error: {e}"))));
                    return;
                }

                let socket_handle = self.sockets.add(tcp_socket);

                let (read_tx, read_rx) = mpsc::channel(TCP_DATA_CHANNEL_CAPACITY);
                let (write_tx, write_rx) = mpsc::channel(TCP_DATA_CHANNEL_CAPACITY);

                // Return stream immediately. Reads pend until handshake completes
                // and data arrives. Writes buffer in the channel and are flushed
                // once can_send() returns true (after Established state).
                let stream = SmoltcpTcpStream::new(read_rx, write_tx, notify.clone());

                self.active_tcp.push(ActiveTcpSocket {
                    socket_handle,
                    read_tx,
                    write_rx,
                    write_buf: VecDeque::new(),
                });

                let _ = response.send(Ok(stream));
            }

            SocketCmd::CreateIcmpSocket { ident, response } => {
                let rx_buffer = icmp::PacketBuffer::new(
                    vec![icmp::PacketMetadata::EMPTY; ICMP_METADATA_SLOTS],
                    vec![0u8; ICMP_BUFFER_SIZE],
                );
                let tx_buffer = icmp::PacketBuffer::new(
                    vec![icmp::PacketMetadata::EMPTY; ICMP_METADATA_SLOTS],
                    vec![0u8; ICMP_BUFFER_SIZE],
                );
                let mut icmp_socket = icmp::Socket::new(rx_buffer, tx_buffer);

                if let Err(e) = icmp_socket.bind(icmp::Endpoint::Ident(ident)) {
                    let _ = response.send(Err(io::Error::other(format!(
                        "smoltcp icmp bind error: {e}"
                    ))));
                    return;
                }

                let socket_handle = self.sockets.add(icmp_socket);

                let (send_tx, send_rx) = mpsc::channel(ICMP_CHANNEL_CAPACITY);

                self.active_icmp.push(ActiveIcmpSocket {
                    socket_handle,
                    send_rx,
                });

                let _ = response.send(Ok(SmoltcpIcmpSocket::new(send_tx, notify.clone())));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotatun::packet::Ipv4;
    use std::net::Ipv4Addr;

    /// Retransmissions are driven by the timestamps fed to `poll`, not by real
    /// time. With no SYN-ACK arriving, advancing the virtual clock past the RTO
    /// and polling re-emits the SYN. This is the deterministic, instant
    /// counterpart to the old wall-clock-bound retransmission test, and it
    /// confirms the poll loop drives smoltcp independently of the stream ever
    /// being read (the stream is created but never polled).
    #[test]
    fn stack_retransmits_syn_on_virtual_clock() {
        fn count_syns(rx: &mut mpsc::Receiver<Packet<Ip>>) -> u32 {
            let mut syns = 0;
            while let Ok(pkt) = rx.try_recv() {
                let is_syn = Packet::<Ipv4>::try_from(pkt)
                    .and_then(|v4| v4.try_into_tcp())
                    .is_ok_and(|tcp| tcp.payload.header.syn());
                if is_syn {
                    syns += 1;
                }
            }
            syns
        }

        let mut now_ms: i64 = 0;
        let (tx, mut rx) = mpsc::channel(1024);
        let mut stack = SmoltcpStack::new(
            &SmoltcpNetworkConfig {
                ipv4_addr: Ipv4Addr::new(10, 0, 0, 1),
                ipv6_addr: None,
                mtu: 1420,
            },
            SmoltcpInstant::from_millis(now_ms),
            tx,
        );

        let notify = std::sync::Arc::new(tokio::sync::Notify::new());
        let (resp_tx, mut resp_rx) = tokio::sync::oneshot::channel();
        stack.handle_cmd(
            SocketCmd::TcpConnect {
                addr: "10.0.0.2:1337".parse().unwrap(),
                response: resp_tx,
            },
            &notify,
        );
        // Keep the stream alive (but never read it) so the socket isn't reaped.
        let _stream = resp_rx.try_recv().unwrap().unwrap();

        // Initial poll emits the first SYN; no SYN-ACK ever comes back.
        stack.poll(SmoltcpInstant::from_millis(now_ms));
        let mut syns = count_syns(&mut rx);

        // Jump the virtual clock well past each (backing-off) RTO; every poll
        // past a pending retransmit deadline re-emits the SYN.
        for _ in 0..5 {
            now_ms += 2_000;
            stack.poll(SmoltcpInstant::from_millis(now_ms));
            syns += count_syns(&mut rx);
        }

        assert!(
            syns >= 2,
            "expected initial SYN + at least one retransmission, got {syns}"
        );
    }

    /// The [`SmoltcpStack`] can be driven synchronously, without the async poll
    /// loop or any timeouts: issuing a `TcpConnect` and polling once makes
    /// smoltcp emit a SYN onto the outbound channel. This exercises the
    /// `handle_cmd` → `poll` → `transmit` seam directly.
    #[test]
    fn stack_connect_emits_syn() {
        let now = SmoltcpInstant::from_millis(0);
        let (tx, mut rx) = mpsc::channel(64);
        let mut stack = SmoltcpStack::new(
            &SmoltcpNetworkConfig {
                ipv4_addr: Ipv4Addr::new(10, 0, 0, 1),
                ipv6_addr: None,
                mtu: 1420,
            },
            now,
            tx,
        );
        let notify = std::sync::Arc::new(tokio::sync::Notify::new());
        let (resp_tx, _resp_rx) = tokio::sync::oneshot::channel();

        stack.handle_cmd(
            SocketCmd::TcpConnect {
                addr: "10.0.0.2:1337".parse().unwrap(),
                response: resp_tx,
            },
            &notify,
        );
        stack.poll(now);

        let saw_syn = std::iter::from_fn(|| rx.try_recv().ok())
            .filter_map(|pkt| {
                Packet::<Ipv4>::try_from(pkt)
                    .and_then(|v4| v4.try_into_tcp())
                    .ok()
            })
            .any(|tcp| tcp.payload.header.syn());
        assert!(saw_syn, "a TCP SYN packet should have been emitted");
    }

    /// Regression test for silent TCP data loss under read backpressure.
    ///
    /// A real smoltcp "server" interface is wired to our [`SmoltcpStack`] by
    /// shuttling packets between their devices. After the handshake, the server
    /// sends more distinct segments than the read channel can hold while the
    /// consumer never reads the stream.
    #[test]
    fn reap_tcp_does_not_drop_bytes_under_backpressure() {
        use smoltcp::{
            iface::{Config as IfaceConfig, Interface, SocketSet},
            socket::tcp,
            wire::{HardwareAddress, IpAddress, IpCidr, Ipv4Address},
        };

        // Move all packets server -> client.
        fn server_to_client(client: &mut SmoltcpStack, srv_rx: &mut mpsc::Receiver<Packet<Ip>>) {
            while let Ok(p) = srv_rx.try_recv() {
                client.device.enqueue_rx(p.into_bytes());
            }
        }

        // --- Client: the stack under test (10.0.0.1), connecting out ---
        let client_ref = StdInstant::now();
        let (client_tx, mut client_rx) = mpsc::channel(2048);
        let mut client = SmoltcpStack::new(
            &SmoltcpNetworkConfig {
                ipv4_addr: Ipv4Addr::new(10, 0, 0, 1),
                ipv6_addr: None,
                mtu: 1420,
            },
            smoltcp_now(&client_ref),
            client_tx,
        );
        let notify = std::sync::Arc::new(tokio::sync::Notify::new());
        let (resp_tx, mut resp_rx) = tokio::sync::oneshot::channel();
        client.handle_cmd(
            SocketCmd::TcpConnect {
                addr: "10.0.0.2:1337".parse().unwrap(),
                response: resp_tx,
            },
            &notify,
        );
        let mut stream = resp_rx.try_recv().unwrap().unwrap();

        // --- Server: a plain smoltcp interface (10.0.0.2) with a listener ---
        let srv_ref = StdInstant::now();
        let (srv_tx, mut srv_rx) = mpsc::channel(2048);
        let mut srv_device = SmoltcpDevice::new(1420, srv_tx);
        let mut srv_iface = Interface::new(
            IfaceConfig::new(HardwareAddress::Ip),
            &mut srv_device,
            smoltcp_now(&srv_ref),
        );
        srv_iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::new(IpAddress::Ipv4(Ipv4Addr::new(10, 0, 0, 2)), 32))
                .unwrap();
        });
        srv_iface
            .routes_mut()
            .add_default_ipv4_route(Ipv4Address::new(0, 0, 0, 1))
            .unwrap();
        let mut srv_sockets = SocketSet::new(Vec::new());
        let mut srv_tcp = tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0u8; 65535]),
            tcp::SocketBuffer::new(vec![0u8; 65535]),
        );
        srv_tcp.listen(1337).unwrap();
        srv_tcp.set_nagle_enabled(false);
        let srv_handle = srv_sockets.add(srv_tcp);

        // --- Handshake: shuttle SYN / SYN-ACK / ACK back and forth ---
        for _ in 0..16 {
            client.poll(smoltcp_now(&client_ref));
            while let Ok(p) = client_rx.try_recv() {
                srv_device.enqueue_rx(p.into_bytes());
            }
            srv_iface.poll(smoltcp_now(&srv_ref), &mut srv_device, &mut srv_sockets);
            server_to_client(&mut client, &mut srv_rx);
        }
        assert_eq!(
            srv_sockets.get::<tcp::Socket<'_>>(srv_handle).state(),
            tcp::State::Established,
            "server should be connected after handshake"
        );

        // --- Server sends TOTAL bytes, one segment each, and the client reaps
        // once per byte without the consumer ever reading the stream. That
        // forces one read-channel message per byte, so the bounded channel
        // (capacity TCP_DATA_CHANNEL_CAPACITY) fills long before all bytes are
        // delivered. The client's receive window stays wide open (65535), so
        // the server can send everything without waiting on ACKs. ---
        const TOTAL: usize = 200;
        const { assert!(TOTAL > TCP_DATA_CHANNEL_CAPACITY) };
        for i in 0..TOTAL {
            let srv = srv_sockets.get_mut::<tcp::Socket<'_>>(srv_handle);
            assert_eq!(srv.send_slice(&[i as u8]).unwrap(), 1);
            srv_iface.poll(smoltcp_now(&srv_ref), &mut srv_device, &mut srv_sockets);
            server_to_client(&mut client, &mut srv_rx);

            client.poll(smoltcp_now(&client_ref));
            client.reap_tcp();
            // Discard the client's outgoing ACKs; the server doesn't need them
            // to keep sending within the open window.
            while client_rx.try_recv().is_ok() {}
        }

        // --- Now drain the consumer. Each freed slot lets `reap_tcp` move more
        // out of smoltcp's receive buffer. Nothing may be lost. ---
        let mut received: Vec<u8> = Vec::new();
        for _ in 0..(TOTAL * 4) {
            while let Ok(msg) = stream.upstream_rx.try_recv() {
                received.extend(msg.expect("no io error on the stream"));
            }
            if received.len() >= TOTAL {
                break;
            }
            client.reap_tcp();
            while client_rx.try_recv().is_ok() {}
        }

        let expected: Vec<u8> = (0..TOTAL).map(|i| i as u8).collect();
        assert_eq!(
            received, expected,
            "every byte the server sent must be delivered in order, none dropped"
        );
    }
}
