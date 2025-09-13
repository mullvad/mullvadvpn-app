use std::collections::{BTreeMap, VecDeque, btree_map};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use h3::proto::varint::VarInt;

use crate::FRAGMENT_HEADER_SIZE_FRAGMENTED;

/// The index of the first fragment of a packet.
const FRAGMENT_INDEX_START: u8 = 1;

/// The maximum number of unassembled fragments that we buffer.
// 255 is the theoretical maximum number of fragments for a single packet.
pub const FRAGMENT_BUFFER_CAP: usize = 255;

pub struct Fragments {
    /// FIFO queue of fragment indices. Used to mitigate floods of unordered packet fragments.
    ///
    /// When receiving a fragment, push it to the back of the queue.  When the queue length exceeds
    /// [FRAGMENT_BUFFER_CAP], pop the first element and remove it from [Self::fragment_map].
    fragment_index_fifo: VecDeque<u16>,

    /// Map of fragmented packets.
    ///
    /// If fragments are arriving in order, this should never hold more than one set of fragments.
    ///
    /// INVARIANT: The `Vec` is sorted by `Fragment::index`
    // TODO: would a hashmap be faster?
    fragment_map: BTreeMap<u16, Vec<Fragment>>,
}

// When a packet that arrives is too small to be decoded.
#[derive(Debug, thiserror::Error)]
pub enum DefragError {
    #[error("Bad context id: {:?}", .0)]
    BadContextId(Result<VarInt, h3::proto::coding::UnexpectedEnd>),

    #[error("Payload is too small")]
    PayloadTooSmall,

    #[error("Too few fragments in fragmented packet")]
    TooFewFragments,

    #[error("Received a fragment twice")]
    DuplicateFragment,
}

// When a packet is larger than u16::MAX, it can't be fragmented.
#[derive(Debug, thiserror::Error)]
#[error("Packet is too large to fragment")]
pub struct PacketTooLarge(pub usize);

impl Default for Fragments {
    fn default() -> Self {
        Self {
            fragment_index_fifo: VecDeque::with_capacity(FRAGMENT_BUFFER_CAP),
            fragment_map: Default::default(),
        }
    }
}

pub enum DefragReceived {
    /// Received a whole packet without fragmentation
    Nonfragmented(Bytes),
    /// Received a fragment but was unable to reassemble the packet
    Fragment,
    /// Received reassembled packet
    Reassembled(Bytes),
}

impl Fragments {
    // TODO: Let caller provide output buffer.
    pub fn handle_incoming_packet(
        &mut self,
        mut payload: Bytes,
    ) -> Result<DefragReceived, DefragError> {
        match VarInt::decode(&mut payload) {
            Ok(crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID) => {
                return Ok(DefragReceived::Nonfragmented(payload));
            }
            Ok(crate::HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID) => {}
            unexpected_context_id => {
                return Err(DefragError::BadContextId(unexpected_context_id));
            }
        }

        let id = payload
            .try_get_u16()
            .map_err(|_| DefragError::PayloadTooSmall)?;
        let index = payload
            .try_get_u8()
            .map_err(|_| DefragError::PayloadTooSmall)?;
        let fragment_count = payload
            .try_get_u8()
            .map_err(|_| DefragError::PayloadTooSmall)?;
        if fragment_count < 2 {
            // Packets with only one fragment should be sent as non-fragmented packets.
            return Err(DefragError::TooFewFragments);
        }
        let fragment = Fragment { index, payload };

        // ensure that the fifo has capacity before pushing the new fragment id
        if self.fragment_index_fifo.len() >= FRAGMENT_BUFFER_CAP {
            let id = self.fragment_index_fifo.pop_front().expect("fifo is full");
            if self.fragment_map.remove(&id).is_some() && cfg!(debug_assertions) {
                log::debug!("Fragment was discarded before reassembly");
            };
        }
        self.fragment_index_fifo.push_back(id);

        debug_assert_eq!(
            self.fragment_index_fifo.capacity(),
            FRAGMENT_BUFFER_CAP,
            "fragment_index_fifo must never grow",
        );

        let entry = self.fragment_map.entry(id);

        let mut entry = match entry {
            btree_map::Entry::Occupied(occupied) => occupied,

            // if this is the first received fragment, don't bother trying to reassemble
            btree_map::Entry::Vacant(vacant) => {
                let mut fragment_list = Vec::with_capacity(2); // two fragments should be the norm
                fragment_list.push(fragment);
                vacant.insert(fragment_list);
                return Ok(DefragReceived::Fragment);
            }
        };

        let fragments = entry.get_mut();

        // insert the fragment such that the list is sorted
        match fragments.binary_search_by_key(&fragment.index, |f| f.index) {
            Err(insert_here) => fragments.insert(insert_here, fragment),
            Ok(_) => return Err(DefragError::DuplicateFragment),
        };

        // establish that there are enough fragments to reconstruct the whole packet
        if fragments.len() != fragment_count.into() {
            return Ok(DefragReceived::Fragment);
        }

        let fragments = entry.remove();

        // smush the fragments together
        let mut payload = BytesMut::with_capacity(fragments.iter().map(|f| f.payload.len()).sum());
        for fragment in fragments {
            payload.extend_from_slice(&fragment.payload);
        }

        Ok(DefragReceived::Reassembled(payload.freeze()))
    }
}

struct Fragment {
    index: u8,
    payload: Bytes,
}

