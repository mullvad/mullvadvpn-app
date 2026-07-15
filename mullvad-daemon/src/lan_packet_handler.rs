use crate::{DaemonCommand, DaemonEventSender, InternalDaemonEvent};
use etherparse::checksum::Sum16BitWords;
use etherparse::err::LenError;
use etherparse::{
    IcmpEchoHeader, Icmpv4Header, Icmpv4Slice, Icmpv4Type, IpNumber, Ipv4Header, Ipv6Header,
    NetHeaders, PacketHeaders, PayloadSlice, TransportHeader, UdpHeader,
};
use futures::channel::oneshot::Canceled;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::collections::HashMap;
use std::io;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use talpid_core::mpsc::Sender;
use talpid_core::packet::{Ip, Ipv4, Packet};
use talpid_core::{BufferedIpSend, IpSend, IpSink};
use tokio::net::UdpSocket;
use tokio::task;
use tun::AsyncDevice;

type IcmpEchoId = u16;
type IcmpMap = Arc<RwLock<HashMap<IcmpEchoId, Arc<UdpSocket>>>>;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
struct UdpEntry {
    src_addr: IpAddr,
    src_port: u16,

    dest_addr: IpAddr,
    dest_port: u16,
}
type UdpMap = Arc<RwLock<HashMap<UdpEntry, Arc<UdpSocket>>>>;

pub struct LanPacketHandler {
    daemon_tx: DaemonEventSender<InternalDaemonEvent>,
    icmp_map: IcmpMap,
    udp_map: UdpMap,
}

impl LanPacketHandler {
    pub fn new(daemon_tx: DaemonEventSender<InternalDaemonEvent>) -> io::Result<Self> {
        Ok(LanPacketHandler {
            daemon_tx,
            icmp_map: IcmpMap::new(Default::default()),
            udp_map: UdpMap::new(Default::default()),
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
        let bytes = packet.into_bytes();

        let byte_slice: &[u8] = bytes.as_ref();

        match PacketHeaders::from_ip_slice(byte_slice) {
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

                // self.update_tun_ip(src);

                let outgoing = OutgoingPacket {
                    src,
                    dest,
                    payload: headers.payload,
                };

                match headers.transport {
                    Some(TransportHeader::Tcp(tcp)) => {
                        log::error!("TCP");
                        log::error!("{src} -> {dest}");
                        log::error!("src port: {}", tcp.source_port);
                        log::error!("dst port: {}", tcp.destination_port);
                    }
                    Some(TransportHeader::Udp(udp)) => {
                        log::error!("routing udp");
                        self.route_udp(sender, udp, outgoing).await;
                    }
                    Some(TransportHeader::Icmpv4(icmp)) => {
                        log::error!("routing icmpv4");
                        self.route_icmp(sender, icmp, outgoing).await
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

    async fn route_udp(
        &self,
        sender: Arc<AsyncDevice>,
        header: UdpHeader,
        packet: OutgoingPacket<'_>,
    ) {
        let entry = UdpEntry {
            src_addr: packet.src,
            src_port: header.source_port,
            dest_addr: packet.dest,
            dest_port: header.destination_port,
        };

        log::error!("{:?}", entry);

        log::error!("IN MAP:");
        for key in self.udp_map.read().unwrap().keys() {
            log::error!("{:?}", key);
        }

        if !self.udp_map.read().unwrap().contains_key(&entry) {
            let socket = create_socket(&packet, SupportedProtocol::UDP).unwrap();

            log::error!("bypassing udp socket");
            if self.bypass_socket(&socket).await.is_err() {
                // TODO: error handling
                log::error!("failed to bypass udp socket");
                return;
            }

            let socket = Arc::new(socket);
            self.udp_map
                .write()
                .unwrap()
                .insert(entry.clone(), socket.clone());

            self.start_udp_read_task(sender, entry.clone(), socket);
        }

        let socket = self.udp_map.read().unwrap().get(&entry).cloned().unwrap();

        let sock_addr = match packet.dest {
            IpAddr::V4(dest) => {
                let dest_addr = SocketAddrV4::new(dest, entry.dest_port);
                SocketAddr::V4(dest_addr)
            }
            IpAddr::V6(dest) => {
                let dest_addr = SocketAddrV6::new(dest, entry.dest_port, 0, 0);
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

    fn start_udp_read_task(
        &self,
        sender: Arc<AsyncDevice>,
        entry: UdpEntry,
        socket: Arc<UdpSocket>,
    ) {
        task::spawn(async move {
            let mut packet = [0u8; 1500];

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

        log::error!("ICMPv4 echo request: {} -> {}", packet.src, packet.dest);
        log::error!("{:?}", header);

        if !self.icmp_map.read().unwrap().contains_key(&echo_req.id) {
            let socket = create_socket(&packet, SupportedProtocol::ICMP).unwrap();

            if self.bypass_socket(&socket).await.is_err() {
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

    fn start_icmp_read_task(
        &self,
        sender: Arc<AsyncDevice>,
        id: IcmpEchoId,
        socket: Arc<UdpSocket>,
        tun_ip: IpAddr,
    ) {
        task::spawn(async move {
            let mut packet = [0u8; 1500];

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

    async fn bypass_socket(&self, socket: &UdpSocket) -> Result<(), Canceled> {
        let (bypass_tx, bypass_rx) = futures::channel::oneshot::channel();
        let event = InternalDaemonEvent::Command(DaemonCommand::BypassLanSocket(
            socket.as_raw_fd(),
            bypass_tx,
        ));

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

fn create_socket(packet: &OutgoingPacket<'_>, protocol: SupportedProtocol) -> io::Result<UdpSocket> {
    let is_ipv6 = packet.dest.is_ipv6();

    let socket_protocol = match protocol {
        SupportedProtocol::UDP => Protocol::UDP,
        SupportedProtocol::TCP => Protocol::TCP,
        SupportedProtocol::ICMP if is_ipv6 => Protocol::ICMPV6,
        SupportedProtocol::ICMP => Protocol::ICMPV4,
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

/**
The protocols that we support LAN access for when block all VPN is enabled.
**/
enum SupportedProtocol {
    UDP,
    TCP,
    ICMP
}
