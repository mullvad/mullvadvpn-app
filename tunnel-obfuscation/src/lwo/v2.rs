//! LWO v2 (Lightweight WireGuard Obfuscation, version 2)
//!
//! The expected calling order on send is [`padding_len`] (append that many random bytes to the
//! packet), then [`obfuscate`]. On receive, [`deobfuscate`] validates the packet and returns a
//! [`Verdict`] telling the caller whether to forward, truncate, or drop it.
//!
//! Correct implementation requires using the WireGuard timers defined in [timers].

use rand::Rng;

// WG message types
type MessageType = u8;
const HANDSHAKE_INIT: MessageType = 1;
const HANDSHAKE_RESP: MessageType = 2;
const COOKIE_REPLY: MessageType = 3;
const DATA: MessageType = 4;

/// Protected (obfuscated) area of a handshake initiation. Also its plaintext size.
const HANDSHAKE_INIT_SZ: usize = 148;
/// Protected (obfuscated) area of a handshake response. Also its plaintext size.
const HANDSHAKE_RESP_SZ: usize = 92;
/// Protected (obfuscated) area of a data packet.
const DATA_PROTECTED_SZ: usize = 16;
/// Minimum size of a valid data packet.
const DATA_MIN_SZ: usize = 32;

/// Maximum number of random padding bytes appended to outgoing handshake packets.
const MAX_PADDING: usize = 256;

/// Marker bits in byte 1 identifying an LWO v2 packet: bit 7 = 0, bit 6 = 1.
const MARKER_MASK: u8 = 0xc0;
const MARKER: u8 = 0x40;

/// Custom WireGuard timer parameters
pub mod timers {
    use core::ops::RangeInclusive;
    use std::time::Duration;

    /// Passive keepalive: 10 s +/- 2 s
    pub const KEEPALIVE_TIMEOUT: RangeInclusive<Duration> =
        Duration::from_secs(8)..=Duration::from_secs(12);
    /// New handshake after silence: 15 s +/- 2 s
    pub const NEW_HANDSHAKE_TIMEOUT: RangeInclusive<Duration> =
        Duration::from_secs(13)..=Duration::from_secs(17);
    /// Handshake retransmit: 5 s +/- 250 ms
    pub const REKEY_TIMEOUT: RangeInclusive<Duration> =
        Duration::from_millis(4750)..=Duration::from_millis(5250);
    /// Rekey-after-time: only ever moved earlier
    pub const REKEY_AFTER_TIME: RangeInclusive<Duration> =
        Duration::from_secs(100)..=Duration::from_secs(120);
}

/// The result of [`deobfuscate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The packet does not claim LWO v2. Pass it through as plain WireGuard.
    Plain,
    /// A valid LWO v2 packet, deobfuscated in place. If `trim_to` is `Some`, the packet must be
    /// truncated to that length before WireGuard processing.
    Lwo { trim_to: Option<usize> },
    /// The packet claims LWO v2 but failed validation, or recovered to an invalid type or
    /// length. It must be dropped.
    Invalid,
}

/// Sample the number of random padding bytes to append to an outgoing WireGuard packet before
/// obfuscating it.
pub fn padding_len(packet: &[u8], rng: &mut impl Rng) -> usize {
    match packet.first() {
        Some(&HANDSHAKE_INIT) | Some(&HANDSHAKE_RESP) => rng.random_range(1..=MAX_PADDING),
        _ => 0,
    }
}

/// Obfuscate a plaintext WireGuard packet in place. `key` is the server's public key.
///
/// Any padding must already have been appended (use [`padding_len`]).
pub fn obfuscate(packet: &mut [u8], key: &[u8; 32]) {
    let Some(&message_type) = packet.first() else {
        return;
    };
    let protected_len = match message_type {
        HANDSHAKE_INIT => HANDSHAKE_INIT_SZ,
        HANDSHAKE_RESP => HANDSHAKE_RESP_SZ,
        DATA => DATA_PROTECTED_SZ,
        // Cookie replies must not be obfuscated. Unknown types are left alone.
        _ => return,
    };
    if packet.len() < protected_len {
        if cfg!(debug_assertions) {
            log::debug!("Missing header bytes for packet of type {message_type:x}");
        }
        return;
    }

    let len_mix = packet.len() as u8; // truncate
    xor_protected_area(&mut packet[..protected_len], key, len_mix);
    packet[1] = marker_byte(key, len_mix);
}

