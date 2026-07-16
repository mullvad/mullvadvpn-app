use crate::lan_nat::tcp::virtual_device::{VirtualDevice, BUF_SIZE};
use crate::{DaemonCommand, DaemonEventSender, InternalDaemonEvent};
use bytes::{Bytes, BytesMut};
use etherparse::checksum::Sum16BitWords;
use etherparse::{
    Icmpv4Header, Icmpv4Type, IpNumber, Ipv4Header, Ipv6Header, NetHeaders, PacketHeaders,
    PayloadSlice, TcpHeader, TransportHeader, UdpHeader,
};
use futures::channel::oneshot::Canceled;
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::socket::tcp::{RecvError, Socket as SmolTcpSocket, SocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpCidr};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::{AsRawFd, RawFd};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use talpid_core::IpSink;
use talpid_core::mpsc::Sender;
use talpid_core::packet::{Ip, Packet};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, UdpSocket};
use tokio::sync::mpsc;
use tokio::task;
use tun::AsyncDevice;

type IcmpEchoId = u16;
type IcmpMap = Arc<RwLock<HashMap<IcmpEchoId, Arc<UdpSocket>>>>;
type TcpMap = Arc<RwLock<HashMap<Connection, mpsc::Sender<Bytes>>>>;
type UdpMap = Arc<RwLock<HashMap<Connection, Arc<UdpSocket>>>>;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
struct Connection {
    src_addr: IpAddr,
    src_port: u16,

    dest_addr: IpAddr,
    dest_port: u16,
}

pub struct LanPacketHandler {
    daemon_tx: DaemonEventSender<InternalDaemonEvent>,
    icmp_map: IcmpMap,
    udp_map: UdpMap,
    tcp_map: TcpMap,
}

impl LanPacketHandler {
    pub fn new(daemon_tx: DaemonEventSender<InternalDaemonEvent>) -> io::Result<Self> {
        Ok(LanPacketHandler {
            daemon_tx,
            icmp_map: IcmpMap::new(Default::default()),
            udp_map: UdpMap::new(Default::default()),
            tcp_map: TcpMap::new(Default::default()),
        })
    }
}

impl IpSink for LanPacketHandler {
    fn accept(&self, packet: &Packet<Ip>) -> bool {
        static WG_RELAY_IP: Ipv4Addr = Ipv4Addr::new(10, 64, 0, 1);

        match packet.destination() {
            Some(IpAddr::V4(addr)) => addr != WG_RELAY_IP && addr.is_private(),
            Some(IpAddr::V6(addr)) => addr.is_unique_local(),
            None => false,
        }
    }

    fn consume(
        &self,
        packet: Packet<Ip>,
        tun_dev: Arc<AsyncDevice>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.route_packet(packet, tun_dev).await;
        })
    }
}

struct OutgoingPacket<'a> {
    src: IpAddr,
    dest: IpAddr,
    payload: PayloadSlice<'a>,
}

impl OutgoingPacket<'_> {
    fn create_raw_packet(&self, header: &[u8]) -> Vec<u8> {
        let payload = self.payload.slice();
        let mut buffer = Vec::with_capacity(header.len() + payload.len());
        buffer.extend_from_slice(header);
        buffer.extend_from_slice(payload);
        buffer
    }
}

