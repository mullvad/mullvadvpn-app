//! Tracks connections originating from the secondary (smoltcp) interface so
//! that the [`IpMux`](super::ip_mux::IpMux) can route matching return traffic
//! back to it.
//!
//! There is no shared state: the mux's receive (outbound) half classifies each
//! secondary outbound packet into a [`ConnectionTrackerEvent`] via
//! [`outbound_event`] and sends it over a channel; the send (inbound) half owns
//! the [`ConnectionTracker`] outright and applies those events before each
//! lookup. The connection is always registered before its SYN/echo-request is
//! transmitted, and any return traffic can only arrive a network round trip
//! later, so the event is always in the channel before the packet that needs
//! it — no lock, and the reader's lookups are on data it alone owns.

use smoltcp::wire::{
    Icmpv4Message, Icmpv4Packet, Icmpv6Message, Icmpv6Packet, IpProtocol, Ipv4Packet, Ipv6Packet,
    TcpPacket,
};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

/// An opaque connection-tracking event produced from a secondary *outbound*
/// packet by [`outbound_event`] and applied to the reader-owned
/// [`ConnectionTracker`] via [`ConnectionTracker::apply`].
pub(crate) struct ConnectionTrackerEvent(ConnectionUpdate);

enum ConnectionUpdate {
    TrackTcp(FourTuple),
    /// Outbound FIN: our half of the close. The entry is kept until the peer's
    /// half has been seen too, so the closing handshake still reaches smoltcp.
    LocalFin(FourTuple),
    /// Outbound RST: the connection is dead immediately.
    UntrackTcp(FourTuple),
    TrackIcmp {
        dest: IpAddr,
        ident: u16,
    },
}

/// Inspect a secondary *outbound* packet and produce the tracking event it
/// implies, if any. Runs on the writer (outbound) side, off the read path.
pub(crate) fn outbound_event(packet: &[u8]) -> Option<ConnectionTrackerEvent> {
    let ip = ParsedIp::parse(packet)?;

    let event = match ip.proto {
        IpProtocol::Tcp => tcp_event(ip.payload, ip.src, ip.dst),
        IpProtocol::Icmp => {
            icmpv4_echo_request_ident(ip.payload).map(|ident| ConnectionUpdate::TrackIcmp {
                dest: ip.dst,
                ident,
            })
        }
        IpProtocol::Icmpv6 => {
            icmpv6_echo_request_ident(ip.payload).map(|ident| ConnectionUpdate::TrackIcmp {
                dest: ip.dst,
                ident,
            })
        }
        _ => None,
    };

    event.map(ConnectionTrackerEvent)
}

fn icmpv4_echo_request_ident(payload: &[u8]) -> Option<u16> {
    let icmp = Icmpv4Packet::new_checked(payload).ok()?;
    (icmp.msg_type() == Icmpv4Message::EchoRequest).then(|| icmp.echo_ident())
}

fn icmpv6_echo_request_ident(payload: &[u8]) -> Option<u16> {
    let icmp = Icmpv6Packet::new_checked(payload).ok()?;
    (icmp.msg_type() == Icmpv6Message::EchoRequest).then(|| icmp.echo_ident())
}

fn icmpv4_echo_reply_ident(payload: &[u8]) -> Option<u16> {
    let icmp = Icmpv4Packet::new_checked(payload).ok()?;
    (icmp.msg_type() == Icmpv4Message::EchoReply).then(|| icmp.echo_ident())
}

fn icmpv6_echo_reply_ident(payload: &[u8]) -> Option<u16> {
    let icmp = Icmpv6Packet::new_checked(payload).ok()?;
    (icmp.msg_type() == Icmpv6Message::EchoReply).then(|| icmp.echo_ident())
}

/// The header fields shared by v4/v6 packets, extracted once so the
/// protocol-level matching below is written a single time.
struct ParsedIp<'a> {
    src: IpAddr,
    dst: IpAddr,
    proto: IpProtocol,
    payload: &'a [u8],
}

