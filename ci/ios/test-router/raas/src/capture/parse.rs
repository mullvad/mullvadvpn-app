// use packet::ip::{v4, v6, Packet};
use pcap_file_tokio::{
    pcap::{PcapPacket, PcapReader},
    PcapError,
};
use pnet_packet::{
    ethernet::EthernetPacket,
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    tcp::TcpPacket,
    udp::UdpPacket,
    Packet,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

pub async fn parse_pcap<F: tokio::io::AsyncRead + Unpin>(
    file: F,
    peer_addrs: BTreeSet<IpAddr>,
) -> Result<Vec<Connection>, PcapError> {
    let mut reader = PcapReader::new(file).await?;
    let mut connections = ParsedConnections::new(peer_addrs);

    while let Some(block) = reader.next_packet().await {
        match block {
            Ok(block) => {
                connections.parse_pcap_packet(&block);
            }
            Err(err) => {
                log::error!("Failed to parse a packet: {err}");
                continue;
            }
        }
    }

    Ok(connections.to_vec())
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Connection {
    #[serde(flatten)]
    pub id: ConnectionId,
    pub packets: Vec<PacketTransmission>,
}

#[derive(serde::Serialize, PartialOrd, Hash, PartialEq, Clone, Copy, Ord, Eq, Debug)]
pub struct ConnectionId {
    pub peer_addr: SocketAddr,
    pub other_addr: SocketAddr,
    pub flow_id: Option<u32>,
    pub transport_protocol: TransportProtocol,
}

#[derive(serde::Serialize, Clone, Copy, Debug)]
pub struct PacketTransmission {
    from_peer: bool,
    timestamp: u64,
}

#[derive(Default, Debug)]
struct ParsedConnections {
    /// Peer addresses, only packets associated with these addreseses will be accounted for.
    /// TODO: reconsider the name peer in this context
    peer_addrs: BTreeSet<IpAddr>,
    /// The connections are mapped to a tuple of the peer address, associated address and the
    /// transport protocol. The behavior is undefined for peer to peer connections, it is assumed
    /// peers will never need to send traffic amongst themselves.
    connections: BTreeMap<ConnectionId, Connection>,
}

impl ParsedConnections {
    fn new(peer_addrs: BTreeSet<IpAddr>) -> Self {
        Self {
            peer_addrs,
            connections: Default::default(),
        }
    }

    fn parse_pcap_packet(&mut self, packet: &PcapPacket<'_>) {
        let timestamp =
            packet.timestamp.as_secs() * 1_000_000 + packet.timestamp.subsec_nanos() as u64 / 1000;
        if packet.data.len() < 3 {
            return;
        }
        // Parse the ethernet packet and truncate the pcap header.
        let Some(eth_packet) = EthernetPacket::new(&packet.data[2..]) else {
            return;
        };
        if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
            self.parse_ip_packet(&ipv4_packet, timestamp);
            return;
        }

        if let Some(ipv6_packet) = Ipv6Packet::new(eth_packet.payload()) {
            self.parse_ip_packet(&ipv6_packet, timestamp);
        }
    }

    fn parse_ip_packet(&mut self, packet: &dyn IpPacket, timestamp: u64) {
        // if packet is not associated with any of our peers, we do not care about it
        let source = packet.source();
        let destination = packet.destination();

        if !self.ip_matches_peer(source) && !self.ip_matches_peer(destination) {
            return;
        }

        let transport_protocol = packet.transport_protocol();
        let Some((source_port, destination_port)) =
            packet_ports(packet.payload(), transport_protocol)
        else {
            log::debug!("Failed to parse an IP packet from {source} to {destination}");
            return;
        };

        let (peer_addr, other_addr) = if self.ip_matches_peer(source) {
            (
                SocketAddr::new(source, source_port),
                SocketAddr::new(destination, destination_port),
            )
        } else {
            (
                SocketAddr::new(destination, destination_port),
                SocketAddr::new(source, source_port),
            )
        };

        let connection_id = ConnectionId {
            peer_addr,
            other_addr,
            flow_id: packet.flow_id(),
            transport_protocol,
        };

        let packet_transmission = PacketTransmission {
            from_peer: self.ip_matches_peer(source),
            timestamp,
        };

        self.connections
            .entry(connection_id)
            .and_modify(|c| {
                c.packets.push(packet_transmission);
            })
            .or_insert_with(|| Connection {
                id: connection_id,
                packets: vec![packet_transmission],
            });
    }

    fn ip_matches_peer(&self, ip: impl Into<IpAddr>) -> bool {
        let ip = ip.into();
        self.peer_addrs.iter().any(|peer| *peer == ip)
    }

    fn to_vec(&self) -> Vec<Connection> {
        self.connections.values().cloned().collect()
    }
}

/// Represents a layer 4 protocol
#[derive(serde::Serialize, PartialOrd, PartialEq, Hash, Clone, Copy, Eq, Ord, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    Icmp6,
    Unkown,
}

impl From<IpNextHeaderProtocol> for TransportProtocol {
    fn from(value: IpNextHeaderProtocol) -> Self {
        match value {
            IpNextHeaderProtocols::Udp => Self::Udp,
            IpNextHeaderProtocols::Tcp => Self::Tcp,
            IpNextHeaderProtocols::Icmp => Self::Icmp,
            IpNextHeaderProtocols::Icmpv6 => Self::Icmp6,
            _ => Self::Unkown,
        }
    }
}

trait IpPacket: pnet_packet::Packet {
    fn source(&self) -> IpAddr;
    fn destination(&self) -> IpAddr;
    fn transport_protocol(&self) -> TransportProtocol;
    fn flow_id(&self) -> Option<u32> {
        None
    }
}

impl<'a> IpPacket for Ipv4Packet<'a> {
    fn source(&self) -> IpAddr {
        self.get_source().into()
    }

    fn destination(&self) -> IpAddr {
        self.get_destination().into()
    }

    fn transport_protocol(&self) -> TransportProtocol {
        self.get_next_level_protocol().into()
    }
}

impl<'a> IpPacket for Ipv6Packet<'a> {
    fn source(&self) -> IpAddr {
        self.get_source().into()
    }

    fn destination(&self) -> IpAddr {
        self.get_destination().into()
    }

    fn transport_protocol(&self) -> TransportProtocol {
        self.get_next_header().into()
    }

    fn flow_id(&self) -> Option<u32> {
        Some(self.get_flow_label())
    }
}

/// Returns a tuple representing the source and destination ports for a given packet if the
/// transport protocol has ports.
fn packet_ports(payload: &[u8], transport_protocol: TransportProtocol) -> Option<(u16, u16)> {
    match transport_protocol {
        TransportProtocol::Tcp => {
            let packet = TcpPacket::new(payload)?;
            Some((packet.get_source(), packet.get_destination()))
        }
        TransportProtocol::Udp => {
            let packet = UdpPacket::new(payload)?;
            Some((packet.get_source(), packet.get_destination()))
        }
        _ => Some((0, 0)),
    }
}
