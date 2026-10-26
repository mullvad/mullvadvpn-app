use gotatun::packet::{IpNextProtocol, Ipv6};
use zerocopy::{FromBytes, Immutable, KnownLayout, big_endian};

pub struct NextProtocolOffset {
    /// The next protocol e.g. TCP or UDP or ICMP
    pub protocol: IpNextProtocol,
    /// The offset into the IP packet where the next protocol payload starts
    pub offset: usize,
    /// True if a Layer 4 header is present and accessible.
    /// False if the payload is encrypted (ESP), or the packet is a subsequent fragment.
    pub has_l4_header: bool,
}

/// Extracts the final next protocol header and its offset for the IPv6 packet `ipv6`.
/// This will be the L4 transport protocol if one exists and is not encrypted.
pub fn parse_next_protocol(ipv6: &Ipv6) -> Option<NextProtocolOffset> {
    let mut current_protocol = ipv6.header.next_protocol();
    let mut payload_offset = 0;
    let mut header_count = 0;

    // Loop while the protocol is a known extension header
    while is_ipv6_extension_header(current_protocol) {
        // Defend against maliciously deep header chains
        header_count += 1;
        if header_count > 8 {
            return None;
        }

        if current_protocol == IpNextProtocol::Esp {
            return Some(NextProtocolOffset {
                protocol: current_protocol,
                offset: payload_offset,
                has_l4_header: false,
            });
        }

        if current_protocol == IpNextProtocol::Ipv6Frag {
            let frag_header = Ipv6FragmentHeader::parse(&ipv6.payload[payload_offset..])?;

            if frag_header.fragment_offset() > 0 {
                // Subsequent fragment: No L4 header exists
                return Some(NextProtocolOffset {
                    protocol: current_protocol,
                    offset: payload_offset,
                    has_l4_header: false,
                });
            }
        }

        let ext_len = get_ext_header_length(current_protocol, &ipv6.payload[payload_offset..])?;
        let next_proto = get_next_protocol_from_ext_header(&ipv6.payload[payload_offset..])?;

        payload_offset += ext_len;
        current_protocol = next_proto;

        // ensure we haven't jumped outside the payload buffer
        if payload_offset > ipv6.payload.len() {
            return None; // Malformed packet
        }
    }

    // Successfully found the end of the extension header chain
    Some(NextProtocolOffset {
        protocol: current_protocol,
        offset: payload_offset,
        has_l4_header: current_protocol != IpNextProtocol::Ipv6NoNxt,
    })
}

// Returns the length of the extension header in bytes.
fn get_ext_header_length(protocol: IpNextProtocol, buffer: &[u8]) -> Option<usize> {
    if buffer.len() < 2 {
        return None;
    }

    match protocol {
        // Fixed Length: Fragment Header is always exactly 8 bytes.
        IpNextProtocol::Ipv6Frag => Some(8),

        // 32-bit word chunks: Authentication Header (AH) (RFC 4302)
        // Length = (Payload Len + 2) * 4 bytes
        IpNextProtocol::Ah => {
            let payload_len = buffer[1] as usize;
            Some((payload_len + 2) * 4)
        }

        // 64-bit word chunks: Hop-by-Hop (0), Routing (43), Dest Opts (60),
        // Mobility (135), Host Identity (139), Shim6 (140) (RFC 8200)
        // Length = (Hdr Ext Len + 1) * 8 bytes
        IpNextProtocol::Hopopt
        | IpNextProtocol::Ipv6Route
        | IpNextProtocol::Ipv6Opts
        | IpNextProtocol::MobilityHeader
        | IpNextProtocol::Hip
        | IpNextProtocol::Shim6 => {
            let hdr_ext_len = buffer[1] as usize;
            Some((hdr_ext_len + 1) * 8)
        }

        // Unknown or unsupported extension header
        _ => None,
    }
}

pub fn get_next_protocol_from_ext_header(buffer: &[u8]) -> Option<IpNextProtocol> {
    IpNextProtocol::ref_from_prefix(buffer)
        .ok()
        .map(|(next_protocol, _)| *next_protocol)
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable)]
pub struct Ipv6FragmentHeader {
    pub next_header: u8,
    pub reserved: u8,
    /// Contains the 13-bit offset, 2 reserved bits, and 1-bit 'More Fragments' flag
    pub frag_data: big_endian::U16,
    pub identification: big_endian::U32,
}

impl Ipv6FragmentHeader {
    /// Attempts to read an IPv6 Fragment Header from the beginning of the buffer
    /// without copying any memory.
    pub fn parse(buffer: &[u8]) -> Option<&Self> {
        Self::ref_from_prefix(buffer)
            .ok()
            .map(|(header, _remaining_bytes)| header)
    }

    /// Extracts the 13-bit fragment offset
    #[inline]
    pub fn fragment_offset(&self) -> u16 {
        self.frag_data.get() >> 3
    }
}