impl LanPacketHandler {
    async fn route_packet(&self, packet: Packet<Ip>, sender: Arc<AsyncDevice>) {
        let bytes: BytesMut = packet.into_bytes().into();
        let bytes: Bytes = bytes.freeze();
        let bytes_clone = bytes.clone();

        let headers = PacketHeaders::from_ip_slice(bytes_clone.as_ref());

        match headers {
            Ok(headers) => {
                let (src, dest) = match headers.net.unwrap() {
                    NetHeaders::Ipv4(ip, _) => {
                        (IpAddr::from(ip.source), IpAddr::from(ip.destination))
                    }
                    NetHeaders::Ipv6(ip, _) => {
                        (IpAddr::from(ip.source), IpAddr::from(ip.destination))
                    }
                    NetHeaders::Arp(_) => {
                        log::trace!("Ignoring ARP packet");
                        return;
                    }
                };

                let outgoing = OutgoingPacket {
                    src,
                    dest,
                    payload: headers.payload,
                };

                match headers.transport {
                    Some(TransportHeader::Tcp(tcp)) => {
                        log::error!("routing tcp");
                        self.route_tcp(sender, tcp, outgoing, bytes).await;
                    }
                    Some(TransportHeader::Udp(udp)) => {
                        log::error!("routing udp");
                        self.route_udp(sender, udp, outgoing).await;
                    }
                    Some(TransportHeader::Icmpv4(icmp)) => {
                        log::error!("routing icmpv4");
                        self.route_icmp(sender, icmp, outgoing).await;
                    }
                    Some(TransportHeader::Icmpv6(icmp)) => {
                        log::error!("ICMPv6");
                        log::error!("{src} -> {dest}");
                        log::error!("{:?}", icmp);
                    }
                    None => {
                        log::error!("No transport header");
                    }
                }
            }
            Err(err) => {
                log::error!("Parse error: {:?}", err);
            }
        }
    }

    async fn route_tcp(
        &self,
        sender: Arc<AsyncDevice>,
        header: TcpHeader,
        packet: OutgoingPacket<'_>,
        ip_packet_bytes: Bytes,
    ) {
        let conn = Connection {
            src_addr: packet.src,
            src_port: header.source_port,
            dest_addr: packet.dest,
            dest_port: header.destination_port,
        };

        log::error!("{:?}", conn);

        if !self.tcp_map.read().unwrap().contains_key(&conn) {
            let prot_sock = create_tcp_socket(&packet).unwrap();

            log::error!("bypassing tcp socket");
            if self.bypass_socket(prot_sock.as_raw_fd()).await.is_err() {
                // TODO: error handling
                log::error!("failed to bypass tcp socket");
                return;
            }

            let (tx, rx) = mpsc::channel::<Bytes>(100);

            self.tcp_map.write().unwrap().insert(conn.clone(), tx);

            self.start_tcp_read_task(sender, conn.clone(), prot_sock, rx)
        }

        let sender = self.tcp_map.read().unwrap().get(&conn).cloned().unwrap();
        let _ = sender.send(ip_packet_bytes).await;
    }

    async fn route_udp(
        &self,
        sender: Arc<AsyncDevice>,
        header: UdpHeader,
        packet: OutgoingPacket<'_>,
    ) {
        let conn = Connection {
            src_addr: packet.src,
            src_port: header.source_port,
            dest_addr: packet.dest,
            dest_port: header.destination_port,
        };

        log::error!("{:?}", conn);

        if !self.udp_map.read().unwrap().contains_key(&conn) {
            let socket = create_udp_socket(&packet, UdpSocketProtocol::UDP).unwrap();

            log::error!("bypassing udp socket");
            if self.bypass_socket(socket.as_raw_fd()).await.is_err() {
                // TODO: error handling
                log::error!("failed to bypass udp socket");
                return;
            }

            let socket = Arc::new(socket);
            self.udp_map
                .write()
                .unwrap()
                .insert(conn.clone(), socket.clone());

            self.start_udp_read_task(sender, conn.clone(), socket);
        }

        let socket = self.udp_map.read().unwrap().get(&conn).cloned().unwrap();

        let sock_addr = match packet.dest {
            IpAddr::V4(dest) => {
                let dest_addr = SocketAddrV4::new(dest, conn.dest_port);
                SocketAddr::V4(dest_addr)
            }
            IpAddr::V6(dest) => {
                let dest_addr = SocketAddrV6::new(dest, conn.dest_port, 0, 0);
                SocketAddr::V6(dest_addr)
            }
        };

        let packet_bytes = packet.create_raw_packet(&header.to_bytes());

        match socket.send_to(&packet_bytes, &sock_addr).await {
            Ok(ok) => {
                log::error!("SENT UDP LEN: {}", ok);
            }
            Err(e) => {
                log::error!("ERROR SENDING ICMP: {:?}", e);
            }
        }
    }

