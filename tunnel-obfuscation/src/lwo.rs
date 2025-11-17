//! LWO (Lightweight WireGuard Obfuscation)

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use rand::{RngCore, SeedableRng};
use talpid_types::net::wireguard::PublicKey;
use tokio::{io, net::UdpSocket, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{Obfuscator, socket::create_remote_socket};

const MAX_UDP_SIZE: usize = u16::MAX as usize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to bind local UDP socket
    #[error("Failed to bind client UDP socket")]
    BindUdp(#[source] io::Error),
    /// Failed to connect remote UDP socket
    #[error("Failed to connect remote UDP socket")]
    ConnectRemoteUdp(#[source] io::Error),
    /// Missing UDP listener address
    #[error("Failed to retrieve UDP socket bind address")]
    GetUdpLocalAddress(#[source] io::Error),
    /// Failed to get client sender address
    #[error("Failed to retrieve client sender")]
    PeekUdpSender(#[source] io::Error),
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// Remote LWO/WG server
    pub server_addr: SocketAddr,
    /// Public key of the WG client
    pub client_public_key: PublicKey,
    /// Public key of the WG server
    pub server_public_key: PublicKey,
    /// Optional fwmark to set on the remote socket
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

pub struct Lwo {
    client: Client,
    local_endpoint: SocketAddr,
    #[cfg(target_os = "android")]
    wg_endpoint: Arc<UdpSocket>,
}

impl Lwo {
    pub async fn new(settings: &Settings) -> crate::Result<Self> {
        let remote_socket = Arc::new(
            create_remote_socket(
                settings.server_addr.is_ipv4(),
                #[cfg(target_os = "linux")]
                settings.fwmark,
            )
            .await?,
        );
        let client_socket = Arc::new(
            UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
                .await
                .map_err(Error::BindUdp)
                .map_err(crate::Error::CreateLwoObfuscator)?,
        );
        let local_endpoint = client_socket
            .local_addr()
            .map_err(Error::GetUdpLocalAddress)
            .map_err(crate::Error::CreateLwoObfuscator)?;

        #[cfg(target_os = "android")]
        let wg_endpoint = Arc::clone(&remote_socket);

        let client = Client {
            server_addr: settings.server_addr,
            rx_key: settings.client_public_key.clone(),
            tx_key: settings.server_public_key.clone(),
            remote_socket,
            client_socket,
        };

        Ok(Self {
            local_endpoint,
            client,
            #[cfg(target_os = "android")]
            wg_endpoint,
        })
    }

    async fn run_forwarding(client: Client, cancel_token: CancellationToken) -> Result<(), Error> {
        let mut running = client.connect().await?;
        log::trace!("LWO client is running! 🎉");
        tokio::select! {
            _ = cancel_token.cancelled() => log::trace!("Stopping LWO obfuscation"),
            _result = &mut running.send => log::trace!("LWO client closed (send_task)"),
            _result = &mut running.recv => log::trace!("LWO client closed (recv_task)"),
        };

        Ok(())
    }
}

struct Client {
    server_addr: SocketAddr,

    rx_key: PublicKey,
    tx_key: PublicKey,

    remote_socket: Arc<UdpSocket>,
    client_socket: Arc<UdpSocket>,
}

/// Start an LWO client by calling [Client::run].
struct RunningClient {
    /// Egress task.
    send: JoinHandle<()>,
    /// Ingress task.
    recv: JoinHandle<()>,
}

// Auto-abort the send/recv tasks on drop.
impl Drop for RunningClient {
    fn drop(&mut self) {
        self.send.abort();
        self.recv.abort();
    }
}

impl Client {
    /// Returns join handles to the send and receive tasks. These need to be aborted when the
    /// obfuscator is aborted / finished.
    async fn connect(self) -> Result<RunningClient, Error> {
        let Client {
            server_addr,
            rx_key,
            tx_key,
            remote_socket,
            client_socket,
        } = self;

        remote_socket
            .connect(server_addr)
            .await
            .map_err(Error::ConnectRemoteUdp)?;
        log::debug!("Connected to {server_addr}");

        let client_addr = client_socket
            .peek_sender()
            .await
            .map_err(Error::GetUdpLocalAddress)?;
        client_socket
            .connect(client_addr)
            .await
            .map_err(Error::PeekUdpSender)?;
        log::debug!("Client socket connected to {client_addr}");

        let rx_socket = client_socket.clone();
        let tx_socket = remote_socket.clone();
        let send_task = tokio::spawn(async move {
            run_obfuscation(true, tx_key, rx_socket, tx_socket).await;
        });

        let rx_socket = remote_socket.clone();
        let tx_socket = client_socket.clone();
        let recv_task = tokio::spawn(async move {
            run_obfuscation(false, rx_key, rx_socket, tx_socket).await;
        });

        Ok(RunningClient {
            send: send_task,
            recv: recv_task,
        })
    }
}

async fn run_obfuscation(
    sending: bool,
    key: PublicKey,
    read_socket: Arc<UdpSocket>,
    write_socket: Arc<UdpSocket>,
) {
    if sending {
        let mut rng = new_rng();
        run_obfuscation_inner(
            move |buf| obfuscate(&mut rng, buf, key.as_bytes()),
            read_socket,
            write_socket,
        )
        .await
    } else {
        run_obfuscation_inner(
            move |buf| deobfuscate(buf, key.as_bytes()),
            read_socket,
            write_socket,
        )
        .await
    }
}

async fn run_obfuscation_inner(
    mut action: impl FnMut(&mut [u8]),
    read_socket: Arc<UdpSocket>,
    write_socket: Arc<UdpSocket>,
) {
    let mut buf = vec![0u8; MAX_UDP_SIZE];

    loop {
        let read_n = match read_socket.recv(&mut buf).await {
            Ok(read_n) => read_n,
            Err(err) => {
                log::debug!("read_socket.recv failed: {err}");
                return;
            }
        };

        // TODO: recv and send concurrently
        action(&mut buf[..read_n]);

        if let Err(err) = write_socket.send(&buf[..read_n]).await {
            log::debug!("write_socket.send_to failed: {err}");
            return;
        }
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
    let rand_byte = (rng.next_u32() % u8::MAX as u32) as u8;
    header_bytes[1] = rand_byte | OBFUSCATION_BIT;
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

#[async_trait]
impl Obfuscator for Lwo {
    fn endpoint(&self) -> SocketAddr {
        self.local_endpoint
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        let token = CancellationToken::new();
        let child_token = token.child_token();
        // This will always cancel `child_token` as soon as `run` is finished or aborted.
        let _drop_guard = token.drop_guard();

        let client = self.client;
        tokio::spawn(Lwo::run_forwarding(client, child_token))
            .await
            .unwrap()
            .map_err(crate::Error::RunLwoObfuscator)
    }

    fn packet_overhead(&self) -> u16 {
        0
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        use std::os::fd::AsRawFd;
        self.wg_endpoint.as_raw_fd()
    }
}

pub fn new_rng() -> impl RngCore {
    rand::rngs::SmallRng::from_entropy()
}

#[cfg(test)]
mod test {
    use super::*;

    fn fake_packet() -> Vec<u8> {
        let mut packet = vec![0u8; DATA_OVERHEAD_SZ + 100];
        packet[0] = DATA;
        rand::thread_rng().fill_bytes(&mut packet[DATA_OVERHEAD_SZ..]);
        packet
    }

    #[test]
    fn test_obfuscation() {
        let key = [0xefu8; 32];
        let mut packet = fake_packet();
        let original_packet = packet.clone();

        let mut rng = new_rng();

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
            #[cfg(target_os = "linux")]
            fwmark: None,
        };

        let lwo = Lwo::new(&settings).await.unwrap();
        let client_socket_addr = lwo.local_endpoint;

        tokio::spawn(Box::new(lwo).run());

        let mut rng = new_rng();

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
