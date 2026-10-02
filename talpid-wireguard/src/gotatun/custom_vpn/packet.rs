//! Parsing and address rewriting of IP packets, shared by the custom VPN adapters.

use gotatun::packet::{Ip, IpNextProtocol, Ipv4, Ipv4Header, Ipv6, Packet, Udp};
use std::net::IpAddr;
use zerocopy::{FromBytes, IntoBytes, big_endian};

/// Offset of the checksum in a TCP header.
const TCP_CHECKSUM_OFFSET: usize = 16;

/// The IHL value when no optional fields are present.
const IPV4_NO_OPTIONAL_FIELDS_IHL: u8 = 5;

/// A UDP datagram, or the first fragment of one.
pub struct UdpPacket<'a> {
    pub source: IpAddr,
    pub udp: &'a Udp,
}

impl<'a> UdpPacket<'a> {
    /// Parse `ip` as a UDP packet. Returns `None` for anything else, including IPv4 fragments
    /// other than the first, and IPv6 packets with extension headers.
    pub fn parse(ip: &'a Ip) -> Option<Self> {
        let (source, payload) = match ip.header.version() {
            4 => {
                let ipv4 = Ipv4::<[u8]>::ref_from_bytes(ip.as_bytes()).ok()?;
                let flags = ipv4.header.flags_and_fragment_offset;
                if ipv4.header.next_protocol() != IpNextProtocol::Udp
                    || flags.fragment_offset() != 0
                {
                    return None;
                }
                // `Ipv4::payload` starts after the fixed 20-byte header, so any IPv4 options must
                // be skipped to reach the transport payload.
                let options_len = usize::from(ipv4.header.ihl())
                    .checked_mul(4)?
                    .checked_sub(Ipv4Header::LEN)?;
                (
                    IpAddr::from(ipv4.header.source()),
                    ipv4.payload.get(options_len..)?,
                )
            }
            6 => {
                let ipv6 = Ipv6::<[u8]>::ref_from_bytes(ip.as_bytes()).ok()?;
                if ipv6.header.next_protocol() != IpNextProtocol::Udp {
                    return None;
                }
                (IpAddr::from(ipv6.header.source()), &ipv6.payload[..])
            }
            _ => return None,
        };
        Some(Self {
            source,
            udp: Udp::<[u8]>::ref_from_bytes(payload).ok()?,
        })
    }

