//! Socket bypass on Windows.
//!
//! Sockets are excluded from the firewall based on their *local* endpoint: the tunnel state
//! machine keeps a set of local endpoints and pushes it to winfw, which installs a filter
//! permitting outbound connections from each of them.
//!
//! # Limitations
//!
//! * A TCP socket has no local port until it is connected, and `ALE_AUTH_CONNECT` is classified
//!   *at* connect time. A TCP socket can therefore only be bypassed after its connection has been
//!   established, which means the handshake itself still relies on some other rule (in practice
//!   the relay `PermitEndpoint` rule).
//! * Routing is not affected. A bypassed socket still follows the routing table, so in the
//!   connected state a destination without an explicit non-tunnel route still goes into the
//!   tunnel.

use std::{
    collections::{HashMap, hash_map::Entry},
    io,
    sync::{Weak, mpsc as sync_mpsc},
    time::Duration,
};

use futures::channel::mpsc;
use socket2::{SockRef, Type};
use talpid_net::bypass::{BypassToken, SocketBypass};
use talpid_types::net::{Endpoint, TransportProtocol};

use super::TunnelCommand;

/// How long to wait for the tunnel state machine to apply the firewall exception before giving up.
const BYPASS_TIMEOUT: Duration = Duration::from_secs(5);

/// [SocketBypass] implementation that excludes sockets from the Windows firewall by asking the
/// tunnel state machine to permit their local endpoints.
pub struct WindowsSocketBypass {
    command_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
}

impl WindowsSocketBypass {
    pub fn new(command_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>) -> Self {
        Self { command_tx }
    }
}

impl SocketBypass for WindowsSocketBypass {
    fn bypass_socket(&self, socket: SockRef<'_>, _token: &BypassToken) -> io::Result<()> {
        let endpoint = endpoint_of(&socket)?;

        let command_tx = self
            .command_tx
            .upgrade()
            .ok_or_else(|| io::Error::other("The tunnel state machine is gone"))?;

        // NOTE: This blocks until the filter is in place. Returning any earlier would mean that
        // the first packets sent on the socket are dropped by the firewall.
        //
        // The state machine loop stays responsive while we wait, because the tunnel (and thus the
        // obfuscator) is created on a separate thread.
        let (reply_tx, reply_rx) = sync_mpsc::sync_channel(1);

        command_tx
            .unbounded_send(TunnelCommand::BypassSocket(endpoint, reply_tx))
            .map_err(|_| io::Error::other("The tunnel state machine is gone"))?;

        reply_rx
            .recv_timeout(BYPASS_TIMEOUT)
            .map_err(|_| io::Error::other("Timed out waiting for the socket bypass"))?
            .map_err(io::Error::other)
    }

    fn revoke_bypass(&self, socket: SockRef<'_>, _token: &BypassToken) -> io::Result<()> {
        let endpoint = endpoint_of(&socket)?;

        // If the state machine is gone, so is the firewall policy it applied.
        let Some(command_tx) = self.command_tx.upgrade() else {
            return Ok(());
        };

        command_tx
            .unbounded_send(TunnelCommand::RevokeSocketBypass(endpoint))
            .map_err(|_| io::Error::other("The tunnel state machine is gone"))
    }
}

/// Derive the local endpoint of `socket`, i.e. its local address and transport protocol.
fn endpoint_of(socket: &SockRef<'_>) -> io::Result<Endpoint> {
    let address = socket.local_addr()?.as_socket().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Socket is not bound to an IP address",
        )
    })?;

    // A socket without a local port cannot be expressed as a firewall filter. This happens for
    // TCP sockets that have not been connected yet.
    if address.port() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Socket is not bound to a port",
        ));
    }

    let protocol = match socket.r#type()? {
        Type::STREAM => TransportProtocol::Tcp,
        Type::DGRAM => TransportProtocol::Udp,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported socket type",
            ));
        }
    };

    Ok(Endpoint::from_socket_address(address, protocol))
}

/// A reference counted set of local endpoints that are excluded from the firewall.
///
/// Endpoints are reference counted because the same endpoint can legitimately be excluded more
/// than once: with `SO_REUSEADDR`, or when a dual-stack socket and an IPv4 socket end up on the
/// same port with different local addresses.
#[derive(Default)]
pub struct ExcludedSockets(HashMap<Endpoint, u32>);