    async fn route_icmp(
        &self,
        sender: Arc<AsyncDevice>,
        header: Icmpv4Header,
        packet: OutgoingPacket<'_>,
    ) {
        let Icmpv4Type::EchoRequest(echo_req) = header.icmp_type else {
            log::trace!("Ignoring ICMP packet that is not of type EchoRequest");
            return;
        };

        log::error!("ICMP echo request: {} -> {}", packet.src, packet.dest);
        log::error!("{:?}", header);

        if !self.icmp_map.read().unwrap().contains_key(&echo_req.id) {
            let socket = create_udp_socket(&packet, UdpSocketProtocol::ICMP).unwrap();

            if self.bypass_socket(socket.as_raw_fd()).await.is_err() {
                // TODO: error handling
                log::error!("failed to bypass icmp socket");
                return;
            }

            let socket = Arc::new(socket);
            self.icmp_map
                .write()
                .unwrap()
                .insert(echo_req.id, socket.clone());

            self.start_icmp_read_task(sender, echo_req.id, socket, packet.src);
        }

        let socket = self
            .icmp_map
            .read()
            .unwrap()
            .get(&echo_req.id)
            .cloned()
            .unwrap();

        let sock_addr = match packet.dest {
            IpAddr::V4(dest) => {
                let dest_addr = SocketAddrV4::new(dest, 0);
                SocketAddr::from(dest_addr)
            }
            IpAddr::V6(dest) => {
                let dest_addr = SocketAddrV6::new(dest, 0, 0, 0);
                SocketAddr::from(dest_addr)
            }
        };

        let packet_bytes = packet.create_raw_packet(&header.to_bytes());

        match socket.send_to(&packet_bytes, &sock_addr).await {
            Ok(ok) => {
                log::error!("SENT ICMP LEN: {}", ok);
            }
            Err(e) => {
                log::error!("ERROR SENDING ICMP: {:?}", e);
            }
        }
    }

