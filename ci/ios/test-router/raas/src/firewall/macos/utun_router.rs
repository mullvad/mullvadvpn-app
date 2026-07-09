use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use ping_tokio::IcmpSocket;
use smoltcp::{
    phy::ChecksumCapabilities,
    wire::{IpProtocol, Ipv4Packet, Ipv4Repr, UdpPacket, UdpRepr},
};
use tokio::net::UdpSocket;
use tun_rs::AsyncDevice;

fn spawn_udp_receiver(
    tunnel_device: Arc<AsyncDevice>,
    upstream_socket: Arc<UdpSocket>,
    local_socket_address: SocketAddrV4,
) {
    tokio::spawn(async move {
        let mut buffer = vec![0u8; 2048];
        let mut return_buffer = vec![0u8; 2048];
        while let Ok((bytes_received, upstream_address)) =
            upstream_socket.recv_from(&mut buffer).await
        {

            println!("receiving return traffic for {}", local_socket_address);
            let IpAddr::V4(upstream_ip) = upstream_address.ip() else {
                log::error!("Received IPv6 upstream address from an IPv4 socket");
                continue;
            };

            let payload = &mut buffer[..bytes_received];
            let mut return_packet = Ipv4Packet::<&mut [u8]>::new_unchecked(&mut return_buffer[..]);

            let ipv4_repr = Ipv4Repr {
                src_addr: upstream_ip,
                dst_addr: *local_socket_address.ip(),
                next_header: IpProtocol::Udp,
                payload_len: bytes_received + smoltcp::wire::UDP_HEADER_LEN,
                hop_limit: 64,
            };

            ipv4_repr.emit(&mut return_packet, &ChecksumCapabilities::default());

            let udp_repr = UdpRepr {
                src_port: upstream_address.port(),
                dst_port: local_socket_address.port(),
            };

            let mut return_udp_packet = UdpPacket::new_unchecked(return_packet.payload_mut());
            udp_repr.emit(
                &mut return_udp_packet,
                &upstream_ip.into(),
                &(*local_socket_address.ip()).into(),
                bytes_received,
                |udp_payload_buffer| udp_payload_buffer.copy_from_slice(payload),
                &ChecksumCapabilities::default(),
            );
        }
    });
}

struct LiveIcmpSocket {
    tunnel_device: Arc<AsyncDevice>,
    upstream_socket: IcmpSocket,
}

#[derive(PartialEq, PartialOrd)]
struct ConnectionIdentifier {
    local_peer: SocketAddr,
    upstream_peer: SocketAddr,
}

impl ConnectionIdentifier {
    fn from_local_peer(local_peer: SocketAddr, upstream_peer: SocketAddr) -> Self {
        Self {
            local_peer,
            upstream_peer,
        }
    }
}

pub struct Router {
    tunnel_device: Arc<AsyncDevice>,
    udp_sockets: BTreeMap<SocketAddr, Arc<UdpSocket>>,
}

impl Router {
    fn new(tunnel_device: Arc<AsyncDevice>) -> Self {
        Self {
            tunnel_device,
            udp_sockets: Default::default(),
        }
    }

    pub fn spawn(device: AsyncDevice) {
        let tunnel_device = Arc::new(device);
        let mut router = Self::new(tunnel_device.clone());

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 2048];
            loop {
                let packet_bytes = match tunnel_device.recv(&mut buffer).await {
                    Ok(bytes_read) => &buffer[..bytes_read],
                    Err(err) => {
                        log::error!("Failed to read from tunnel device: {err}");
                        return;
                    }
                };

                let Ok(ipv4_packet) = Ipv4Packet::new_checked(packet_bytes) else {
                    log::error!("Received malformed IPv4 packet");
                    continue;
                };

                router.process_packet(ipv4_packet).await;
            }
        });
    }

    async fn process_packet(&mut self, ipv4_packet: Ipv4Packet<&[u8]>) {
        let source_address = ipv4_packet.src_addr();
        let destination_address = ipv4_packet.dst_addr();

        match ipv4_packet.next_header() {
            IpProtocol::Udp => {
                self.process_udp_packet(source_address, destination_address, ipv4_packet.payload())
                    .await;
            }
            IpProtocol::Tcp => {
                self.process_tcp_packet(source_address, destination_address, ipv4_packet.payload())
                    .await;
            }
            IpProtocol::Icmp => {
                self.process_icmp_packet(
                    source_address,
                    destination_address,
                    ipv4_packet.payload(),
                )
                .await;
            }
            _ => (),
        }
    }

    async fn process_udp_packet(
        &mut self,
        source_address: std::net::Ipv4Addr,
        destination_address: std::net::Ipv4Addr,
        udp_packet_bytes: &[u8],
    ) {
        if let Err(err) = self
            .process_udp_packet_inner(source_address, destination_address, udp_packet_bytes)
            .await
        {
            println!("Failed to process UDP packet: {err}");
        }
    }

    async fn process_udp_packet_inner(
        &mut self,
        source_address: std::net::Ipv4Addr,
        destination_address: std::net::Ipv4Addr,
        udp_packet_bytes: &[u8], // [ UDP header, payload]
    ) -> anyhow::Result<()> {
        println!("Processing packet from {} to {}", source_address, destination_address);
        let udp_header = UdpPacket::new_checked(udp_packet_bytes)?;
        let local_socket_address = SocketAddrV4::new(source_address, udp_header.src_port());
        let remote_socket_address =
            SocketAddr::new(destination_address.into(), udp_header.dst_port());

        let socket = match self.udp_sockets.get_mut(&local_socket_address.into()) {
            Some(socket) => socket.clone(),
            None => {
                let socket = UdpSocket::bind("0.0.0.0:0").await?;
                let socket = Arc::new(socket);
                self.udp_sockets
                    .insert(local_socket_address.into(), socket.clone());

                spawn_udp_receiver(
                    self.tunnel_device.clone(),
                    socket.clone(),
                    local_socket_address,
                );

                socket
            }
        };

        // Do not care about how many bytes were sent. Maybe we should?
        let _ = socket
            .send_to(udp_header.payload(), remote_socket_address)
            .await?;
        println!("Forwarded packet from {} to {}", local_socket_address, remote_socket_address);
        Ok(())
    }

    async fn process_tcp_packet(
        &self,
        source_address: std::net::Ipv4Addr,
        destination_address: std::net::Ipv4Addr,
        payload: &[u8],
    ) {
        return;
    }

    async fn process_icmp_packet(
        &self,
        source_address: std::net::Ipv4Addr,
        destination_address: std::net::Ipv4Addr,
        payload: &[u8],
    ) {
        return;
    }
}