    pub fn source_port(&self) -> u16 {
        self.udp.header.source_port.get()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Source,
    Destination,
}

/// Replace the source or destination address of `packet` with `to`, if it is `from`, and update
/// the IP and transport (UDP or TCP) checksums. Returns whether the address was replaced.
///
/// IPv4 packets with options are left alone.
pub fn rewrite_address(
    packet: &mut Packet<Ip>,
    direction: Direction,
    from: IpAddr,
    to: IpAddr,
) -> bool {
    match (packet.header.version(), from, to) {
        (4, IpAddr::V4(from), IpAddr::V4(to)) => {
            let Ok(ipv4) = Ipv4::<[u8]>::mut_from_bytes(packet.as_mut_bytes()) else {
                return false;
            };
            if ipv4.header.ihl() != IPV4_NO_OPTIONAL_FIELDS_IHL {
                return false;
            }
            let address = match direction {
                Direction::Source => &mut ipv4.header.source_address,
                Direction::Destination => &mut ipv4.header.destination_address,
            };
            if *address != big_endian::U32::from_bytes(from.octets()) {
                return false;
            }
            *address = big_endian::U32::from_bytes(to.octets());
            ipv4.header.header_checksum = ipv4.header.compute_checksum().into();

            let protocol = ipv4.header.next_protocol();
            update_transport_checksum(&mut ipv4.payload, protocol, &from.octets(), &to.octets());
            true
        }
        (6, IpAddr::V6(from), IpAddr::V6(to)) => {
            let Ok(ipv6) = Ipv6::<[u8]>::mut_from_bytes(packet.as_mut_bytes()) else {
                return false;
            };
            let address = match direction {
                Direction::Source => &mut ipv6.header.source_address,
                Direction::Destination => &mut ipv6.header.destination_address,
            };
            if *address != big_endian::U128::from_bytes(from.octets()) {
                return false;
            }
            *address = big_endian::U128::from_bytes(to.octets());

            let protocol = ipv6.header.next_protocol();
            update_transport_checksum(&mut ipv6.payload, protocol, &from.octets(), &to.octets());
            true
        }
        _ => false,
    }
}

/// Update the checksum of the TCP or UDP header at the start of `transport`, after an address in
/// its pseudo-header changed from `old` to `new`.
fn update_transport_checksum(
    transport: &mut [u8],
    protocol: IpNextProtocol,
    old: &[u8],
    new: &[u8],
) {
    let offset = match protocol {
        IpNextProtocol::Udp => {
            let Ok(udp) = Udp::<[u8]>::ref_from_bytes(transport) else {
                return;
            };
            // For UDP, a checksum of 0 means "no checksum"
            if udp.header.checksum.get() == 0 {
                return;
            }
            std::mem::offset_of!(gotatun::packet::UdpHeader, checksum)
        }
        IpNextProtocol::Tcp => TCP_CHECKSUM_OFFSET,
        _ => return,
    };
    let Some(checksum) = transport.get_mut(offset..offset + 2) else {
        return;
    };
    let old_checksum = u16::from_be_bytes([checksum[0], checksum[1]]);
    let new_checksum = incremental_checksum_update(old_checksum, old, new);
    checksum.copy_from_slice(&new_checksum.to_be_bytes());
}

/// Update the Internet checksum `checksum` after the data it covers changed from `old` to `new`,
/// without scanning the entire packet. See [RFC 1624](https://www.rfc-editor.org/rfc/rfc1624).
///
/// `old` and `new` must have the same even length.
fn incremental_checksum_update(checksum: u16, old: &[u8], new: &[u8]) -> u16 {
    // HC' = ~(~HC + ~m + m'), where HC is the old checksum, m is old data, m' is new data
    fn words(bytes: &[u8]) -> impl Iterator<Item = u32> + '_ {
        bytes
            .chunks_exact(2)
            .map(|word| u32::from(u16::from_be_bytes([word[0], word[1]])))
    }
    let mut sum = u32::from(!checksum);
    sum += words(old).map(|word| word ^ 0xffff).sum::<u32>();
    sum += words(new).sum::<u32>();
    while sum > 0xffff {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use gotatun::packet::{
        Ipv6Header, PseudoHeaderV4, PseudoHeaderV6, UdpHeader, checksum_udp, checksum_udp_with_skip,
    };
    use std::net::SocketAddr;
    use zerocopy::FromZeros;

    /// Build a UDP packet with valid checksums.
    pub fn udp_packet(src: SocketAddr, dst: SocketAddr, payload: &[u8]) -> Packet<Ip> {
        let udp_len = UdpHeader::LEN + payload.len();
        let mut udp = vec![0u8; udp_len];
        {
            let udp = Udp::<[u8]>::mut_from_bytes(&mut udp).unwrap();
            udp.header.source_port = src.port().into();
            udp.header.destination_port = dst.port().into();
            udp.header.length = u16::try_from(udp_len).unwrap().into();
            udp.payload.copy_from_slice(payload);
        }

        let mut bytes = match (src.ip(), dst.ip()) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => {
                let mut header = Ipv4Header::new(src, dst, IpNextProtocol::Udp, &udp);
                header.header_checksum = header.compute_checksum().into();
                let pseudo = PseudoHeaderV4::from_udp(
                    header.source_address,
                    header.destination_address,
                    Udp::<[u8]>::ref_from_bytes(&udp).unwrap(),
                );
                let checksum = checksum_udp(pseudo, &udp);
                Udp::<[u8]>::mut_from_bytes(&mut udp)
                    .unwrap()
                    .header
                    .checksum = checksum.into();
                header.as_bytes().to_vec()
            }
            (IpAddr::V6(src), IpAddr::V6(dst)) => {
                let mut header = Ipv6Header::new_zeroed();
                header.set_version(6);
                header.payload_length = u16::try_from(udp_len).unwrap().into();
                header.next_header = IpNextProtocol::Udp;
                header.hop_limit = 64;
                header.source_address = src.to_bits().into();
                header.destination_address = dst.to_bits().into();
                let pseudo = PseudoHeaderV6::from_udp(
                    header.source_address,
                    header.destination_address,
                    Udp::<[u8]>::ref_from_bytes(&udp).unwrap(),
                );
                let checksum = checksum_udp(pseudo, &udp);
                Udp::<[u8]>::mut_from_bytes(&mut udp)
                    .unwrap()
                    .header
                    .checksum = checksum.into();
                header.as_bytes().to_vec()
            }
            _ => unreachable!("mixed address families"),
        };
        bytes.extend_from_slice(&udp);
        Packet::copy_from(&bytes[..]).try_into_ip().unwrap()
    }

