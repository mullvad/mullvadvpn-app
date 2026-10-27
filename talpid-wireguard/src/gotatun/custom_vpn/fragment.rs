//! Reassembly of IPv6 fragments.
// TODO: This could be upstreamed to [gotatun], which currently only handles IPv4 reassembly for
// channels.

use super::ipv6::Ipv6FragmentHeader;
use gotatun::packet::{Ip, IpNextProtocol, Ipv6, Ipv6Header, Packet};
use std::{collections::VecDeque, net::Ipv6Addr};
use zerocopy::IntoBytes;

/// The maximum number of datagrams that can be reassembled concurrently. When this limit is
/// reached, the fragments of the oldest datagram are dropped to make space.
///
/// A datagram cannot exceed 64 KiB, so at most 4 MiB is buffered.
const MAX_CONCURRENT_DATAGRAMS: usize = 64;

#[derive(Debug, PartialEq, Eq)]
struct FragmentId {
    source: Ipv6Addr,
    destination: Ipv6Addr,
    identification: u32,
}

struct Fragment {
    /// Offset of `payload` in the reassembled payload, in bytes.
    offset: usize,
    more_fragments: bool,
    /// The protocol of the reassembled payload. Only meaningful for the first fragment.
    next_header: IpNextProtocol,
    payload: Vec<u8>,
}

impl Fragment {
    fn end(&self) -> usize {
        self.offset + self.payload.len()
    }
}

/// A datagram being reassembled.
struct Datagram {
    id: FragmentId,
    /// The fixed header of the first fragment received.
    header: Ipv6Header,
    /// INVARIANT: Sorted by offset, and non-overlapping.
    fragments: Vec<Fragment>,
}

/// A buffer for reassembling IPv6 datagrams from their fragments.
///
/// Only fragments whose Fragment header directly follows the fixed IPv6 header are supported.
/// This assumes that WireGuard won't add any other extension headers. Nodes along the path may
/// not insert extension headers either, see
/// [RFC 8200, section 4](https://www.rfc-editor.org/rfc/rfc8200#section-4).
#[derive(Default)]
pub struct Ipv6Fragments {
    /// FIFO queue of datagrams being reassembled, oldest first.
    datagrams: VecDeque<Datagram>,
}

impl Ipv6Fragments {
    /// Returns whether `ipv6` is a fragment that [`Ipv6Fragments::assemble`] can reassemble.
    pub fn is_fragment(ipv6: &Ipv6) -> bool {
        ipv6.header.next_protocol() == IpNextProtocol::Ipv6Frag
    }

    /// Add the fragment `ipv6` to the buffer. Returns the reassembled datagram once all of its
    /// fragments have been added.
    ///
    /// Invalid fragments are dropped. Per [RFC 8200], a fragment that overlaps another fragment
    /// of the same datagram causes the whole datagram to be dropped, unless it is an exact
    /// duplicate.
    ///
    /// [RFC 8200]: https://www.rfc-editor.org/rfc/rfc8200#section-4.5
    pub fn assemble(&mut self, ipv6: &Ipv6) -> Option<Packet<Ip>> {
        if !Self::is_fragment(ipv6) {
            return None;
        }
        let payload = ipv6
            .payload
            .get(..usize::from(ipv6.header.payload_length.get()))?;
        let fragment_header = Ipv6FragmentHeader::parse(payload)?;
        let fragment = Fragment {
            offset: usize::from(fragment_header.fragment_offset()) * 8,
            more_fragments: fragment_header.more_fragments(),
            next_header: fragment_header.next_header,
            payload: payload[size_of::<Ipv6FragmentHeader>()..].to_vec(),
        };
        let id = FragmentId {
            source: ipv6.header.source(),
            destination: ipv6.header.destination(),
            identification: fragment_header.identification.get(),
        };

        // An atomic fragment is a whole datagram (RFC 6946)
        if fragment.offset == 0 && !fragment.more_fragments {
            return Some(reassemble(ipv6.header, vec![fragment]));
        }

        // All fragments except the last must have a length that is a multiple of 8 bytes, and the
        // reassembled payload must fit in an IPv6 packet.
        if (fragment.more_fragments && !fragment.payload.len().is_multiple_of(8))
            || fragment.end() > usize::from(u16::MAX)
        {
            log::trace!(
                "Invalid IPv6 fragment with offset {} and length {}, dropping",
                fragment.offset,
                fragment.payload.len(),
            );
            return None;
        }

        let Some(pos) = self.datagrams.iter().position(|datagram| datagram.id == id) else {
            if self.datagrams.len() >= MAX_CONCURRENT_DATAGRAMS {
                let dropped = self.datagrams.pop_front().expect("datagrams is not empty");
                log::trace!(
                    "Too many IPv6 datagrams being reassembled, dropping the oldest: {:?}",
                    dropped.id
                );
            }
            self.datagrams.push_back(Datagram {
                id,
                header: ipv6.header,
                fragments: vec![fragment],
            });
            return None;
        };

        let fragments = &mut self.datagrams[pos].fragments;
        let i = fragments.partition_point(|other| other.offset < fragment.offset);
        if let Some(other) = fragments.get(i)
            && other.offset == fragment.offset
            && other.payload == fragment.payload
        {
            log::trace!("Dropping duplicate IPv6 fragment for {id:?}");
            return None;
        }
        let overlaps_previous = i > 0 && fragments[i - 1].end() > fragment.offset;
        let overlaps_next = fragments
            .get(i)
            .is_some_and(|next| fragment.end() > next.offset);
        if overlaps_previous || overlaps_next {
            log::trace!("Overlapping IPv6 fragments for {id:?}, dropping the datagram");
            self.datagrams.remove(pos);
            return None;
        }
        fragments.insert(i, fragment);

        // The datagram is complete if each fragment begins where the previous one ended, the
        // first one is at offset 0, and only the last one lacks the "more fragments" flag.
        let (last, rest) = fragments.split_last().expect("fragments is not empty");
        let complete = fragments[0].offset == 0
            && !last.more_fragments
            && rest.iter().all(|fragment| fragment.more_fragments)
            && fragments
                .windows(2)
                .all(|pair| pair[0].end() == pair[1].offset);
        if !complete {
            return None;
        }

        let datagram = self.datagrams.remove(pos).expect("datagram exists");
        Some(reassemble(datagram.header, datagram.fragments))
    }
}