    fn start_tcp_read_task(
        &self,
        tun: Arc<AsyncDevice>,
        conn: Connection,
        prot_sock: TcpSocket,
        mut tun_packets: mpsc::Receiver<Bytes>,
    ) {
        let mut device = VirtualDevice::new();
        let config = Config::new(HardwareAddress::Ip);
        let mut iface = Interface::new(config, &mut device, Instant::now());

        iface.update_ip_addrs(|ip_addrs| {
            // A /32 CIDR means this interface specifically owns this single IP address
            ip_addrs
                .push(IpCidr::new(conn.dest_addr.into(), 32))
                .unwrap();
        });

        let mut tcp_socket = SmolTcpSocket::new(
            SocketBuffer::new(vec![0; BUF_SIZE]),
            SocketBuffer::new(vec![0; BUF_SIZE]),
        );
        tcp_socket.listen(conn.dest_port).unwrap();

        let mut socket_set = SocketSet::new(vec![]);
        let socket_handle = socket_set.add(tcp_socket);

        // Buffer for moving data.
        let mut read_buf = vec![0; BUF_SIZE];

        // Elastic buffer to prevent dropping data if prot_sock is faster than smoltcp.
        let mut prot_to_smoltcp_buf = Vec::new();

        task::spawn(async move {
            let loop_start = std::time::Instant::now();

            let mut prot_sock = prot_sock
                .connect(
                    format!("{}:{}", conn.dest_addr, conn.dest_port)
                        .parse()
                        .unwrap(),
                )
                .await
                .unwrap();

            loop {
                // Send pending bytes read from the protected socket into smoltcp.
                let socket = socket_set.get_mut::<SmolTcpSocket<'_>>(socket_handle);
                if socket.can_send() && !prot_to_smoltcp_buf.is_empty() {
                    match socket.send_slice(&prot_to_smoltcp_buf) {
                        Ok(written) => {
                            // Remove only the bytes smoltcp accepted (leaves the rest for next time)
                            prot_to_smoltcp_buf.drain(..written);
                        }
                        Err(e) => log::error!("smoltcp send error: {e:?}"),
                    }
                }

                // Poll interface to drive the smoltcp state machine forward.
                iface.poll(Instant::now(), &mut device, &mut socket_set);

                // Drain the device TX Queue (smoltcp -> TUN device).
                while let Some(raw_ip_packet) = device.tx_queue.pop_front() {
                    match tun.send(&raw_ip_packet).await {
                        Ok(_) => device.recycle_buffer(raw_ip_packet),
                        Err(e) => {
                            log::error!("Failed to send to TUN: {e}");
                            return; // Exit task if TUN is broken
                        }
                    }
                }

                // Send packets from smoltcp -> protected socket.
                let socket = socket_set.get_mut::<SmolTcpSocket<'_>>(socket_handle);
                while socket.can_recv() {
                    match socket.recv_slice(&mut read_buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Err(e) = prot_sock.write_all(&read_buf[..n]).await {
                                log::error!("Failed to write to prot_sock: {e}");
                                return;
                            }
                        }
                        Err(RecvError::Finished) => {
                            log::info!("Remote peer closed the connection gracefully.");
                            // Close the protected socket half if necessary
                        }
                        Err(RecvError::InvalidState) => {
                            log::info!("Socket is disconnected or in an invalid state.");
                        }
                    }
                }

                // If, at this point, we have unsent buffered data from the protected socket we need
                // to set the delay to zero so that it is guaranteed to be processed in the next
                // iteration of the loop. If we don't do this we could potentially deadlock in the
                // select.
                let delay = if prot_to_smoltcp_buf.is_empty() {
                    iface.poll_delay(Instant::now(), &socket_set)
                } else {
                    Some(smoltcp::time::Duration::ZERO)
                };

                // Only read more from prot_sock when all pending data has been written to smoltcp.
                let should_read_prot_sock = prot_to_smoltcp_buf.is_empty();

                tokio::select! {
                    // A new packet arrived from the TUN interface.
                    Some(packet) = tun_packets.recv() => {
                        device.rx_queue.push_back(packet);
                    }

                    // Data arrived from the protected socket.
                    result = prot_sock.read(&mut read_buf), if should_read_prot_sock => {
                        match result {
                            Ok(0) => {
                                log::info!("Protected socket reached EOF.");
                                // Trigger graceful close on smoltcp side.
                                let socket = socket_set.get_mut::<SmolTcpSocket<'_>>(socket_handle);
                                socket.close();
                            }
                            Ok(n) => {
                                // Buffer it. It will be flushed to smoltcp at the start of the next loop.
                                prot_to_smoltcp_buf.extend_from_slice(&read_buf[..n]);
                            }
                            Err(e) => {
                                log::error!("prot_sock read error: {e:?}");
                                return;
                            }
                        }
                    }

                    // smoltcp timer expired (e.g., TCP retransmission needed).
                    _ = async {
                        if let Some(d) = delay {
                            tokio::time::sleep(d.into()).await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    } => ()
                }

                log::error!("time: {} ms", loop_start.elapsed().as_millis());
            }
        });
    }