    /// Whether the IP and UDP checksums of `packet` are valid.
    pub fn checksums_are_valid(packet: &Packet<Ip>) -> bool {
        match packet.header.version() {
            4 => {
                let ipv4 = Ipv4::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap();
                let udp = Udp::<[u8]>::ref_from_bytes(&ipv4.payload).unwrap();
                let pseudo = PseudoHeaderV4::from_udp(
                    ipv4.header.source_address,
                    ipv4.header.destination_address,
                    udp,
                );
                ipv4.header.compute_checksum() == ipv4.header.header_checksum.get()
                    && checksum_udp_with_skip(pseudo, udp) == udp.header.checksum.get()
            }
            6 => {
                let ipv6 = Ipv6::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap();
                let udp = Udp::<[u8]>::ref_from_bytes(&ipv6.payload).unwrap();
                let pseudo = PseudoHeaderV6::from_udp(
                    ipv6.header.source_address,
                    ipv6.header.destination_address,
                    udp,
                );
                checksum_udp_with_skip(pseudo, udp) == udp.header.checksum.get()
            }
            _ => unreachable!(),
        }
    }

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn rewrite_updates_checksums() {
        for (src, dst, new_src, new_dst) in [
            ("10.64.0.2:1234", "10.0.0.1:53", "172.16.0.1", "10.100.0.5"),
            ("[fc00::2]:1234", "[fd00::1]:53", "fd00::2", "fd00::53"),
        ] {
            let (src, dst) = (addr(src), addr(dst));
            let (new_src, new_dst) = (new_src.parse().unwrap(), new_dst.parse().unwrap());
            let mut packet = udp_packet(src, dst, b"payload");

            assert!(rewrite_address(
                &mut packet,
                Direction::Source,
                src.ip(),
                new_src
            ));
            assert!(rewrite_address(
                &mut packet,
                Direction::Destination,
                dst.ip(),
                new_dst
            ));
            assert_eq!(packet.source(), Some(new_src));
            assert_eq!(packet.destination(), Some(new_dst));
            assert!(checksums_are_valid(&packet));
        }
    }

    #[test]
    fn no_rewrite_of_other_addresses() {
        let (src, dst) = (addr("10.64.0.2:1234"), addr("10.0.0.1:53"));
        let mut packet = udp_packet(src, dst, b"payload");
        let other = "10.1.2.3".parse().unwrap();

        assert!(!rewrite_address(
            &mut packet,
            Direction::Source,
            other,
            other
        ));
        assert!(!rewrite_address(
            &mut packet,
            Direction::Destination,
            src.ip(),
            other
        ));
        assert_eq!(packet.source(), Some(src.ip()));
        assert_eq!(packet.destination(), Some(dst.ip()));
    }

    #[test]
    fn zero_udp_checksum_is_not_updated() {
        let (src, dst) = (addr("10.64.0.2:1234"), addr("10.0.0.1:53"));
        let mut packet = udp_packet(src, dst, b"payload");
        {
            let ipv4 = Ipv4::<[u8]>::mut_from_bytes(packet.as_mut_bytes()).unwrap();
            let udp = Udp::<[u8]>::mut_from_bytes(&mut ipv4.payload).unwrap();
            udp.header.checksum = 0.into();
        }

        let new_src = "172.16.0.1".parse().unwrap();
        assert!(rewrite_address(
            &mut packet,
            Direction::Source,
            src.ip(),
            new_src
        ));
        let ipv4 = Ipv4::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap();
        let udp = Udp::<[u8]>::ref_from_bytes(&ipv4.payload).unwrap();
        assert_eq!(udp.header.checksum.get(), 0);
        assert_eq!(
            ipv4.header.compute_checksum(),
            ipv4.header.header_checksum.get()
        );
    }

    #[test]
    fn parse_udp() {
        let (src, dst) = (addr("10.64.0.2:1234"), addr("10.0.0.1:53"));
        let packet = udp_packet(src, dst, b"payload");
        let udp = UdpPacket::parse(&packet).unwrap();
        assert_eq!(SocketAddr::new(udp.source, udp.source_port()), src);
        assert_eq!(&udp.udp.payload, b"payload");
    }
}
