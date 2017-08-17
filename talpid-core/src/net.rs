use std::net::{IpAddr, SocketAddr};

/// Represents a network layer IP address together with the transport layer protocol and port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    /// The address part of this endpoint, contains the IP and port.
    pub address: SocketAddr,
    /// The protocol part of this endpoint.
    pub protocol: TransportProtocol,
}

impl Endpoint {
    /// Constructs a new `Endpoint` from the given parameters.
    pub fn new<T: Into<IpAddr>>(address: T, port: u16, protocol: TransportProtocol) -> Self {
        Endpoint {
            address: SocketAddr::new(address.into(), port),
            protocol: protocol,
        }
    }
}

/// Representation of a transport protocol, either UDP or TCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TransportProtocol {
    /// Represents the UDP transport protocol.
    Udp,
    /// Represents the TCP transport protocol.
    Tcp,
}