fn is_ipv6_extension_header(next_protocol: IpNextProtocol) -> bool {
    matches!(
        next_protocol,
        IpNextProtocol::Hopopt
            | IpNextProtocol::Ipv6Route
            | IpNextProtocol::Ipv6Frag
            | IpNextProtocol::Esp
            | IpNextProtocol::Ah
            | IpNextProtocol::Ipv6Opts
            | IpNextProtocol::MobilityHeader
            | IpNextProtocol::Hip
            | IpNextProtocol::Shim6
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_l4_udp() {
        let udp_payload = [0x12, 0x34, 0x56, 0x78]; // Dummy UDP data
        let packet_bytes = build_ipv6_packet(17, &udp_payload); // 17 = UDP
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Udp);
        assert_eq!(result.offset, 0);
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_direct_l4_tcp() {
        let tcp_payload = [0xde, 0xad, 0xbe, 0xef];
        let packet_bytes = build_ipv6_packet(6, &tcp_payload); // 6 = TCP
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Tcp);
        assert_eq!(result.offset, 0);
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_single_hop_by_hop_to_udp() {
        // Hop-by-Hop Header (protocol 0):
        // [Next: UDP (17), HdrExtLen: 0 (8 bytes total), 6 bytes of padding/options]
        let hop_by_hop = [17, 0, 0, 0, 0, 0, 0, 0];
        let udp_data = [1, 2, 3, 4];
        let payload = [hop_by_hop.as_slice(), udp_data.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(0, &payload); // 0 = Hopopt
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Udp);
        assert_eq!(result.offset, 8); // Offset past the 8-byte Hop-by-Hop header
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_chained_extension_headers() {
        // Chain: Hop-by-Hop (8 bytes) -> Routing (8 bytes) -> UDP
        let hop_by_hop = [43, 0, 0, 0, 0, 0, 0, 0]; // Next: 43 (Routing)
        let routing = [17, 0, 0, 0, 0, 0, 0, 0]; // Next: 17 (UDP)
        let udp_data = [9, 8, 7, 6];
        let payload = [
            hop_by_hop.as_slice(),
            routing.as_slice(),
            udp_data.as_slice(),
        ]
        .concat();

        let packet_bytes = build_ipv6_packet(0, &payload);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Udp);
        assert_eq!(result.offset, 16); // 8 + 8 = 16 bytes
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_first_fragment_has_l4_header() {
        // Fragment header with Offset = 0, MF = 1 pointing to UDP
        let frag_hdr = build_fragment_header(17, 0, true, 0x12345678);
        let udp_data = [1, 2, 3, 4];
        let payload = [frag_hdr.as_slice(), udp_data.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(44, &payload); // 44 = Ipv6Frag
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Udp);
        assert_eq!(result.offset, 8); // Past the 8-byte fragment header
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_subsequent_fragment_no_l4_header() {
        // Fragment header with Offset = 100 (> 0)
        let frag_hdr = build_fragment_header(17, 100, false, 0x12345678);
        let payload_slice = [0xaa, 0xbb, 0xcc, 0xdd];
        let payload = [frag_hdr.as_slice(), payload_slice.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(44, &payload);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Ipv6Frag);
        assert_eq!(result.offset, 0);
        assert!(!result.has_l4_header); // Subsequent fragment must NOT be treated as having an L4 header
    }

    #[test]
    fn test_subsequent_fragment_after_hop_by_hop() {
        // Hop-by-Hop (8 bytes) -> Fragment Header (Offset = 50)
        let hop_by_hop = [44, 0, 0, 0, 0, 0, 0, 0]; // Next: Fragment (44)
        let frag_hdr = build_fragment_header(17, 50, false, 0xabcd);
        let payload = [hop_by_hop.as_slice(), frag_hdr.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(0, &payload);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Ipv6Frag);
        assert_eq!(result.offset, 8); // Offset stops at the fragment header
        assert!(!result.has_l4_header);
    }

    #[test]
    fn test_esp_header_opaque() {
        // Hop-by-Hop (8 bytes) -> ESP (50)
        let hop_by_hop = [50, 0, 0, 0, 0, 0, 0, 0]; // Next: ESP (50)
        let esp_payload = [0x01, 0x02, 0x03, 0x04];
        let payload = [hop_by_hop.as_slice(), esp_payload.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(0, &payload);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Esp);
        assert_eq!(result.offset, 8);
        assert!(!result.has_l4_header); // Encrypted payload is not accessible
    }

    #[test]
    fn test_no_next_header_59() {
        let packet_bytes = build_ipv6_packet(59, &[]); // 59 = Ipv6NoNxt
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Ipv6NoNxt);
        assert_eq!(result.offset, 0);
        assert!(!result.has_l4_header);
    }

    #[test]
    fn test_ah_header_length_calculation() {
        // AH (51): Length = (Payload Len + 2) * 4 bytes
        // Payload Len = 1 => (1 + 2) * 4 = 12 bytes
        let mut ah_header = vec![0u8; 12];
        ah_header[0] = 17; // Next: UDP
        ah_header[1] = 1; // Payload Len = 1

        let udp_data = [1, 2, 3, 4];
        let payload = [ah_header.as_slice(), udp_data.as_slice()].concat();

        let packet_bytes = build_ipv6_packet(51, &payload); // 51 = AH
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        let result = parse_next_protocol(&ipv6).expect("Should parse");
        assert_eq!(result.protocol, IpNextProtocol::Udp);
        assert_eq!(result.offset, 12);
        assert!(result.has_l4_header);
    }

    #[test]
    fn test_defense_against_deep_header_chain_dos() {
        // Chain 9 Hop-by-Hop headers (Limit is 8)
        let mut payload = Vec::new();
        for _ in 0..9 {
            payload.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]); // Next = 0 (Hopopt)
        }

        let packet_bytes = build_ipv6_packet(0, &payload);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        assert!(parse_next_protocol(&ipv6).is_none());
    }

    #[test]
    fn test_truncated_extension_header_out_of_bounds() {
        // Header claims length is (1 + 1) * 8 = 16 bytes, but payload is only 8 bytes
        let malformed_ext_hdr = [17, 1, 0, 0, 0, 0, 0, 0];
        let packet_bytes = build_ipv6_packet(0, &malformed_ext_hdr);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(&packet_bytes).expect("Valid IPv6");

        assert!(parse_next_protocol(&ipv6).is_none());
    }

    #[test]
    fn test_fragment_header_parse_and_offset() {
        // Offset = 42 (0x2A), M = 1, ID = 0xdeadbeef
        let hdr_bytes = build_fragment_header(6, 42, true, 0xdeadbeef);

        let frag = Ipv6FragmentHeader::parse(&hdr_bytes).expect("Should parse");
        assert_eq!(frag.next_header, 6);
        assert_eq!(frag.fragment_offset(), 42);
        assert_eq!(frag.identification.get(), 0xdeadbeef);

        // Truncated buffer (< 8 bytes)
        assert!(Ipv6FragmentHeader::parse(&hdr_bytes[..7]).is_none());
    }

    #[test]
    fn test_get_ext_header_length() {
        // Fragment Header is always 8 bytes
        assert_eq!(
            get_ext_header_length(IpNextProtocol::Ipv6Frag, &[0, 0]),
            Some(8)
        );

        // AH: (PayloadLen + 2) * 4
        assert_eq!(get_ext_header_length(IpNextProtocol::Ah, &[0, 1]), Some(12));
        assert_eq!(get_ext_header_length(IpNextProtocol::Ah, &[0, 3]), Some(20));

        // Standard extension headers: (HdrExtLen + 1) * 8
        assert_eq!(
            get_ext_header_length(IpNextProtocol::Hopopt, &[0, 0]),
            Some(8)
        );
        assert_eq!(
            get_ext_header_length(IpNextProtocol::Ipv6Route, &[0, 1]),
            Some(16)
        );
        assert_eq!(
            get_ext_header_length(IpNextProtocol::Ipv6Opts, &[0, 2]),
            Some(24)
        );

        // ESP is opaque
        assert_eq!(get_ext_header_length(IpNextProtocol::Esp, &[0, 0]), None);

        // Non-extension header
        assert_eq!(get_ext_header_length(IpNextProtocol::Tcp, &[0, 0]), None);

        // Truncated buffer (< 2 bytes)
        assert_eq!(get_ext_header_length(IpNextProtocol::Hopopt, &[0]), None);
    }

    // Helper to construct a valid 40-byte IPv6 header followed by `payload`.
    fn build_ipv6_packet(next_header: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = vec![0u8; 40 + payload.len()];
        // Version 6, Traffic Class 0, Flow Label 0
        packet[0] = 0x60;
        // Payload Length (16-bit big endian)
        let payload_len = (payload.len() as u16).to_be_bytes();
        packet[4] = payload_len[0];
        packet[5] = payload_len[1];
        // Next Header
        packet[6] = next_header;
        // Hop Limit
        packet[7] = 64;
        // Source: 2001:db8::1
        packet[8] = 0x20;
        packet[9] = 0x01;
        packet[10] = 0x0d;
        packet[11] = 0xb8;
        packet[23] = 0x01;
        // Destination: 2001:db8::2
        packet[24] = 0x20;
        packet[25] = 0x01;
        packet[26] = 0x0d;
        packet[27] = 0xb8;
        packet[39] = 0x02;
        // Append payload
        packet[40..].copy_from_slice(payload);
        packet
    }

    // Helper to construct an 8-byte Fragment Header.
    fn build_fragment_header(
        next_header: u8,
        offset: u16,
        more_fragments: bool,
        id: u32,
    ) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0] = next_header;
        buf[1] = 0; // Reserved

        // Offset is in the top 13 bits; lowest bit is M flag
        let mut frag_data = offset << 3;
        if more_fragments {
            frag_data |= 0x0001;
        }
        buf[2..4].copy_from_slice(&frag_data.to_be_bytes());
        buf[4..8].copy_from_slice(&id.to_be_bytes());
        buf
    }
}
