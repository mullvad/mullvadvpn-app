use h3::proto::varint::VarInt;
use std::net::SocketAddr;

pub mod client;
mod fragment;
pub mod server;
mod stats;

pub const HTTP_MASQUE_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(0);
pub const HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(1);

/// Minimum size of buffer used to hold UDP packets.
// 1 byte for size of HTTP_MASQUE_DATAGRAM_CONTEXT_ID
const PACKET_BUFFER_SIZE: usize = (u16::MAX - UDP_HEADER_SIZE + 1) as usize;

/// Maximum number of inflight packets, in both directions.
const MAX_INFLIGHT_PACKETS: usize = 100;

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

/// Minimum allowed MTU (IPv6) is the overhead of all headers, plus 1 byte for actual data.
/// QUIC defines that clients must support UDP payloads of at least 1200 bytes.
/// <https://datatracker.ietf.org/doc/html/rfc9000#section-8.1>
// 20 = IPv4 header (without optional fields)
pub const MIN_IPV4_MTU: u16 = 20 + UDP_HEADER_SIZE + 1200;

/// Minimum allowed MTU (IPv6) is the overhead of all headers, plus 1 byte for actual data.
/// QUIC defines that clients must support UDP payloads of at least 1200 bytes.
/// <https://datatracker.ietf.org/doc/html/rfc9000#section-8.1>
// 40 = IPv6 header (without optional fields)
pub const MIN_IPV6_MTU: u16 = 40 + UDP_HEADER_SIZE + 1200;