    fn start_udp_read_task(
        &self,
        sender: Arc<AsyncDevice>,
        entry: Connection,
        socket: Arc<UdpSocket>,
    ) {
        task::spawn(async move {
            let mut packet = vec![0u8; 1500];

            loop {
                let (size, sock_addr) = socket.recv_from(&mut packet).await.unwrap();

                log::error!("READ {} bytes from socket, addr: {:?}", size, sock_addr);

                let (mut header, payload) = match UdpHeader::from_slice(&packet) {
                    Ok(header) => header,
                    Err(e) => {
                        log::error!("Error parsing UDP header: #{:?}", e);
                        continue;
                    }
                };

                log::error!("pre header: #{:?}", header);

                // let Some(src_ip) = sock_addr.as_socket().map(|sa| sa.ip()) else {
                //     log::error!("Received {} bytes from a non-IP socket", size);
                //     continue;
                // };

                // We are receiving a packet so the entry's src/dest ports are flipped.

                let src_addr = entry.dest_addr;
                let dest_addr = entry.src_addr;

                header.source_port = entry.dest_port;
                header.destination_port = entry.src_port;

                header.checksum = match (src_addr, dest_addr) {
                    (IpAddr::V4(src), IpAddr::V4(dest)) => header
                        .calc_checksum_ipv4_raw(src.octets(), dest.octets(), payload)
                        .unwrap(),
                    (IpAddr::V6(src), IpAddr::V6(dest)) => header
                        .calc_checksum_ipv6_raw(src.octets(), dest.octets(), payload)
                        .unwrap(),
                    _ => {
                        log::error!("invalid ipv4/v6 address combination");
                        continue;
                    }
                };

                log::error!("pre bytes:");
                for b in packet.iter().take(8) {
                    log::error!("{:02x} ", b);
                }

                // Reborrow the slice. `writer` will be advanced, but `packet` will remain untouched
                // and will still point to the full-length buffer.
                let mut writer = &mut packet[..];
                header.write(&mut writer).expect("failed to write header");

                log::error!("post bytes:");
                for b in packet.iter().take(8) {
                    log::error!("{:02x} ", b);
                }

                let (header, _) = match UdpHeader::from_slice(&packet) {
                    Ok(header) => header,
                    Err(e) => {
                        log::error!("Error parsing UDP header: #{:?}", e);
                        continue;
                    }
                };

                log::error!("post header: #{:?}", header);

                match create_tun_packet(&packet, src_addr, dest_addr, IpNumber::UDP) {
                    Ok(packet) => {
                        if let Err(e) = sender.send(packet.as_slice()).await {
                            log::error!("UDP Packet sent to tun device error: {:?}", e);
                        } else {
                            log::error!("UDP Packet sent to tun device");
                        }
                    }
                    Err(e) => log::error!("Error creating TUN UDP packet: {}", e),
                }
            }
        });
    }

    fn start_icmp_read_task(
        &self,
        sender: Arc<AsyncDevice>,
        id: IcmpEchoId,
        socket: Arc<UdpSocket>,
        tun_ip: IpAddr,
    ) {
        task::spawn(async move {
            let mut packet = vec![0u8; 1500];

            loop {
                let (size, sock_addr) = socket.recv_from(&mut packet).await.unwrap();

                if size < 8 {
                    continue;
                }

                log::error!(
                    "READ {} bytes from socket, addr: {:?}",
                    packet.len(),
                    sock_addr
                );

                let (src_ip, protocol) = match sock_addr {
                    sa @ SocketAddr::V4(_) => (sa.ip(), IpNumber::ICMP),
                    sa @ SocketAddr::V6(_) => {
                        log::error!("Received {} bytes from a IPV6 socket", size);
                        log::error!("tun IP: {tun_ip}");
                        (sa.ip(), IpNumber::IPV6_ICMP)
                    }
                };

                // 1. Rewrite the ICMP ID (Bytes 4 and 5)
                let id_bytes = id.to_be_bytes();
                packet[4] = id_bytes[0];
                packet[5] = id_bytes[1];

                // 2. Clear the old checksum (Bytes 2 and 3)
                packet[2] = 0;
                packet[3] = 0;

                let checksum = Sum16BitWords::new().add_slice(&packet).ones_complement();

                // Sum16BitWords has already converted to big endian, so get bytes as is.
                let checksum_bytes = checksum.to_ne_bytes();

                packet[2] = checksum_bytes[0];
                packet[3] = checksum_bytes[1];

                match create_tun_packet(&packet, src_ip, tun_ip, protocol) {
                    Ok(packet) => {
                        if let Err(e) = sender.send(packet.as_slice()).await {
                            log::error!("ICMP Packet sent to tun device error: {:?}", e);
                        } else {
                            log::error!("ICMP Packet sent to tun device");
                        }
                    }
                    Err(e) => log::error!("Error creating TUN ICMP packet: {}", e),
                }
            }
        });
    }

