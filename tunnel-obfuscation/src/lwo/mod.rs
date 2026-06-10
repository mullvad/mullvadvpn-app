//! LWO (Lightweight WireGuard Obfuscation)

pub mod v2;

use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use rand::RngCore;
use talpid_net::bypass::{BypassSocket, SocketBypass};
use talpid_types::net::{obfuscation::LwoVersion, wireguard::PublicKey};
use tokio::net::UdpSocket;

use crate::{socket::create_remote_socket, transport::ObfuscatedTransport};

#[derive(Debug, Clone)]
pub struct Settings {
    /// Remote LWO/WG server
    pub server_addr: SocketAddr,
    /// Public key of the WG client
    pub client_public_key: PublicKey,
    /// Public key of the WG server
    pub server_public_key: PublicKey,
    /// Which version of the protocol the server speaks
    pub version: LwoVersion,
}

/// The keys used to obfuscate traffic, which differ between protocol versions.
enum Keys {
    /// v1 obfuscates each direction with a different key.
    V1 {
        /// Key that the server obfuscates incoming packets with.
        rx_key: PublicKey,
        /// Key to obfuscate outgoing packets with.
        tx_key: PublicKey,
    },
    /// v2 obfuscates both directions with the server public key.
    V2 { key: PublicKey },
}

impl Keys {
    fn new(settings: &Settings) -> Self {
        match settings.version {
            LwoVersion::V1 => Keys::V1 {
                rx_key: settings.client_public_key.clone(),
                tx_key: settings.server_public_key.clone(),
            },
            LwoVersion::V2 => Keys::V2 {
                key: settings.server_public_key.clone(),
            },
        }
    }
}

impl Settings {
    /// The overhead (in bytes) that this obfuscation protocol adds to every packet.
    pub fn packet_overhead(&self) -> u16 {
        0
    }
}

/// Obfuscates the WireGuard header in place and forwards the packet to the LWO/WG server.
pub struct Lwo {
    remote_socket: BypassSocket<UdpSocket>,
    server_addr: SocketAddr,
    keys: Keys,
}

impl Lwo {
    pub async fn new(bypass: Arc<dyn SocketBypass>, settings: &Settings) -> crate::Result<Self> {
        let remote_socket = create_remote_socket(&bypass, settings.server_addr.is_ipv4()).await?;

        Ok(Self {
            remote_socket,
            server_addr: settings.server_addr,
            keys: Keys::new(settings),
        })
    }

    async fn send_to(&self, packet: &[u8]) -> io::Result<()> {
        self.remote_socket
            .send_to(packet, self.server_addr)
            .await
            .map(drop)
    }
}

#[async_trait]
impl ObfuscatedTransport for Lwo {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        match &self.keys {
            Keys::V1 { tx_key, .. } => {
                obfuscate(&mut rand::rng(), packet, tx_key.as_bytes());
                self.send_to(packet).await
            }
            Keys::V2 { key } => {
                let mut padded = v2::pad(packet);
                let packet = padded.as_deref_mut().unwrap_or(packet);
                v2::obfuscate(packet, key.as_bytes());
                self.send_to(packet).await
            }
        }
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let (n, from) = self.remote_socket.recv_from(buf).await?;
            if from != self.server_addr {
                continue;
            }

            match &self.keys {
                Keys::V1 { rx_key, .. } => {
                    deobfuscate(&mut buf[..n], rx_key.as_bytes());
                    return Ok(n);
                }
                Keys::V2 { key } => match v2::deobfuscate(&mut buf[..n], key.as_bytes()) {
                    v2::Verdict::Plain => return Ok(n),
                    v2::Verdict::Lwo { trim_to } => return Ok(trim_to.unwrap_or(n)),
                    v2::Verdict::Invalid => {
                        if cfg!(debug_assertions) {
                            log::trace!("Dropping invalid LWO packet");
                        }
                        continue;
                    }
                },
            }
        }
    }

    fn endpoint(&self) -> SocketAddr {
        self.server_addr
    }

    fn packet_overhead(&self) -> u16 {
        0
    }
}

