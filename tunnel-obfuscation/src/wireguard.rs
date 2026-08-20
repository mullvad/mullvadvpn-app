//! Some WireGuard helpers.
//!
//! See the [WireGuard whitepaper](https://www.wireguard.com/papers/wireguard.pdf).

use std::{collections::HashSet, ops::Range};

use blake2::{
    Blake2s256, Blake2sMac,
    digest::{Digest, Mac, consts::U16},
};
use talpid_types::net::wireguard::PublicKey;

// First byte of each type of message.
pub const HANDSHAKE_INITIATION: u8 = 1;
pub const HANDSHAKE_RESPONSE: u8 = 2;
pub const COOKIE_REPLY: u8 = 3;
pub const DATA: u8 = 4;

// Handshake messages are a fixed size, so anything else is not one.
pub const HANDSHAKE_INITIATION_SIZE: usize = 148;
pub const HANDSHAKE_RESPONSE_SIZE: usize = 92;
pub const COOKIE_REPLY_SIZE: usize = 64;
// Everything a data message carries besides its payload: the header and the tag.
pub const DATA_OVERHEAD_SIZE: usize = 32;

// Field offsets. `mac1` sits at the end of a response and covers every byte before it.
const SENDER_INDEX: Range<usize> = 4..8;
const RESPONSE_RECEIVER_INDEX: Range<usize> = 8..12;
const COOKIE_RECEIVER_INDEX: Range<usize> = 4..8;
const RESPONSE_MAC1: Range<usize> = 60..76;

/// Keyed BLAKE2s with a 16 byte output.
type Mac1 = Blake2sMac<U16>;

/// What a packet received from the remote turned out to be.
#[derive(Debug, PartialEq, Eq)]
pub enum Received {
    /// Response with a valid `mac1`.
    HandshakeResponse,
    /// Cookie reply.
    CookieReply,
    /// Not an answer to any initiation the filter has seen.
    Unrecognized,
}

/// Recognizes the answers to the initiations passed to [Self::record_initiation].
///
/// This rejects packets that are not potential responses to a handshake initiation from the
/// given client.
pub struct HandshakeFilter {
    /// The `mac1` key, precomputed since it is the same for every packet.
    mac1_key: [u8; 32],
    /// `sender_index` of every initiation recorded so far.
    initiations: HashSet<u32>,
}

impl HandshakeFilter {
    /// `client_public_key` is the key of the local WireGuard instance, which is what a responder
    /// addressing it computes `mac1` with.
    pub fn new(client_public_key: &PublicKey) -> Self {
        Self {
            mac1_key: mac1_key(client_public_key),
            initiations: HashSet::new(),
        }
    }

    /// Note `packet` if it is a handshake initiation, so that its answer can be recognized.
    pub fn record_initiation(&mut self, packet: &[u8]) {
        if packet.len() == HANDSHAKE_INITIATION_SIZE && packet[0] == HANDSHAKE_INITIATION {
            self.initiations.insert(index(packet, SENDER_INDEX));
        }
    }

    pub fn classify(&self, packet: &[u8]) -> Received {
        let answers_an_initiation =
            |field: Range<usize>| self.initiations.contains(&index(packet, field));

        match (packet.first(), packet.len()) {
            (Some(&HANDSHAKE_RESPONSE), HANDSHAKE_RESPONSE_SIZE)
                if answers_an_initiation(RESPONSE_RECEIVER_INDEX) && self.verify_mac1(packet) =>
            {
                Received::HandshakeResponse
            }
            // A cookie reply carries no `mac1`, so the index is all there is to check.
            (Some(&COOKIE_REPLY), COOKIE_REPLY_SIZE)
                if answers_an_initiation(COOKIE_RECEIVER_INDEX) =>
            {
                Received::CookieReply
            }
            _ => Received::Unrecognized,
        }
    }

    fn verify_mac1(&self, response: &[u8]) -> bool {
        mac1(&self.mac1_key)
            .chain_update(&response[..RESPONSE_MAC1.start])
            .verify_slice(&response[RESPONSE_MAC1])
            .is_ok()
    }
}

/// The key that `mac1` of a message addressed to `public_key` is computed with.
fn mac1_key(public_key: &PublicKey) -> [u8; 32] {
    Blake2s256::new()
        .chain_update(b"mac1----")
        .chain_update(public_key.as_bytes())
        .finalize()
        .into()
}

