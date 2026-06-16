//! Tracks connections originating from the secondary (smoltcp) interface so
//! that the [`IpMuxRecv`](super::ip_mux::IpMuxRecv) can route matching return traffic
//! back to it.

use gotatun::packet::{Ip, IpNextProtocol, Ipv4, Ipv4Header, Ipv6, Tcp};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};
use zerocopy::{FromBytes, IntoBytes};

/// ICMPv4 echo request `type`.
const ICMPV4_ECHO_REQUEST: u8 = 8;
/// ICMPv4 echo reply `type`.
const ICMPV4_ECHO_REPLY: u8 = 0;
/// ICMPv6 echo request `type`.
const ICMPV6_ECHO_REQUEST: u8 = 128;
/// ICMPv6 echo reply `type`.
const ICMPV6_ECHO_REPLY: u8 = 129;

/// An opaque connection-tracking event produced from a secondary *outbound*
/// packet by [`outbound_conntrack_event`] and applied to the reader-owned
/// [`ConnectionTracker`] via [`ConnectionTracker::apply`].
pub(crate) struct ConnectionTrackerEvent(ConnectionUpdate);

enum ConnectionUpdate {
    TrackTcp(SockAddrPair),
    /// Outbound FIN: our half of the close. The entry is kept until the peer's
    /// half has been seen too, so the closing handshake still reaches smoltcp.
    LocalFin(SockAddrPair),
    /// Outbound RST: the connection is dead immediately.
    UntrackTcp(SockAddrPair),
    /// Track ICMP traffic for a given destination and an identity.
    TrackIcmp {
        dest: IpAddr,
        ident: u16,
    },
}

/// Inspects a secondary *outbound* packet and produces a corresponding tracking event, if
/// applicable
pub(crate) fn outbound_conntrack_event(ip: &Ip) -> Option<ConnectionTrackerEvent> {
    let ip = ParsedIp::parse(ip)?;

    let event = match ip.proto {
        IpNextProtocol::Tcp => tcp_tracking_event(ip.payload, ip.src, ip.dst),
        IpNextProtocol::Icmp => icmp_echo_ident(ip.payload, ICMPV4_ECHO_REQUEST).map(|ident| {
            ConnectionUpdate::TrackIcmp {
                dest: ip.dst,
                ident,
            }
        }),
        IpNextProtocol::Icmpv6 => icmp_echo_ident(ip.payload, ICMPV6_ECHO_REQUEST).map(|ident| {
            ConnectionUpdate::TrackIcmp {
                dest: ip.dst,
                ident,
            }
        }),
        _ => None,
    };

    event.map(ConnectionTrackerEvent)
}

/// The identifier of an ICMP echo packet of the given `type`, or `None` if the
/// payload isn't an echo of that type.
///
/// The ICMP echo header is `type(1) code(1) checksum(2) identifier(2) seq(2)`.
fn icmp_echo_ident(payload: &[u8], echo_type: u8) -> Option<u16> {
    let header: &[u8; 8] = payload.first_chunk()?;
    (header[0] == echo_type).then(|| u16::from_be_bytes([header[4], header[5]]))
}

/// The header fields shared by v4/v6 packets, extracted once so the
/// protocol-level matching below is written a single time.
struct ParsedIp<'a> {
    src: IpAddr,
    dst: IpAddr,
    proto: IpNextProtocol,
    /// The transport-layer payload (after any IPv4 options).
    payload: &'a [u8],
}

impl<'a> ParsedIp<'a> {
    fn parse(ip: &'a Ip) -> Option<Self> {
        match ip.header.version() {
            4 => {
                let ipv4 = Ipv4::<[u8]>::ref_from_bytes(ip.as_bytes()).ok()?;
                // `Ipv4::payload` starts after the fixed 20-byte header, so any
                // IPv4 options must be skipped to reach the transport payload.
                let options_len = usize::from(ipv4.header.ihl())
                    .checked_mul(4)?
                    .checked_sub(Ipv4Header::LEN)?;
                Some(ParsedIp {
                    src: IpAddr::from(ipv4.header.source()),
                    dst: IpAddr::from(ipv4.header.destination()),
                    proto: ipv4.header.next_protocol(),
                    payload: ipv4.payload.get(options_len..)?,
                })
            }
            6 => {
                let ipv6 = Ipv6::<[u8]>::ref_from_bytes(ip.as_bytes()).ok()?;
                Some(ParsedIp {
                    src: IpAddr::from(ipv6.header.source()),
                    dst: IpAddr::from(ipv6.header.destination()),
                    proto: ipv6.header.next_protocol(),
                    payload: &ipv6.payload,
                })
            }
            _ => None,
        }
    }
}

