use std::net::{Ipv4Addr, Ipv6Addr};
use talpid_types::net::{Endpoint, TransportProtocol, wireguard};

/// Contains WireGuard server data needed to connect to a WireGuard endpoint
#[derive(Debug, Clone)]
pub struct MullvadEndpoint {
    pub peer: wireguard::PeerConfig,
    pub exit_peer: Option<wireguard::PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
}

impl MullvadEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        Endpoint::new(
            self.peer.endpoint.ip(),
            self.peer.endpoint.port(),
            TransportProtocol::Udp,
        )
    }
}