fn mac1(key: &[u8; 32]) -> Mac1 {
    <Mac1 as Mac>::new_from_slice(key).expect("the mac1 key is 32 bytes")
}

fn index(packet: &[u8], field: Range<usize>) -> u32 {
    u32::from_le_bytes(packet[field].try_into().expect("an index is 4 bytes"))
}

/// Build a handshake initiation carrying `sender_index`. Only the fields that identify the
/// handshake are filled in.
#[cfg(test)]
pub fn handshake_initiation(sender_index: u32) -> [u8; HANDSHAKE_INITIATION_SIZE] {
    let mut packet = [0u8; HANDSHAKE_INITIATION_SIZE];
    packet[0] = HANDSHAKE_INITIATION;
    packet[SENDER_INDEX].copy_from_slice(&sender_index.to_le_bytes());
    packet
}

/// Build a handshake response to the initiation that carried `receiver_index`, with a `mac1` that
/// [HandshakeFilter] accepts for `client_public_key`.
#[cfg(test)]
pub fn handshake_response(
    receiver_index: u32,
    client_public_key: &PublicKey,
) -> [u8; HANDSHAKE_RESPONSE_SIZE] {
    let mut packet = [0u8; HANDSHAKE_RESPONSE_SIZE];
    packet[0] = HANDSHAKE_RESPONSE;
    packet[RESPONSE_RECEIVER_INDEX].copy_from_slice(&receiver_index.to_le_bytes());
    let mac = mac1(&mac1_key(client_public_key)).chain_update(&packet[..RESPONSE_MAC1.start]);
    packet[RESPONSE_MAC1].copy_from_slice(&mac.finalize().into_bytes());
    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: u32 = 7;

    fn client_key() -> PublicKey {
        PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap()
    }

    fn other_key() -> PublicKey {
        PublicKey::from_base64("4EkA4c160oQgN/YaNR9GN3gLMevXEfx5hnlc9jYmw14=").unwrap()
    }

    fn response(receiver_index: u32) -> Vec<u8> {
        signed_by(receiver_index, &client_key())
    }

    fn signed_by(receiver_index: u32, key: &PublicKey) -> Vec<u8> {
        handshake_response(receiver_index, key).to_vec()
    }

    fn cookie_reply(receiver_index: u32) -> Vec<u8> {
        let mut packet = vec![0u8; COOKIE_REPLY_SIZE];
        packet[0] = COOKIE_REPLY;
        packet[COOKIE_RECEIVER_INDEX].copy_from_slice(&receiver_index.to_le_bytes());
        packet
    }

    fn filter() -> HandshakeFilter {
        let mut filter = HandshakeFilter::new(&client_key());
        filter.record_initiation(&handshake_initiation(ID));
        filter
    }

    #[test]
    fn classifies_received_packets() {
        // `mac1` covers the ephemeral key, so flipping a bit of it invalidates the response.
        let mut tampered = response(ID);
        tampered[12] ^= 1;
        let mut too_long = response(ID);
        too_long.push(0);
        let mut data = vec![0u8; 1400];
        data[0] = DATA;

        use Received::*;
        let classify = |packet: Vec<u8>| filter().classify(&packet);

        // Answers to the initiation that was recorded
        assert_eq!(classify(response(ID)), HandshakeResponse);
        assert_eq!(classify(cookie_reply(ID)), CookieReply);

        // Answers to an initiation that was never sent
        assert_eq!(classify(response(ID + 1)), Unrecognized);
        assert_eq!(classify(cookie_reply(ID + 1)), Unrecognized);

        // Responses that do not carry a `mac1` we would compute
        assert_eq!(classify(signed_by(ID, &other_key())), Unrecognized);
        assert_eq!(classify(tampered), Unrecognized);

        // Not a handshake response at all
        assert_eq!(classify(too_long), Unrecognized);
        assert_eq!(classify(data), Unrecognized);
        assert_eq!(classify(vec![]), Unrecognized);
        assert_eq!(classify(b"hello".to_vec()), Unrecognized);
    }

    #[test]
    fn records_only_handshake_initiations() {
        let mut short = handshake_initiation(ID).to_vec();
        short.truncate(100);
        let mut wrong_type = handshake_initiation(ID).to_vec();
        wrong_type[0] = DATA;

        for packet in [short, wrong_type] {
            let mut filter = HandshakeFilter::new(&client_key());
            filter.record_initiation(&packet);
            assert_eq!(filter.classify(&response(ID)), Received::Unrecognized);
        }
    }
}
