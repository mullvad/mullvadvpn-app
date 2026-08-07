//! Decides whether the block list forbids a packet.

use std::net::IpAddr;

use smoltcp::wire::{IpProtocol, Ipv4Packet, UdpPacket};

use crate::{
    firewall::{BlockRule, Endpoints, macos::RuleSet},
    web::routes::TransportProtocol,
};

/// Whether any rule in the block list matches this packet.
///
/// Packets we cannot parse pass, as they did before the block list saw them.
pub fn blocked(rules: &RuleSet, packet: &[u8]) -> bool {
    if rules.is_empty() {
        return false;
    }
    let Ok(ip) = Ipv4Packet::new_checked(packet) else {
        return false;
    };
    let src = IpAddr::from(ip.src_addr());
    let dst = IpAddr::from(ip.dst_addr());
    let protocol = transport_protocol(ip.next_header());

    rules.candidates(src).any(|rule| match rule {
        BlockRule::Host {
            endpoints,
            protocols,
        } => {
            matches_endpoints(endpoints, src, dst)
                && (protocols.is_empty()
                    || protocol.is_some_and(|protocol| protocols.contains(&protocol)))
        }
        BlockRule::WireGuard { endpoints } => {
            matches_endpoints(endpoints, src, dst) && is_wireguard(&ip)
        }
    })
}

fn matches_endpoints(endpoints: &Endpoints, src: IpAddr, dst: IpAddr) -> bool {
    endpoints.src.contains(src) && (endpoints.dst.contains(dst) != endpoints.invert_dst)
}

fn transport_protocol(protocol: IpProtocol) -> Option<TransportProtocol> {
    match protocol {
        IpProtocol::Tcp => Some(TransportProtocol::Tcp),
        IpProtocol::Udp => Some(TransportProtocol::Udp),
        IpProtocol::Icmp => Some(TransportProtocol::Icmp),
        IpProtocol::Icmpv6 => Some(TransportProtocol::IcmpV6),
        _ => None,
    }
}

/// A WireGuard datagram starts with a message type of 1 to 4 followed by three reserved zero
/// bytes, right after the UDP header. See <https://wiki.wireshark.org/WireGuard>.
fn is_wireguard(ip: &Ipv4Packet<&[u8]>) -> bool {
    if ip.next_header() != IpProtocol::Udp || ip.frag_offset() > 0 {
        return false;
    }
    let Ok(udp) = UdpPacket::new_checked(ip.payload()) else {
        return false;
    };
    matches!(udp.payload(), [1..=4, 0, 0, 0, ..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::{
        phy::ChecksumCapabilities,
        wire::{IPV4_HEADER_LEN, Ipv4Repr, UdpRepr},
    };
    use std::{collections::BTreeSet, net::Ipv4Addr};

    const CLIENT: Ipv4Addr = Ipv4Addr::new(192, 168, 91, 84);
    const SERVER: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);

    fn udp_packet(payload: &[u8]) -> Vec<u8> {
        let repr = UdpRepr {
            src_port: 51820,
            dst_port: 51820,
        };
        let mut packet = vec![0u8; IPV4_HEADER_LEN + repr.header_len() + payload.len()];
        let mut ip = Ipv4Packet::new_unchecked(&mut packet[..]);
        Ipv4Repr {
            src_addr: CLIENT,
            dst_addr: SERVER,
            next_header: IpProtocol::Udp,
            payload_len: repr.header_len() + payload.len(),
            hop_limit: 64,
        }
        .emit(&mut ip, &ChecksumCapabilities::default());
        repr.emit(
            &mut UdpPacket::new_unchecked(ip.payload_mut()),
            &CLIENT.into(),
            &SERVER.into(),
            payload.len(),
            |buffer| buffer.copy_from_slice(payload),
            &ChecksumCapabilities::default(),
        );
        packet
    }

    fn one_rule(rule: BlockRule) -> RuleSet {
        let mut rules = RuleSet::default();
        rules.insert(rule, uuid::Uuid::nil());
        rules
    }

    fn endpoints(dst: &str, invert_dst: bool) -> Endpoints {
        Endpoints {
            src: "0.0.0.0/0".parse().unwrap(),
            dst: dst.parse().unwrap(),
            invert_dst,
        }
    }

    #[test]
    fn a_host_rule_blocks_only_its_destination() {
        let rules = one_rule(BlockRule::Host {
            endpoints: endpoints("1.1.1.1/32", false),
            protocols: BTreeSet::new(),
        });
        assert!(blocked(&rules, &udp_packet(b"data")));

        let rules = one_rule(BlockRule::Host {
            endpoints: endpoints("8.8.8.8/32", false),
            protocols: BTreeSet::new(),
        });
        assert!(!blocked(&rules, &udp_packet(b"data")));
    }

    #[test]
    fn an_inverted_rule_blocks_everything_but_its_destination() {
        let rules = one_rule(BlockRule::Host {
            endpoints: endpoints("8.8.8.8/32", true),
            protocols: BTreeSet::new(),
        });
        assert!(blocked(&rules, &udp_packet(b"data")));

        let rules = one_rule(BlockRule::Host {
            endpoints: endpoints("1.1.1.1/32", true),
            protocols: BTreeSet::new(),
        });
        assert!(!blocked(&rules, &udp_packet(b"data")));
    }

    /// The source-host bucketing must find rules for a packet's source and only for it, or the
    /// rule set would silently filter for the wrong device.
    #[test]
    fn a_host_source_rule_applies_only_to_packets_from_that_host() {
        let host_endpoints = |src: &str| Endpoints {
            src: src.parse().unwrap(),
            dst: "0.0.0.0/0".parse().unwrap(),
            invert_dst: false,
        };

        let rules = one_rule(BlockRule::Host {
            endpoints: host_endpoints("192.168.91.84/32"),
            protocols: BTreeSet::new(),
        });
        assert!(blocked(&rules, &udp_packet(b"data")));

        let rules = one_rule(BlockRule::Host {
            endpoints: host_endpoints("192.168.91.85/32"),
            protocols: BTreeSet::new(),
        });
        assert!(!blocked(&rules, &udp_packet(b"data")));
    }

    #[test]
    fn a_protocol_rule_ignores_other_protocols() {
        let rules = one_rule(BlockRule::Host {
            endpoints: endpoints("1.1.1.1/32", false),
            protocols: BTreeSet::from([TransportProtocol::Tcp]),
        });
        assert!(!blocked(&rules, &udp_packet(b"data")));
    }

    #[test]
    fn a_wireguard_rule_needs_the_wireguard_header() {
        let rules = one_rule(BlockRule::WireGuard {
            endpoints: endpoints("1.1.1.1/32", false),
        });
        let handshake_initiation = [1, 0, 0, 0, 0xaa, 0xbb];
        assert!(blocked(&rules, &udp_packet(&handshake_initiation)));
        assert!(!blocked(&rules, &udp_packet(&[9, 0, 0, 0])));
        assert!(!blocked(&rules, &udp_packet(&[1, 1, 0, 0])));
    }
}