impl ExcludedSockets {
    /// Add a reference to `endpoint`. Returns whether the set of endpoints changed.
    pub fn add(&mut self, endpoint: Endpoint) -> bool {
        match self.0.entry(endpoint) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() += 1;
                false
            }
            Entry::Vacant(entry) => {
                entry.insert(1);
                true
            }
        }
    }

    /// Remove a reference to `endpoint`. Returns whether the set of endpoints changed.
    pub fn remove(&mut self, endpoint: &Endpoint) -> bool {
        let Entry::Occupied(mut entry) = self.0.entry(*endpoint) else {
            return false;
        };

        if *entry.get() > 1 {
            *entry.get_mut() -= 1;
            return false;
        }

        entry.remove();
        true
    }

    /// Remove all endpoints. Returns whether the set of endpoints changed.
    pub fn clear(&mut self) -> bool {
        let changed = !self.0.is_empty();
        self.0.clear();
        changed
    }

    /// All currently excluded endpoints, in no particular order.
    pub fn endpoints(&self) -> impl ExactSizeIterator<Item = &Endpoint> {
        self.0.keys()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, TcpListener, UdpSocket};

    use super::*;

    #[test]
    fn test_endpoint_of_udp_v4() {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let endpoint = endpoint_of(&SockRef::from(&socket)).unwrap();

        assert_eq!(endpoint.address.ip(), Ipv4Addr::UNSPECIFIED);
        assert_ne!(endpoint.address.port(), 0);
        assert_eq!(endpoint.protocol, TransportProtocol::Udp);
    }

    #[test]
    fn test_endpoint_of_udp_v6() {
        let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();
        let endpoint = endpoint_of(&SockRef::from(&socket)).unwrap();

        assert_eq!(endpoint.address.ip(), Ipv6Addr::UNSPECIFIED);
        assert_ne!(endpoint.address.port(), 0);
        assert_eq!(endpoint.protocol, TransportProtocol::Udp);
    }

    #[test]
    fn test_endpoint_of_tcp() {
        let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let endpoint = endpoint_of(&SockRef::from(&socket)).unwrap();

        assert_eq!(endpoint.address.ip(), Ipv4Addr::LOCALHOST);
        assert_ne!(endpoint.address.port(), 0);
        assert_eq!(endpoint.protocol, TransportProtocol::Tcp);
    }

    /// An unbound socket has no local port, so it cannot be excluded.
    #[test]
    fn test_endpoint_of_unbound_socket() {
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .unwrap();

        endpoint_of(&SockRef::from(&socket)).unwrap_err();
    }

    fn endpoint(port: u16) -> Endpoint {
        Endpoint::new(Ipv4Addr::UNSPECIFIED, port, TransportProtocol::Udp)
    }

    #[test]
    fn test_excluded_sockets_refcount() {
        let mut sockets = ExcludedSockets::default();

        assert!(sockets.add(endpoint(1)));
        assert!(!sockets.add(endpoint(1)));
        assert_eq!(sockets.endpoints().len(), 1);

        // The endpoint is still referenced once, so the set is unchanged.
        assert!(!sockets.remove(&endpoint(1)));
        assert_eq!(sockets.endpoints().len(), 1);

        assert!(sockets.remove(&endpoint(1)));
        assert_eq!(sockets.endpoints().len(), 0);

        // Removing an endpoint that is not excluded is a no-op.
        assert!(!sockets.remove(&endpoint(1)));
    }

    #[test]
    fn test_excluded_sockets_distinct_endpoints() {
        let mut sockets = ExcludedSockets::default();

        assert!(sockets.add(endpoint(1)));
        assert!(sockets.add(endpoint(2)));
        assert_eq!(sockets.endpoints().len(), 2);

        assert!(sockets.remove(&endpoint(1)));
        assert_eq!(
            sockets.endpoints().copied().collect::<Vec<_>>(),
            vec![endpoint(2)]
        );
    }

    #[test]
    fn test_excluded_sockets_clear() {
        let mut sockets = ExcludedSockets::default();

        // Clearing an empty set does not change anything.
        assert!(!sockets.clear());

        sockets.add(endpoint(1));
        sockets.add(endpoint(1));
        sockets.add(endpoint(2));

        assert!(sockets.clear());
        assert_eq!(sockets.endpoints().len(), 0);
        assert!(!sockets.clear());
    }
}