// WG message types, copied from gotatun
type MessageType = u8;
const HANDSHAKE_INIT: MessageType = 1;
const HANDSHAKE_RESP: MessageType = 2;
const COOKIE_REPLY: MessageType = 3;
const DATA: MessageType = 4;

const HANDSHAKE_INIT_SZ: usize = 148;
const HANDSHAKE_RESP_SZ: usize = 92;
const COOKIE_REPLY_SZ: usize = 64;
const DATA_OVERHEAD_SZ: usize = 32;

/// Bit to set in the second byte of the WG header to enable LWO
const OBFUSCATION_BIT: u8 = 0b10000000;

pub fn obfuscate(rng: &mut impl RngCore, packet: &mut [u8], key: &[u8; 32]) {
    let Some(header_bytes) = header_mut(packet, 0) else {
        return;
    };

    xor_bytes(header_bytes, key);

    // randomize byte and set MSB
    rng.fill_bytes(&mut header_bytes[1..2]);
    header_bytes[1] |= OBFUSCATION_BIT;
}

pub fn deobfuscate(packet: &mut [u8], key: &[u8; 32]) {
    let Some(header_bytes) = header_mut(packet, key[0]) else {
        return;
    };
    #[cfg(debug_assertions)]
    if !is_obfuscated(header_bytes[1]) {
        log::error!("Received non-obfuscated packet from relay");
        return;
    }

    xor_bytes(header_bytes, key);

    header_bytes[1] = 0;
}

#[cfg(debug_assertions)]
const fn is_obfuscated(reserved_byte: u8) -> bool {
    reserved_byte & OBFUSCATION_BIT != 0
}

fn header_mut(packet: &mut [u8], key_byte: u8) -> Option<&mut [u8]> {
    let &header_type = packet.first()?;
    match header_type ^ key_byte {
        HANDSHAKE_INIT => packet.get_mut(..HANDSHAKE_INIT_SZ),
        HANDSHAKE_RESP => packet.get_mut(..HANDSHAKE_RESP_SZ),
        COOKIE_REPLY => packet.get_mut(..COOKIE_REPLY_SZ),
        DATA => packet.get_mut(..DATA_OVERHEAD_SZ),
        _ => None,
    }
}

