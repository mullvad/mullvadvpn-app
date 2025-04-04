use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use h3::proto::varint::VarInt;

#[derive(Default)]
pub struct Fragments {
    fragment_map: BTreeMap<u16, Vec<Fragment>>,
}

// When a packet that arrives is too small to be decoded.
#[derive(Debug)]
pub enum DefragError {
    #[allow(dead_code)] // TODO: use this error or remove it.
    BadContextId(Result<VarInt, h3::proto::coding::UnexpectedEnd>),
    PayloadTooSmall,
}

// When a packet is larger than u16::MAX, it can't be fragmented.
#[derive(Debug, thiserror::Error)]
#[error("Packet is too large to fragment")]
pub struct PacketTooLarge(pub usize);

impl Fragments {
    // TODO: Let caller provide output buffer.
    pub fn handle_incoming_packet(
        &mut self,
        mut payload: Bytes,
    ) -> Result<Option<Bytes>, DefragError> {
        match VarInt::decode(&mut payload) {
            Ok(crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID) => {
                return Ok(Some(payload));
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
        let fragment = Fragment {
            index,
            payload,
            time_received: Instant::now(),
        };

        let fragments = self.fragment_map.entry(id).or_default();
        fragments.push(fragment);

        Ok(self.try_fetch(id, fragment_count))
    }

    // TODO: Let caller provide output buffer.
    fn try_fetch(&mut self, id: u16, fragment_count: u8) -> Option<Bytes> {
        // establish that there are enough fragments to reconstruct the whole packet
        let payload = {
            let fragments = self.fragment_map.get_mut(&id)?;

            if fragments.len() != fragment_count.into() {
                return None;
            }

            fragments.sort_by_key(|f| f.index);
            let mut payload =
                BytesMut::with_capacity(fragments.iter().map(|f| f.payload.len()).sum());
            for fragment in fragments {
                payload.extend_from_slice(&fragment.payload);
            }
            payload
        };

        self.fragment_map.remove(&id);
        Some(payload.into())
    }

    pub fn clear_old_fragments(&mut self, max_age: Duration) {
        self.fragment_map.retain(|_, fragments| {
            fragments
                .iter()
                .any(|fragment| fragment.time_received.elapsed() <= max_age)
        });
    }
}

struct Fragment {
    index: u8,
    payload: Bytes,
    time_received: Instant,
}

pub fn fragment_packet(
    maximum_packet_size: u16,
    payload: &'_ mut Bytes,
    packet_id: u16,
) -> Result<impl Iterator<Item = Bytes> + '_, PacketTooLarge> {
    let num_fragments: usize = payload.chunks(maximum_packet_size.into()).count();
    let Ok(fragment_count): std::result::Result<u8, _> = num_fragments.try_into() else {
        return Err(PacketTooLarge(payload.len()));
    };

    let iterator = payload.chunks(maximum_packet_size.into()).enumerate().map(
        move |(fragment_index, fragment_payload)| {
            let mut fragment = BytesMut::with_capacity((maximum_packet_size + 1).into());
            crate::HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID.encode(&mut fragment);
            fragment.put_u16(packet_id);
            fragment.put_u8(
                // fragment indexes start at 1
                u8::try_from(fragment_index + 1)
                    .expect("fragment index must fit in an u8, since num_fragments fits is an u8"),
            );
            fragment.put_u8(fragment_count);
            fragment.extend_from_slice(fragment_payload);
            fragment.freeze()
        },
    );
    Ok(iterator)
}

#[test]
fn test_fragment_reconstruction() {
    use rand::{seq::SliceRandom, thread_rng};

    let payload = (0..255).collect::<Vec<u8>>();
    let max_payload_size = 50;
    let packet_id = 76;

    let mut fragments = Fragments::default();

    let mut payload_clone = Bytes::from(payload.clone());
    let mut fragment_buf = fragment_packet(max_payload_size, &mut payload_clone, packet_id)
        .unwrap()
        .collect::<Vec<_>>();

    fragment_buf.shuffle(&mut thread_rng());

    for fragment in fragment_buf {
        if let Some(reconstructed_packet) = fragments.handle_incoming_packet(fragment).unwrap() {
            assert_eq!(payload.as_slice(), reconstructed_packet.as_ref());
            return;
        }
    }

    panic!("Failed to reconstruct packet");
}
