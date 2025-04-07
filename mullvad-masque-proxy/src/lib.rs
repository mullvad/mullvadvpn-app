use h3::proto::varint::VarInt;

pub mod client;
mod fragment;
pub mod server;
mod stats;

const PACKET_BUFFER_SIZE: usize = 1700;
pub const HTTP_MASQUE_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(0);
pub const HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(1);