/// Deobfuscate a received packet in place. `key` is the server's public key.
///
/// Validates the marker and reserved bytes, recovers the WireGuard type, and checks that the
/// packet length is valid for that type. See [`Verdict`] for how to proceed.
pub fn deobfuscate(packet: &mut [u8], key: &[u8; 32]) -> Verdict {
    // The shortest valid LWO v2 packet is a 32-byte data packet, but byte 1 decides whether the
    // packet claims LWO v2 at all.
    if packet.len() < DATA_MIN_SZ {
        return match packet.get(1) {
            Some(&reserved) if claims_lwo(reserved) => Verdict::Invalid,
            _ => Verdict::Plain,
        };
    }
    if !claims_lwo(packet[1]) {
        return Verdict::Plain;
    }

    let len_mix = packet.len() as u8; // truncate
    let valid = packet[1] == marker_byte(key, len_mix)
        && packet[2] == obfuscation_byte(key, len_mix, 2)
        && packet[3] == obfuscation_byte(key, len_mix, 3);
    if !valid {
        return Verdict::Invalid;
    }

    let message_type = packet[0] ^ key[0].wrapping_add(len_mix);
    let (protected_len, valid_len, trim_to) = match message_type {
        HANDSHAKE_INIT => (
            HANDSHAKE_INIT_SZ,
            (HANDSHAKE_INIT_SZ..=HANDSHAKE_INIT_SZ + MAX_PADDING).contains(&packet.len()),
            Some(HANDSHAKE_INIT_SZ),
        ),
        HANDSHAKE_RESP => (
            HANDSHAKE_RESP_SZ,
            (HANDSHAKE_RESP_SZ..=HANDSHAKE_RESP_SZ + MAX_PADDING).contains(&packet.len()),
            Some(HANDSHAKE_RESP_SZ),
        ),
        DATA => (DATA_PROTECTED_SZ, packet.len() >= DATA_MIN_SZ, None),
        // Cookie replies are never LWO v2 packets. They must be plain.
        COOKIE_REPLY => return Verdict::Invalid,
        // Unknown WireGuard type.
        _ => return Verdict::Invalid,
    };
    if !valid_len {
        return Verdict::Invalid;
    }

    xor_protected_area(&mut packet[..protected_len], key, len_mix);
    packet[1] = 0;

    Verdict::Lwo { trim_to }
}

/// Whether the reserved byte (byte 1) claims that the packet is LWO v2.
const fn claims_lwo(reserved_byte: u8) -> bool {
    reserved_byte & MARKER_MASK == MARKER
}

/// The finalized value of byte 1: the low bits of its obfuscation byte, plus the marker.
const fn marker_byte(key: &[u8; 32], len_mix: u8) -> u8 {
    (key[0].wrapping_add(len_mix) & !MARKER_MASK) | MARKER
}

/// The value XORed into the packet byte at `index` within the protected area.
const fn obfuscation_byte(key: &[u8; 32], len_mix: u8, index: usize) -> u8 {
    key[index % 32]
        .wrapping_add(len_mix)
        .wrapping_add(index as u8)
}