    async fn bypass_socket(&self, socket_raw_fd: RawFd) -> Result<(), Canceled> {
        let (bypass_tx, bypass_rx) = futures::channel::oneshot::channel();
        let event =
            InternalDaemonEvent::Command(DaemonCommand::BypassLanSocket(socket_raw_fd, bypass_tx));

        self.daemon_tx.send(event).map_err(|_| Canceled {})?;
        bypass_rx.await
    }
}

pub fn create_tun_packet(
    payload: &[u8],
    src: IpAddr,
    dest: IpAddr,
    protocol: IpNumber,
) -> Result<Vec<u8>, String> {
    // Allocate space for the max IP header (40 bytes for IPv6) + payload length
    let mut packet = Vec::with_capacity(40 + payload.len());

    match (src, dest) {
        (IpAddr::V4(src), IpAddr::V4(dest)) => {
            let ip_header = Ipv4Header::new(
                payload.len() as u16,
                64,
                protocol,
                src.octets(),
                dest.octets(),
            )
            .map_err(|e| format!("Invalid IPv4 header parameters: {:?}", e))?;

            log::error!("IP HEADERS: {:?}", ip_header);

            // Write the header.
            // etherparse's `write` method automatically computes and sets the IPv4 checksum.
            ip_header.write(&mut packet).map_err(|e| e.to_string())?;
        }
        (IpAddr::V6(src_v6), IpAddr::V6(dst_v6)) => {
            let ip_header = Ipv6Header {
                source: src_v6.octets(),
                destination: dst_v6.octets(),
                payload_length: payload.len() as u16,
                next_header: protocol,
                hop_limit: 64,
                ..Default::default()
            };

            // Write the IPv6 header.
            ip_header.write(&mut packet).map_err(|e| e.to_string())?;
        }
        _ => {
            return Err("IP version mismatch between source socket and TUN IP".into());
        }
    }

    // Append the original ICMP packet directly after the IP header
    packet.extend_from_slice(payload);

    Ok(packet)
}

fn create_udp_socket(
    packet: &OutgoingPacket<'_>,
    protocol: UdpSocketProtocol,
) -> io::Result<UdpSocket> {
    let is_ipv6 = packet.dest.is_ipv6();

    let socket_protocol = match protocol {
        UdpSocketProtocol::UDP => Protocol::UDP,
        UdpSocketProtocol::ICMP if is_ipv6 => Protocol::ICMPV6,
        UdpSocketProtocol::ICMP => Protocol::ICMPV4,
    };

    let socket = if is_ipv6 {
        Socket::new(Domain::IPV6, Type::DGRAM, Some(socket_protocol))?
    } else {
        Socket::new(Domain::IPV4, Type::DGRAM, Some(socket_protocol))?
    };

    socket.set_nonblocking(true)?;

    let socket: std::net::UdpSocket = socket.into();

    Ok(UdpSocket::from_std(socket)?)
}

fn create_tcp_socket(packet: &OutgoingPacket<'_>) -> io::Result<TcpSocket> {
    let socket = if packet.dest.is_ipv6() {
        Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?
    } else {
        Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?
    };

    socket.set_nonblocking(true)?;

    let socket: std::net::TcpStream = socket.into();

    Ok(TcpSocket::from_std_stream(socket))
}

enum UdpSocketProtocol {
    UDP,
    ICMP,
}