/// Build an unfragmented packet from `header` and the complete, sorted `fragments`.
fn reassemble(mut header: Ipv6Header, fragments: Vec<Fragment>) -> Packet<Ip> {
    let payload_len = fragments.last().map(Fragment::end).unwrap_or(0);
    header.next_header = fragments[0].next_header;
    header.payload_length = u16::try_from(payload_len)
        .expect("payload length was checked")
        .into();

    let mut bytes = Vec::with_capacity(Ipv6Header::LEN + payload_len);
    bytes.extend_from_slice(header.as_bytes());
    for fragment in &fragments {
        bytes.extend_from_slice(&fragment.payload);
    }
    Packet::copy_from(&bytes[..])
        .try_into_ip()
        .expect("an IPv6 header is longer than the minimum IP packet length")
}

#[cfg(test)]
pub mod tests {
    use super::super::packet::{UdpPacket, tests::udp_packet};
    use super::*;
    use std::net::SocketAddr;
    use zerocopy::FromBytes;

    /// Split an IPv6 packet into fragments with `lens` bytes of payload each (the last one gets
    /// the rest), with the Fragment header directly after the fixed header.
    pub fn fragment_v6(
        packet: &Packet<Ip>,
        identification: u32,
        lens: &[usize],
    ) -> Vec<Packet<Ip>> {
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap();
        let mut chunks = vec![];
        let mut rest = &ipv6.payload[..];
        for &len in lens {
            let (chunk, tail) = rest.split_at(len);
            chunks.push(chunk);
            rest = tail;
        }
        chunks.push(rest);

        let mut offset = 0;
        let count = chunks.len();
        chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                let more = i + 1 < count;
                let packet = fragment_packet(&ipv6.header, identification, offset, more, chunk);
                offset += chunk.len();
                packet
            })
            .collect()
    }

    fn fragment_packet(
        original: &Ipv6Header,
        identification: u32,
        offset: usize,
        more: bool,
        payload: &[u8],
    ) -> Packet<Ip> {
        let mut header = *original;
        header.next_header = IpNextProtocol::Ipv6Frag;
        header.payload_length = u16::try_from(size_of::<Ipv6FragmentHeader>() + payload.len())
            .unwrap()
            .into();
        let fragment_header = Ipv6FragmentHeader {
            next_header: original.next_header,
            reserved: 0,
            frag_data: (u16::try_from(offset / 8).unwrap() << 3 | u16::from(more)).into(),
            identification: identification.into(),
        };
        let bytes = [header.as_bytes(), fragment_header.as_bytes(), payload].concat();
        Packet::copy_from(&bytes[..]).try_into_ip().unwrap()
    }

    fn datagram() -> Packet<Ip> {
        let payload: Vec<u8> = (0..200).map(|i| i as u8).collect();
        udp_packet(
            "[2001:db8::1]:51820".parse().unwrap(),
            "[fc00::2]:1234".parse().unwrap(),
            &payload,
        )
    }

    fn assemble(fragments: &mut Ipv6Fragments, packet: &Packet<Ip>) -> Option<Packet<Ip>> {
        fragments.assemble(Ipv6::<[u8]>::ref_from_bytes(packet.as_bytes()).unwrap())
    }

    #[test]
    fn reassembles_in_any_order() {
        let datagram = datagram();
        let mut parts = fragment_v6(&datagram, 1, &[64, 80]);
        parts.reverse();

        let mut fragments = Ipv6Fragments::default();
        assert!(assemble(&mut fragments, &parts[0]).is_none());
        assert!(assemble(&mut fragments, &parts[1]).is_none());
        let reassembled = assemble(&mut fragments, &parts[2]).unwrap();

        assert_eq!(reassembled.as_bytes(), datagram.as_bytes());
        let udp = UdpPacket::parse(&reassembled).unwrap();
        assert_eq!(
            SocketAddr::new(udp.source, udp.source_port()),
            "[2001:db8::1]:51820".parse().unwrap()
        );
        assert!(fragments.datagrams.is_empty());
    }

    #[test]
    fn atomic_fragment_is_unwrapped() {
        let datagram = datagram();
        let [atomic] = &fragment_v6(&datagram, 1, &[])[..] else {
            unreachable!()
        };
        let mut fragments = Ipv6Fragments::default();
        let reassembled = assemble(&mut fragments, atomic).unwrap();
        assert_eq!(reassembled.as_bytes(), datagram.as_bytes());
    }

    #[test]
    fn fragments_of_different_datagrams_are_kept_apart() {
        let datagram = datagram();
        let a = fragment_v6(&datagram, 1, &[64]);
        let b = fragment_v6(&datagram, 2, &[64]);

        let mut fragments = Ipv6Fragments::default();
        assert!(assemble(&mut fragments, &a[0]).is_none());
        assert!(assemble(&mut fragments, &b[1]).is_none());
        assert!(assemble(&mut fragments, &a[1]).is_some());
        assert!(assemble(&mut fragments, &b[0]).is_some());
    }

    #[test]
    fn duplicate_fragment_is_ignored() {
        let datagram = datagram();
        let parts = fragment_v6(&datagram, 1, &[64]);

        let mut fragments = Ipv6Fragments::default();
        assert!(assemble(&mut fragments, &parts[0]).is_none());
        assert!(assemble(&mut fragments, &parts[0]).is_none());
        let reassembled = assemble(&mut fragments, &parts[1]).unwrap();
        assert_eq!(reassembled.as_bytes(), datagram.as_bytes());
    }

    #[test]
    fn overlapping_fragment_drops_datagram() {
        let datagram = datagram();
        let parts = fragment_v6(&datagram, 1, &[64]);
        let ipv6 = Ipv6::<[u8]>::ref_from_bytes(datagram.as_bytes()).unwrap();
        let overlapping = fragment_packet(&ipv6.header, 1, 56, true, &ipv6.payload[56..72]);

        let mut fragments = Ipv6Fragments::default();
        assert!(assemble(&mut fragments, &parts[0]).is_none());
        assert!(assemble(&mut fragments, &overlapping).is_none());
        assert!(assemble(&mut fragments, &parts[1]).is_none());
        assert_eq!(
            fragments.datagrams.len(),
            1,
            "only the last fragment remains"
        );
    }

    #[test]
    fn misaligned_fragment_is_dropped() {
        let datagram = datagram();
        let parts = fragment_v6(&datagram, 1, &[60]);

        let mut fragments = Ipv6Fragments::default();
        assert!(assemble(&mut fragments, &parts[0]).is_none());
        assert!(fragments.datagrams.is_empty());
    }

    #[test]
    fn oldest_datagram_is_evicted() {
        let datagram = datagram();
        let mut fragments = Ipv6Fragments::default();
        let first = fragment_v6(&datagram, 0, &[64]);
        assert!(assemble(&mut fragments, &first[0]).is_none());
        for id in 1..=MAX_CONCURRENT_DATAGRAMS as u32 {
            assert!(assemble(&mut fragments, &fragment_v6(&datagram, id, &[64])[0]).is_none());
        }
        assert_eq!(fragments.datagrams.len(), MAX_CONCURRENT_DATAGRAMS);
        assert!(assemble(&mut fragments, &first[1]).is_none());
    }
}