fn xor_protected_area(protected_area: &mut [u8], key: &[u8; 32], len_mix: u8) {
    for (i, byte) in protected_area.iter_mut().enumerate() {
        *byte ^= obfuscation_byte(key, len_mix, i);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::RngCore;

    const KEY: [u8; 32] = [0xef; 32];

    fn fake_packet(message_type: u8, len: usize) -> Vec<u8> {
        let mut packet = vec![0u8; len];
        rand::rng().fill_bytes(&mut packet[4..]);
        packet[0] = message_type;
        packet[1..4].fill(0);
        packet
    }

    /// Pad and obfuscate like a sender would.
    fn pad_and_obfuscate(packet: &mut Vec<u8>) {
        let padding = padding_len(packet, &mut rand::rng());
        let len = packet.len();
        packet.resize(len + padding, 0);
        rand::rng().fill_bytes(&mut packet[len..]);
        obfuscate(packet, &KEY);
    }

    #[test]
    fn handshake_init_roundtrip() {
        let original = fake_packet(HANDSHAKE_INIT, HANDSHAKE_INIT_SZ);
        let mut packet = original.clone();

        pad_and_obfuscate(&mut packet);
        let padded_len = packet.len();
        assert!((HANDSHAKE_INIT_SZ + 1..=HANDSHAKE_INIT_SZ + MAX_PADDING).contains(&padded_len));
        assert!(claims_lwo(packet[1]));
        assert_ne!(packet[..HANDSHAKE_INIT_SZ], original[..]);

        let verdict = deobfuscate(&mut packet, &KEY);
        assert_eq!(
            verdict,
            Verdict::Lwo {
                trim_to: Some(HANDSHAKE_INIT_SZ)
            }
        );
        assert_eq!(packet[..HANDSHAKE_INIT_SZ], original[..]);
    }

    #[test]
    fn handshake_resp_roundtrip() {
        let original = fake_packet(HANDSHAKE_RESP, HANDSHAKE_RESP_SZ);
        let mut packet = original.clone();

        pad_and_obfuscate(&mut packet);

        let verdict = deobfuscate(&mut packet, &KEY);
        assert_eq!(
            verdict,
            Verdict::Lwo {
                trim_to: Some(HANDSHAKE_RESP_SZ)
            }
        );
        assert_eq!(packet[..HANDSHAKE_RESP_SZ], original[..]);
    }

    #[test]
    fn data_roundtrip() {
        let original = fake_packet(DATA, DATA_MIN_SZ + 100);
        let mut packet = original.clone();

        pad_and_obfuscate(&mut packet);
        assert_eq!(packet.len(), original.len(), "data packets are not padded");
        assert!(claims_lwo(packet[1]));
        assert_eq!(
            packet[DATA_PROTECTED_SZ..],
            original[DATA_PROTECTED_SZ..],
            "payload beyond the protected area should be unchanged"
        );

        let verdict = deobfuscate(&mut packet, &KEY);
        assert_eq!(verdict, Verdict::Lwo { trim_to: None });
        assert_eq!(packet, original);
    }

    #[test]
    fn cookie_reply_passes_through() {
        let original = fake_packet(COOKIE_REPLY, 64);
        let mut packet = original.clone();

        pad_and_obfuscate(&mut packet);
        assert_eq!(packet, original, "cookie replies must not be obfuscated");

        assert_eq!(deobfuscate(&mut packet, &KEY), Verdict::Plain);
        assert_eq!(packet, original);
    }

    #[test]
    fn plain_wireguard_passes_through() {
        let original = fake_packet(DATA, DATA_MIN_SZ + 100);
        let mut packet = original.clone();
        assert_eq!(deobfuscate(&mut packet, &KEY), Verdict::Plain);
        assert_eq!(packet, original);
    }

    #[test]
    fn tampered_packet_is_dropped() {
        for byte in 1..4 {
            let mut packet = fake_packet(DATA, DATA_MIN_SZ + 100);
            obfuscate(&mut packet, &KEY);
            packet[byte] ^= 0x01;
            // Flipping a bit in byte 1 may also clear the marker, making the packet plain.
            // Bytes 2 and 3 must always fail validation.
            let verdict = deobfuscate(&mut packet, &KEY);
            assert_ne!(
                verdict,
                Verdict::Lwo { trim_to: None },
                "tampered byte {byte} must not validate",
            );
        }
    }

    #[test]
    fn recovered_cookie_reply_is_dropped() {
        // Craft a packet that validates but recovers as a cookie reply.
        let mut packet = fake_packet(COOKIE_REPLY, 64);
        let len_mix = packet.len() as u8;
        packet[0] = COOKIE_REPLY ^ KEY[0].wrapping_add(len_mix);
        packet[1] = marker_byte(&KEY, len_mix);
        packet[2] = obfuscation_byte(&KEY, len_mix, 2);
        packet[3] = obfuscation_byte(&KEY, len_mix, 3);

        assert_eq!(deobfuscate(&mut packet, &KEY), Verdict::Invalid);
    }

    #[test]
    fn invalid_lengths_are_dropped() {
        // A data packet shorter than 32 bytes claiming LWO.
        let mut short_data = fake_packet(DATA, DATA_MIN_SZ - 1);
        obfuscate(&mut short_data, &KEY);
        // Force the marker in case the length made the packet skip obfuscation.
        short_data[1] = marker_byte(&KEY, short_data.len() as u8);
        assert_eq!(deobfuscate(&mut short_data, &KEY), Verdict::Invalid);

        // An over-padded handshake initiation.
        let len = HANDSHAKE_INIT_SZ + MAX_PADDING + 1;
        let mut oversized = fake_packet(HANDSHAKE_INIT, len);
        let len_mix = len as u8;
        oversized[0] = HANDSHAKE_INIT ^ KEY[0].wrapping_add(len_mix);
        oversized[1] = marker_byte(&KEY, len_mix);
        oversized[2] = obfuscation_byte(&KEY, len_mix, 2);
        oversized[3] = obfuscation_byte(&KEY, len_mix, 3);
        assert_eq!(deobfuscate(&mut oversized, &KEY), Verdict::Invalid);
    }

    #[test]
    fn length_is_mixed_into_obfuscation() {
        // The same plaintext at two different padded lengths obfuscate differently.
        let original = fake_packet(DATA, DATA_MIN_SZ + 100);

        let mut a = original.clone();
        obfuscate(&mut a, &KEY);

        let mut b = original.clone();
        b.push(0);
        obfuscate(&mut b, &KEY);

        assert_ne!(a[..DATA_PROTECTED_SZ], b[..DATA_PROTECTED_SZ]);

        // And a packet deobfuscated at the wrong length does not validate.
        let mut truncated = a.clone();
        truncated.pop();
        assert_ne!(
            deobfuscate(&mut truncated, &KEY),
            Verdict::Lwo { trim_to: None }
        );
    }
}
