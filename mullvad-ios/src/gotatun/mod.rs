pub mod ip_mux;
pub mod smoltcp_network;

/// WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
pub const WIREGUARD_HEADER_SIZE: u16 = 8 + 32;
