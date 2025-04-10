use h3::proto::varint::VarInt;
use std::net::SocketAddr;

pub mod client;
mod fragment;
pub mod server;
mod stats;

const PACKET_BUFFER_SIZE: usize = 1700;
pub const HTTP_MASQUE_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(0);
pub const HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(1);

/// Fragment headers size for fragmented packets
const FRAGMENT_HEADER_SIZE_FRAGMENTED: u16 = 5;

/// UDP header overhead
const UDP_HEADER_SIZE: u16 = 8;

/// QUIC header size. This is conservative, real overhead varies
const QUIC_HEADER_SIZE: u16 = 41;

/// This is the size of the payload that stores QUIC packets
/// MTU - IP header - UDP header
const fn compute_udp_payload_size(mtu: u16, target_addr: SocketAddr) -> u16 {
    let ip_overhead = if target_addr.is_ipv4() { 20 } else { 40 };
    mtu - ip_overhead - UDP_HEADER_SIZE
}