/// Fragment packet using the given maximum fragment size (including headers).
///
/// `payload` must not contain any fragmentation headers.
/// `maximum_packet_size` is the maximum fragment size including headers.
pub fn fragment_packet(
    maximum_packet_size: u16,
    payload: &'_ mut Bytes,
    packet_id: u16,
) -> Result<impl Iterator<Item = Bytes> + '_, PacketTooLarge> {
    let fragment_payload_size = maximum_packet_size - FRAGMENT_HEADER_SIZE_FRAGMENTED;

    let num_fragments: usize = payload.chunks(fragment_payload_size.into()).count();
    let Ok(fragment_count): std::result::Result<u8, _> = num_fragments.try_into() else {
        return Err(PacketTooLarge(payload.len()));
    };

    let iterator = payload
        .chunks(fragment_payload_size.into())
        .enumerate()
        .map(move |(fragment_index, fragment_payload)| {
            let mut fragment = BytesMut::with_capacity(usize::from(
                fragment_payload_size + FRAGMENT_HEADER_SIZE_FRAGMENTED,
            ));
            crate::HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID.encode(&mut fragment);
            fragment.put_u16(packet_id);
            fragment.put_u8(
                u8::try_from(fragment_index + usize::from(FRAGMENT_INDEX_START))
                    .expect("fragment index must fit in an u8, since num_fragments fits is an u8"),
            );
            fragment.put_u8(fragment_count);

            debug_assert!(fragment.len() == usize::from(FRAGMENT_HEADER_SIZE_FRAGMENTED));

            fragment.extend_from_slice(fragment_payload);
            fragment.freeze()
        });
    Ok(iterator)
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::{rng, seq::SliceRandom};

    use super::*;

    #[test]
    fn test_fragment_reconstruction() {
        let mut fragments = Fragments::default();

        let max_payload_size = 50;
        'outer: for packet_id in max_payload_size..255u16 {
            let payload = (0..packet_id as u8).collect::<Vec<u8>>();

            let mut payload_clone = Bytes::from(payload.clone());
            let mut fragment_buf = fragment_packet(max_payload_size, &mut payload_clone, packet_id)
                .unwrap()
                .collect::<Vec<_>>();

            fragment_buf.shuffle(&mut rng());

            for fragment in fragment_buf {
                if let DefragReceived::Reassembled(reconstructed_packet) =
                    fragments.handle_incoming_packet(fragment).unwrap()
                {
                    assert_eq!(payload.as_slice(), reconstructed_packet.as_ref());
                    continue 'outer;
                }
            }

            panic!("Failed to reconstruct packet");
        }
    }

    #[test]
    fn test_interleaved_fragment_reconstruction() {
        let mut fragments = Fragments::default();

        let n_packets = 10;
        let payload_len = 255;
        let max_payload_size = 50;

        let mut fragment_buf = Vec::new();
        let mut payloads = HashSet::new();
        for i in 0..n_packets {
            let packet_id = i as u16;
            let mut payload = Bytes::from(vec![i as u8; payload_len]);
            payloads.insert(payload.clone());

            fragment_buf
                .extend(&mut fragment_packet(max_payload_size, &mut payload, packet_id).unwrap());
        }
        fragment_buf.shuffle(&mut rng());

        for fragment in fragment_buf {
            if let DefragReceived::Reassembled(reconstructed_packet) =
                fragments.handle_incoming_packet(fragment).unwrap()
            {
                assert!(
                    payloads.remove(&reconstructed_packet),
                    "reconstructed corrupted or duplicate packet"
                );
            }
        }

        assert!(payloads.is_empty(), "Some packets were not reconstructed");
    }

    #[test]
    fn test_fragment_cap() {
        // test whether we can reassemble a fragmented packet when we receive a flood of bad fragments
        // interspersed with our good fragments. returns true if reassembly was successful.
        let fragment_survives_flood = |number_of_bad_fragments| {
            let mut fragments = Fragments::default();

            let packet_id = 1;
            let mut bad_packet_ids = 2..0xffff;

            let payload = (0..255).collect::<Vec<u8>>();
            let max_payload_size = 50;

            let mut payload_clone = Bytes::from(payload.clone());
            let mut fragment_buf = fragment_packet(max_payload_size, &mut payload_clone, packet_id)
                .unwrap()
                .collect::<Vec<_>>();

            fragment_buf.shuffle(&mut rng());

            // send one fragment
            let packet = fragments
                .handle_incoming_packet(fragment_buf.pop().unwrap())
                .unwrap();
            assert!(
                matches!(packet, DefragReceived::Fragment),
                "haven't sent all fragments yet"
            );

            // then send a bunch of fragments to fill the queue
            let mut bad_payload = Bytes::from([0u8; 2].to_vec());
            for _ in fragment_buf.len()..number_of_bad_fragments {
                let incomplete_fragment = fragment_packet(
                    1 + FRAGMENT_HEADER_SIZE_FRAGMENTED,
                    &mut bad_payload,
                    bad_packet_ids.next().unwrap(),
                )
                .unwrap()
                .next()
                .unwrap();

                let packet = fragments
                    .handle_incoming_packet(incomplete_fragment.clone())
                    .unwrap();
                assert!(matches!(packet, DefragReceived::Fragment));
            }

            for fragment in fragment_buf {
                if let DefragReceived::Reassembled(reconstructed_packet) =
                    fragments.handle_incoming_packet(fragment).unwrap()
                {
                    assert_eq!(payload.as_slice(), reconstructed_packet.as_ref());
                    return true;
                }
            }

            false
        };

        assert!(fragment_survives_flood(FRAGMENT_BUFFER_CAP - 1));
        assert!(!fragment_survives_flood(FRAGMENT_BUFFER_CAP));
    }
}