/// For routing return traffic, we need to care about the following types of TCP packets:
/// - SYN - when a SYN is observed, we should start tracking this connection
/// - RST - implies an immediate untrack
/// - FIN from source - implies a partial untrack
fn tcp_tracking_event(payload: &[u8], src_ip: IpAddr, dst_ip: IpAddr) -> Option<ConnectionUpdate> {
    let tcp = Tcp::<[u8]>::ref_from_bytes(payload).ok()?;
    let connection_identifier = SockAddrPair {
        src_ip,
        src_port: tcp.header.source_port.get(),
        dst_ip,
        dst_port: tcp.header.destination_port.get(),
    };

    if tcp.header.rst() {
        Some(ConnectionUpdate::UntrackTcp(connection_identifier))
    } else if tcp.header.syn() {
        Some(ConnectionUpdate::TrackTcp(connection_identifier))
    } else if tcp.header.fin() {
        Some(ConnectionUpdate::LocalFin(connection_identifier))
    } else {
        None
    }
}

/// The set of secondary-originated connections, owned exclusively by the mux's
/// send (inbound) half.
#[derive(Default)]
pub(crate) struct ConnectionTracker {
    /// Active TCP connections from the secondary, with their close progress.
    tcp: HashMap<SockAddrPair, FinState>,
    /// ICMP identifiers from the secondary, keyed by (dest IP, identifier).
    icmp: HashSet<(IpAddr, u16)>,
}

/// Which sides of a tracked TCP connection have sent a FIN.
#[derive(Default)]
struct FinState {
    local: bool,
    remote: bool,
}

impl ConnectionTracker {
    /// Apply a tracking event produced by [`outbound_conntrack_event`].
    pub(crate) fn apply(&mut self, event: ConnectionTrackerEvent) {
        match event.0 {
            ConnectionUpdate::TrackTcp(connection_identifier) => {
                self.tcp.insert(connection_identifier, FinState::default());
            }
            ConnectionUpdate::LocalFin(connection_identifier) => {
                if let Some(fins) = self.tcp.get_mut(&connection_identifier) {
                    fins.local = true;
                }
            }
            ConnectionUpdate::UntrackTcp(connection_identifier) => {
                self.tcp.remove(&connection_identifier);
            }
            ConnectionUpdate::TrackIcmp { dest, ident } => {
                self.icmp.insert((dest, ident));
            }
        }
    }

    /// Check if inbound `packet` is return traffic for a tracked secondary
    /// connection, advancing TCP close tracking as a side effect.
    pub(crate) fn is_secondary_return(&mut self, packet: &Ip) -> bool {
        let Some(ip) = ParsedIp::parse(packet) else {
            return false;
        };

        match ip.proto {
            IpNextProtocol::Tcp => self.tcp_return(ip.payload, ip.src, ip.dst),
            IpNextProtocol::Icmp => self.icmp_return(ip.payload, ip.src, ICMPV4_ECHO_REPLY),
            IpNextProtocol::Icmpv6 => self.icmp_return(ip.payload, ip.src, ICMPV6_ECHO_REPLY),
            _ => false,
        }
    }

    /// Check an inbound ICMP(v6) packet against the tracked idents, without
    /// parsing it at all when nothing is tracked.
    fn icmp_return(&self, payload: &[u8], src_ip: IpAddr, echo_reply_type: u8) -> bool {
        if self.icmp.is_empty() {
            return false;
        }
        let Some(ident) = icmp_echo_ident(payload, echo_reply_type) else {
            return false;
        };
        self.icmp.contains(&(src_ip, ident))
    }