fn xor_bytes(data: &mut [u8], key: &[u8; 32]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

/// Obfuscate a packet using a thread-local RNG.
///
/// This is a convenience function for callers that do not want to manage their own RNG.
/// Uses a per-thread [`rand::rngs::SmallRng`] initialized lazily on first use.
pub fn obfuscate_thread_local(packet: &mut [u8], key: &[u8; 32]) {
    obfuscate(&mut rand::rng(), packet, key);
}

#[cfg(test)]
mod test {
    use super::*;

    struct FixedByteRng(u8);

    impl RngCore for FixedByteRng {
        fn next_u32(&mut self) -> u32 {
            u32::from(self.0)
        }

        fn next_u64(&mut self) -> u64 {
            u64::from(self.0)
        }

        fn fill_bytes(&mut self, dst: &mut [u8]) {
            dst.fill(self.0);
        }
    }

    fn fake_packet() -> Vec<u8> {
        let mut packet = vec![0u8; DATA_OVERHEAD_SZ + 100];
        packet[0] = DATA;
        rand::rng().fill_bytes(&mut packet[DATA_OVERHEAD_SZ..]);
        packet
    }

    #[test]
    fn test_obfuscation() {
        let key = [0xefu8; 32];
        let mut packet = fake_packet();
        let original_packet = packet.clone();

        let mut rng = &mut rand::rng();

        obfuscate(&mut rng, &mut packet, &key);
        assert_ne!(packet, original_packet);
        assert_eq!(
            packet[DATA_OVERHEAD_SZ..],
            original_packet[DATA_OVERHEAD_SZ..],
            "payload should be unchanged"
        );

        deobfuscate(&mut packet, &key);
        assert_eq!(packet, original_packet);
    }

    #[test]
    fn test_obfuscation_reserved_byte_uses_full_random_byte() {
        let key = [0xefu8; 32];
        let mut packet = fake_packet();

        obfuscate(&mut FixedByteRng(u8::MAX), &mut packet, &key);

        assert_eq!(packet[1], u8::MAX);
    }

    #[tokio::test]
    async fn test_e2e_obfuscation_v1() {
        let wg_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let endpoint = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let client_public_key =
            PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
        let server_public_key =
            PublicKey::from_base64("4EkA4c160oQgN/YaNR9GN3gLMevXEfx5hnlc9jYmw14=").unwrap();

        let settings = Settings {
            server_addr: endpoint.local_addr().unwrap(),
            client_public_key: client_public_key.clone(),
            server_public_key: server_public_key.clone(),
            version: LwoVersion::V1,
        };

        let lwo = crate::create_local_socket_obfuscator(&crate::Settings::Lwo(settings))
            .await
            .unwrap();
        let client_socket_addr = lwo.endpoint();

        tokio::spawn(lwo.run());

        let mut rng = &mut rand::rng();

        // Send a test message, verify it on the server
        let packet = fake_packet();

        wg_socket
            .send_to(&packet, client_socket_addr)
            .await
            .unwrap();

        let mut buf = vec![0u8; 1500];
        let (n, addr) = endpoint.recv_from(&mut buf).await.unwrap();
        deobfuscate(&mut buf, server_public_key.as_bytes());
        assert_eq!(&buf[..n], packet);

        // Send a message to the client, verify it
        let packet = fake_packet();

        let mut obfuscated_packet = packet.clone();
        obfuscate(
            &mut rng,
            &mut obfuscated_packet,
            client_public_key.as_bytes(),
        );

        endpoint.send_to(&obfuscated_packet, addr).await.unwrap();

        let (n, _addr) = wg_socket.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &packet);
    }

    /// Test handshake padding trimming.
    #[tokio::test]
    async fn test_e2e_obfuscation_v2_handshake() {
        let wg_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let endpoint = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let client_public_key =
            PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
        let server_public_key =
            PublicKey::from_base64("4EkA4c160oQgN/YaNR9GN3gLMevXEfx5hnlc9jYmw14=").unwrap();
        let key = server_public_key.as_bytes();

        let settings = Settings {
            server_addr: endpoint.local_addr().unwrap(),
            client_public_key,
            server_public_key: server_public_key.clone(),
            version: LwoVersion::V2,
        };

        let lwo = crate::create_local_socket_obfuscator(&crate::Settings::Lwo(settings))
            .await
            .unwrap();
        let client_socket_addr = lwo.endpoint();

        tokio::spawn(lwo.run());

        // Send a handshake initiation, verify it arrives padded and deobfuscates to the original
        let mut packet = vec![0u8; HANDSHAKE_INIT_SZ];
        packet[0] = HANDSHAKE_INIT;
        rand::rng().fill_bytes(&mut packet[4..]);

        wg_socket
            .send_to(&packet, client_socket_addr)
            .await
            .unwrap();

        let mut buf = vec![0u8; 1500];
        let (n, addr) = endpoint.recv_from(&mut buf).await.unwrap();
        assert!(n > HANDSHAKE_INIT_SZ, "handshake should have been padded");
        assert_eq!(
            v2::deobfuscate(&mut buf[..n], key),
            v2::Verdict::Lwo {
                trim_to: Some(HANDSHAKE_INIT_SZ)
            }
        );
        assert_eq!(&buf[..HANDSHAKE_INIT_SZ], packet);

        // Send a padded handshake response back, verify the client trims it
        let mut response = vec![0u8; HANDSHAKE_RESP_SZ];
        response[0] = HANDSHAKE_RESP;
        rand::rng().fill_bytes(&mut response[4..]);

        let mut obfuscated = v2::pad(&response).expect("a handshake is padded");
        v2::obfuscate(&mut obfuscated, key);

        endpoint.send_to(&obfuscated, addr).await.unwrap();

        let (n, _addr) = wg_socket.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &response, "padding should have been trimmed");
    }
}