impl<'a> ParsedIp<'a> {
    fn parse(packet: &'a [u8]) -> Option<Self> {
        if let Ok(ip) = Ipv4Packet::new_checked(packet) {
            Some(ParsedIp {
                src: IpAddr::from(ip.src_addr()),
                dst: IpAddr::from(ip.dst_addr()),
                proto: ip.next_header(),
                payload: ip.payload(),
            })
        } else if let Ok(ip) = Ipv6Packet::new_checked(packet) {
            Some(ParsedIp {
                src: IpAddr::from(ip.src_addr()),
                dst: IpAddr::from(ip.dst_addr()),
                proto: ip.next_header(),
                payload: ip.payload(),
            })
        } else {
            None
        }
    }
}

/// A secondary outbound TCP packet tracks its 4-tuple on SYN, untracks it on
/// RST, and marks our half of the close on FIN; anything else (a mid-stream
/// segment) implies no change.
fn tcp_event(payload: &[u8], src_ip: IpAddr, dst_ip: IpAddr) -> Option<ConnectionUpdate> {
    let tcp = TcpPacket::new_checked(payload).ok()?;
    let tuple = FourTuple {
        src_ip,
        src_port: tcp.src_port(),
        dst_ip,
        dst_port: tcp.dst_port(),
    };

    if tcp.rst() {
        Some(ConnectionUpdate::UntrackTcp(tuple))
    } else if tcp.syn() {
        Some(ConnectionUpdate::TrackTcp(tuple))
    } else if tcp.fin() {
        Some(ConnectionUpdate::LocalFin(tuple))
    } else {
        None
    }
}

/// The set of secondary-originated connections, owned exclusively by the mux's
/// send (inbound) half.
#[derive(Default)]
pub(crate) struct ConnectionTracker {
    /// Active TCP connections from the secondary, with their close progress.
    tcp: HashMap<FourTuple, FinState>,
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
    /// Apply a tracking event produced by [`outbound_event`].
    pub(crate) fn apply(&mut self, event: ConnectionTrackerEvent) {
        match event.0 {
            ConnectionUpdate::TrackTcp(tuple) => {
                self.tcp.insert(tuple, FinState::default());
            }
            ConnectionUpdate::LocalFin(tuple) => {
                if let Some(fins) = self.tcp.get_mut(&tuple) {
                    fins.local = true;
                }
            }
            ConnectionUpdate::UntrackTcp(tuple) => {
                self.tcp.remove(&tuple);
            }
            ConnectionUpdate::TrackIcmp { dest, ident } => {
                self.icmp.insert((dest, ident));
            }
        }
    }

    /// Check if inbound `packet` is return traffic for a tracked secondary
    /// connection, advancing TCP close tracking as a side effect. Skips the L4
    /// parse entirely when nothing of the packet's protocol is tracked (the
    /// common steady-state download case).
    pub(crate) fn is_secondary_return(&mut self, packet: &[u8]) -> bool {
        let Some(ip) = ParsedIp::parse(packet) else {
            return false;
        };

        match ip.proto {
            IpProtocol::Tcp => self.tcp_return(ip.payload, ip.src, ip.dst),
            IpProtocol::Icmp => self.icmp_return(ip.payload, ip.src, icmpv4_echo_reply_ident),
            IpProtocol::Icmpv6 => self.icmp_return(ip.payload, ip.src, icmpv6_echo_reply_ident),
            _ => false,
        }
    }

    /// Check an inbound ICMP(v6) packet against the tracked idents, without
    /// parsing it at all when nothing is tracked (the common case).
    fn icmp_return(
        &self,
        payload: &[u8],
        src_ip: IpAddr,
        echo_reply_ident: impl FnOnce(&[u8]) -> Option<u16>,
    ) -> bool {
        if self.icmp.is_empty() {
            return false;
        }
        let Some(ident) = echo_reply_ident(payload) else {
            return false;
        };
        self.icmp.contains(&(src_ip, ident))
    }