    /// Match an inbound TCP packet against the tracked connections and advance
    /// close tracking.
    fn tcp_return(&mut self, payload: &[u8], src_ip: IpAddr, dst_ip: IpAddr) -> bool {
        if self.tcp.is_empty() {
            return false;
        }
        let Ok(tcp) = Tcp::<[u8]>::ref_from_bytes(payload) else {
            return false;
        };
        let connection_identifier = SockAddrPair {
            src_ip: dst_ip,
            src_port: tcp.header.destination_port.get(),
            dst_ip: src_ip,
            dst_port: tcp.header.source_port.get(),
        };

        let Some(fins) = self.tcp.get_mut(&connection_identifier) else {
            return false;
        };
        fins.remote |= tcp.header.fin();
        if tcp.header.rst() || (fins.local && fins.remote) {
            self.tcp.remove(&connection_identifier);
        }
        true
    }
}

/// Identifies a specific TCP connection between 2 socket addresses. Technically could also be used
/// for tracking UDP sessions too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SockAddrPair {
    src_ip: IpAddr,
    src_port: u16,
    dst_ip: IpAddr,
    dst_port: u16,
}

#[cfg(test)]
mod tests {
    use crate::gotatun::tcp_packet;

    use super::*;

    /// View raw bytes as an [`Ip`] packet for the tracker entry points.
    fn as_ip(bytes: &[u8]) -> &Ip {
        Ip::ref_from_bytes(bytes).expect("valid IP packet bytes")
    }

    /// Minimal IPv4 TCP packet with the given addresses, ports, and flags.
    fn apply_outbound(tracker: &mut ConnectionTracker, packet: &[u8]) {
        if let Some(event) = outbound_conntrack_event(as_ip(packet)) {
            tracker.apply(event);
        }
    }

    /// With nothing applied, no inbound packet is treated as secondary return.
    #[test]
    fn empty_tracker_reports_no_match() {
        let mut tracker = ConnectionTracker::default();
        let synack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x12);
        assert!(!tracker.is_secondary_return(as_ip(&synack)));
    }

    /// Tests if connection tracker removes a TCP connection only after both peers have issued a
    /// FIN.
    #[test]
    fn tcp_close_untracks_after_both_fins() {
        let mut tracker = ConnectionTracker::default();
        let synack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x12);

        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);
        assert!(tracker.is_secondary_return(as_ip(&synack)));

        let fin = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x01);
        apply_outbound(&mut tracker, &fin);
        assert!(tracker.is_secondary_return(as_ip(&synack)));

        // The peer's FIN+ACK completes the close: delivered, then untracked.
        let fin_ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x11);
        assert!(tracker.is_secondary_return(as_ip(&fin_ack)));
        assert!(!tracker.is_secondary_return(as_ip(&synack)));
    }

    /// When the peer closes first, the connection stays tracked until our FIN
    /// is answered: the peer's final ACK is still delivered, then untracked.
    #[test]
    fn tcp_close_by_peer_first_delivers_final_ack() {
        let mut tracker = ConnectionTracker::default();
        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);

        let peer_fin = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x11);
        assert!(tracker.is_secondary_return(as_ip(&peer_fin)));

        let our_fin = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x01);
        apply_outbound(&mut tracker, &our_fin);

        let final_ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x10);
        assert!(tracker.is_secondary_return(as_ip(&final_ack)));
        assert!(!tracker.is_secondary_return(as_ip(&final_ack)));
    }

    /// An inbound RST is delivered to the secondary and untracks immediately.
    #[test]
    fn inbound_rst_is_delivered_then_untracked() {
        let mut tracker = ConnectionTracker::default();
        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);

        let rst = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x04);
        assert!(tracker.is_secondary_return(as_ip(&rst)));

        let ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x10);
        assert!(!tracker.is_secondary_return(as_ip(&ack)));
    }

    /// A bare ACK (no SYN/FIN/RST) carries no tracking change.
    #[test]
    fn mid_stream_segment_produces_no_event() {
        let ack = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x10);
        assert!(outbound_conntrack_event(as_ip(&ack)).is_none());
    }
}
