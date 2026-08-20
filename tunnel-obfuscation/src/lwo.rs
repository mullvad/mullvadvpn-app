//! LWO (Lightweight WireGuard Obfuscation)

use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use rand::RngCore;
use talpid_net::bypass::{BypassSocket, SocketBypass};
use talpid_types::net::wireguard::PublicKey;
use tokio::net::UdpSocket;

use crate::{
    socket::create_remote_socket,
    transport::ObfuscatedTransport,
    wireguard::{
        COOKIE_REPLY, COOKIE_REPLY_SIZE, DATA, DATA_OVERHEAD_SIZE, HANDSHAKE_INITIATION,
        HANDSHAKE_INITIATION_SIZE, HANDSHAKE_RESPONSE, HANDSHAKE_RESPONSE_SIZE,
    },
};

#[derive(Debug, Clone)]
pub struct Settings {
    /// Remote LWO/WG server
    pub server_addr: SocketAddr,
    /// Public key of the WG client
    pub client_public_key: PublicKey,
    /// Public key of the WG server
    pub server_public_key: PublicKey,
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
    /// Key that the server obfuscates incoming packets with.
    rx_key: PublicKey,
    /// Key to obfuscate outgoing packets with.
    tx_key: PublicKey,
}

impl Lwo {
    pub async fn new(bypass: Arc<dyn SocketBypass>, settings: &Settings) -> crate::Result<Self> {
        let remote_socket = create_remote_socket(&bypass, settings.server_addr.is_ipv4()).await?;

        Ok(Self {
            remote_socket,
            server_addr: settings.server_addr,
            rx_key: settings.client_public_key.clone(),
            tx_key: settings.server_public_key.clone(),
        })
    }
}

#[async_trait]
impl ObfuscatedTransport for Lwo {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        obfuscate(&mut rand::rng(), packet, self.tx_key.as_bytes());
        self.remote_socket
            .send_to(packet, self.server_addr)
            .await
            .map(drop)
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let (n, from) = self.remote_socket.recv_from(buf).await?;
            if from != self.server_addr {
                continue;
            }

            deobfuscate(&mut buf[..n], self.rx_key.as_bytes());
            return Ok(n);
        }
    }

    fn endpoint(&self) -> SocketAddr {
        self.server_addr
    }

    fn packet_overhead(&self) -> u16 {
        0
    }
}

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
        HANDSHAKE_INITIATION => packet.get_mut(..HANDSHAKE_INITIATION_SIZE),
        HANDSHAKE_RESPONSE => packet.get_mut(..HANDSHAKE_RESPONSE_SIZE),
        COOKIE_REPLY => packet.get_mut(..COOKIE_REPLY_SIZE),
        DATA => packet.get_mut(..DATA_OVERHEAD_SIZE),
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
        let mut packet = vec![0u8; DATA_OVERHEAD_SIZE + 100];
        packet[0] = DATA;
        rand::rng().fill_bytes(&mut packet[DATA_OVERHEAD_SIZE..]);
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
            packet[DATA_OVERHEAD_SIZE..],
            original_packet[DATA_OVERHEAD_SIZE..],
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
    async fn test_e2e_obfuscation() {
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
}