    /// Match an inbound TCP packet against the tracked connections and advance
    /// close tracking. The entry is dropped on an inbound RST, or on the first
    /// inbound packet once both sides have FIN'd — that packet (the peer's FIN,
    /// or its final ACK when the peer closed first) is still routed to the
    /// secondary so smoltcp can finish the close handshake.
    fn tcp_return(&mut self, payload: &[u8], src_ip: IpAddr, dst_ip: IpAddr) -> bool {
        if self.tcp.is_empty() {
            return false;
        }
        let Ok(tcp) = TcpPacket::new_checked(payload) else {
            return false;
        };
        let tuple = FourTuple {
            src_ip: dst_ip,
            src_port: tcp.dst_port(),
            dst_ip: src_ip,
            dst_port: tcp.src_port(),
        };

        let Some(fins) = self.tcp.get_mut(&tuple) else {
            return false;
        };
        fins.remote |= tcp.fin();
        if tcp.rst() || (fins.local && fins.remote) {
            self.tcp.remove(&tuple);
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FourTuple {
    src_ip: IpAddr,
    src_port: u16,
    dst_ip: IpAddr,
    dst_port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal IPv4 TCP packet with the given addresses, ports, and flags.
    fn tcp_packet(src: [u8; 4], dst: [u8; 4], sp: u16, dp: u16, flags: u8) -> Vec<u8> {
        let mut pkt = vec![0u8; 40]; // 20 IPv4 + 20 TCP
        pkt[0] = 0x45;
        pkt[2..4].copy_from_slice(&40u16.to_be_bytes());
        pkt[8] = 64;
        pkt[9] = 6; // TCP
        pkt[12..16].copy_from_slice(&src);
        pkt[16..20].copy_from_slice(&dst);
        pkt[20..22].copy_from_slice(&sp.to_be_bytes());
        pkt[22..24].copy_from_slice(&dp.to_be_bytes());
        pkt[32] = 0x50; // data offset = 5 words
        pkt[33] = flags;
        pkt
    }

    fn apply_outbound(tracker: &mut ConnectionTracker, packet: &[u8]) {
        if let Some(event) = outbound_event(packet) {
            tracker.apply(event);
        }
    }

    /// With nothing applied, no inbound packet is treated as secondary return.
    #[test]
    fn empty_tracker_reports_no_match() {
        let mut tracker = ConnectionTracker::default();
        let synack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x12);
        assert!(!tracker.is_secondary_return(&synack));
    }

    /// An outbound SYN tracks the connection. An outbound FIN alone must not
    /// untrack it — the peer's half of the close still has to reach smoltcp —
    /// but once the peer's FIN is routed too, the connection is untracked.
    #[test]
    fn tcp_close_untracks_after_both_fins() {
        let mut tracker = ConnectionTracker::default();
        let synack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x12);

        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);
        assert!(tracker.is_secondary_return(&synack));

        let fin = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x01);
        apply_outbound(&mut tracker, &fin);
        assert!(tracker.is_secondary_return(&synack));

        // The peer's FIN+ACK completes the close: delivered, then untracked.
        let fin_ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x11);
        assert!(tracker.is_secondary_return(&fin_ack));
        assert!(!tracker.is_secondary_return(&synack));
    }

    /// When the peer closes first, the connection stays tracked until our FIN
    /// is answered: the peer's final ACK is still delivered, then untracked.
    #[test]
    fn tcp_close_by_peer_first_delivers_final_ack() {
        let mut tracker = ConnectionTracker::default();
        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);

        let peer_fin = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x11);
        assert!(tracker.is_secondary_return(&peer_fin));

        let our_fin = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x01);
        apply_outbound(&mut tracker, &our_fin);

        let final_ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x10);
        assert!(tracker.is_secondary_return(&final_ack));
        assert!(!tracker.is_secondary_return(&final_ack));
    }

    /// An inbound RST is delivered to the secondary and untracks immediately.
    #[test]
    fn inbound_rst_is_delivered_then_untracked() {
        let mut tracker = ConnectionTracker::default();
        let syn = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x02);
        apply_outbound(&mut tracker, &syn);

        let rst = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x04);
        assert!(tracker.is_secondary_return(&rst));

        let ack = tcp_packet([1, 1, 1, 1], [10, 0, 0, 1], 1337, 49152, 0x10);
        assert!(!tracker.is_secondary_return(&ack));
    }

    /// A bare ACK (no SYN/FIN/RST) carries no tracking change.
    #[test]
    fn mid_stream_segment_produces_no_event() {
        let ack = tcp_packet([10, 0, 0, 1], [1, 1, 1, 1], 49152, 1337, 0x10);
        assert!(outbound_event(&ack).is_none());
    }
}
