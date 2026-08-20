//! # Multiplexer Obfuscation
//!
//! This obfuscation module attempts to establish a connection through multiple obfuscation methods
//! simultaneously. It acts as a UDP proxy that forwards WireGuard traffic through other
//! obfuscation transports (UDP2TCP, Shadowsocks, QUIC, etc.)
//! and automatically selects the first one that successfully establishes a connection.
//!
//! ## How it works
//!
//! 1. **Initial Setup**: The multiplexer creates a local UDP socket that WireGuard connects to
//! 2. **Transport Spawning**: It progressively spawns different obfuscation transports at timed
//!    intervals
//! 3. **Traffic Fanout**: All incoming WireGuard packets are fanned out to all active transports
//! 4. **First Response Wins**: The first transport to receive a response from the server is
//!    selected
//! 5. **Connection Establishment**: Once a transport is selected, the multiplexer switches to a
//!    direct forwarding mode between WireGuard and the selected transport
//!
//! ## Transport Types
//!
//! See the [Transport] enum.

use std::{
    collections::VecDeque,
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesUnordered};
use talpid_net::bypass::SocketBypass;
use talpid_types::net::wireguard::PublicKey;
use tokio::{net::UdpSocket, sync::oneshot};
use tokio_util::task::AbortOnDropHandle;

use crate::{
    direct::Direct,
    transport::{MAX_DATAGRAM_SIZE, ObfuscatedTransport},
    wireguard::{HandshakeFilter, Received},
};

/// How long to wait before spawning the next transport in the queue.
const SPAWN_INTERVAL: Duration = Duration::from_secs(1);

/// Max number of initial outgoing packets to buffer for replaying to new transports
const MAX_INITIAL_PACKETS: usize = 100;

/// Index of a transport in the set of running transports.
type TransportId = usize;

/// An obfuscator that manages multiple other obfuscators and automatically
/// selects the first one that successfully establishes a connection.
///
/// The multiplexer operates in two phases:
/// 1. **Discovery Phase**: Spawn transports progressively and fan out traffic to all of them
/// 2. **Connected Phase**: Once a transport responds with a valid handshake response, switch to
///    forwarding to that transport only
pub struct Multiplexer {
    /// Local UDP socket that WireGuard connects to
    client_socket: UdpSocket,
    /// Address of the client socket that WireGuard should connect to
    client_socket_addr: SocketAddr,
    /// Queue of transports to spawn (in priority order)
    pending: VecDeque<Transport>,
    /// Public key of the local WireGuard instance.
    client_public_key: PublicKey,
    /// Notified with the selected transport
    selected_transport_tx: SelectedTransportTx,
    bypass: Arc<dyn SocketBypass>,
}

/// A transport that has been spawned and is being fanned out to.
struct RunningTransport {
    /// The configuration this transport was spawned from. Announced if it is selected.
    config: Transport,
    transport: Arc<dyn ObfuscatedTransport>,
    /// Buffer that this transport deobfuscates received packets into.
    buf: Box<[u8]>,
}

impl Multiplexer {
    /// Create a new multiplexer with the specified transports (obfuscators) and settings.
    ///
    /// # Arguments
    /// * `settings` - Configuration containing the list of transports to try and network settings
    ///
    /// # Returns
    /// A new multiplexer instance ready to start obfuscation discovery
    pub async fn new(bypass: Arc<dyn SocketBypass>, settings: Settings) -> crate::Result<Self> {
        let client_socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(crate::Error::BindLocalUdp)?;

        let client_socket_addr = client_socket
            .local_addr()
            .map_err(crate::Error::BindLocalUdp)?;

        Ok(Self {
            client_socket,
            client_socket_addr,
            pending: VecDeque::from(settings.transports),
            client_public_key: settings.client_public_key,
            selected_transport_tx: settings.selected_transport,
            bypass,
        })
    }

    /// Run the multiplexer: discover a working transport, then forward traffic over it.
    ///
    /// Blocks until WireGuard sends its first packet, and then until the connection fails.
    async fn start(self) -> io::Result<()> {
        log::debug!("Running multiplexer obfuscation");

        let Self {
            client_socket,
            pending,
            client_public_key,
            selected_transport_tx,
            bypass,
            ..
        } = self;

        // Wait for WireGuard's first packet so that we know where to send replies.
        let wg_addr = client_socket.peek_sender().await?;
        client_socket.connect(wg_addr).await?;
        log::debug!("Local WireGuard instance connected from {wg_addr}");

        let discovery = run_discovery(
            &client_socket,
            pending,
            bypass,
            &client_public_key,
            selected_transport_tx,
        );
        let Some(transport) = discovery.await? else {
            return Ok(());
        };

        run_connected(client_socket, transport).await
    }
}

/// Fan traffic out to all transports until one of them responds.
///
/// Run the main event loop:
/// 1. Receive packets from WireGuard and fan them out to all active transports
/// 2. Receive responses from the transports
/// 3. Spawn new transports at timed intervals
///
/// Returns the selected transport, or `None` if the local WireGuard instance went away before any
/// transport was selected.
async fn run_discovery(
    client_socket: &UdpSocket,
    mut pending: VecDeque<Transport>,
    bypass: Arc<dyn SocketBypass>,
    client_public_key: &PublicKey,
    selected_transport_tx: SelectedTransportTx,
) -> io::Result<Option<Arc<dyn ObfuscatedTransport>>> {
    enum Event {
        /// A packet from the local WireGuard instance, or the error that prevented it.
        Wireguard(io::Result<usize>),
        /// A packet from a running transport, deobfuscated into that transport's buffer.
        Transport(TransportId, io::Result<usize>),
        /// Time to spawn the next pending transport.
        Spawn,
    }

    let mut running: Vec<RunningTransport> = vec![];
    let mut initial_packets: Vec<Box<[u8]>> = vec![];
    let mut handshakes = HandshakeFilter::new(client_public_key);

    let mut wg_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut scratch = vec![0u8; MAX_DATAGRAM_SIZE];

    let mut spawn_timer = tokio::time::interval(SPAWN_INTERVAL);

    loop {
        let event = tokio::select! {
            // From local WG
            result = client_socket.recv(&mut wg_buf) => Event::Wireguard(result),

            // From any running transport
            (id, result) = recv_any(&mut running) => Event::Transport(id, result),

            // Spawning the next transport
            _ = spawn_timer.tick() => Event::Spawn,
        };

        match event {
            Event::Wireguard(Err(err)) => {
                log::error!("Failed to receive traffic from local WireGuard instance: {err}");
                return Ok(None);
            }
            Event::Wireguard(Ok(n)) => {
                let packet = &wg_buf[..n];

                if initial_packets.len() >= MAX_INITIAL_PACKETS {
                    // Initial packets should be handshake initiation packets, so we
                    // should not end up here if there's some reasonable timeout.
                    // If we do, fail so we don't use excessive memory.
                    return Err(io::Error::other("Too many initial packets"));
                }
                initial_packets.push(Box::from(packet));
                handshakes.record_initiation(packet);

                // Fan out the latest WG packet to all currently spawned transports.
                for (id, running) in running.iter().enumerate() {
                    scratch[..n].copy_from_slice(packet);
                    if let Err(err) = running.transport.send(&mut scratch[..n]).await {
                        log::error!("Failed to send packet to transport {id}: {err}");
                    }
                }
            }

            Event::Transport(id, Err(err)) => {
                // A single failing transport is not fatal; the others may still connect.
                log::error!("Dropping transport {id}, which failed to receive traffic: {err}");
                running.remove(id);
            }
            Event::Transport(id, Ok(n)) => match handshakes.classify(&running[id].buf[..n]) {
                // Not an answer to anything WireGuard sent. Whatever it is, it is no reason to
                // believe this transport reaches the relay.
                Received::Unrecognized => {
                    log::debug!("Ignoring unsolicited packet from transport {id}");
                }

                // The relay is rate limiting us. WireGuard needs the cookie in order to retry the
                // initiation with a valid `mac2`, but reaching a rate limiter is not proof that
                // this transport carries a handshake, so keep fanning out.
                Received::CookieReply => {
                    log::debug!("Forwarding cookie reply from transport {id}");
                    client_socket.send(&running[id].buf[..n]).await?;
                }

                // A handshake response means this transport reached the relay, so use it from
                // now on.
                Received::HandshakeResponse => {
                    let selected = running.swap_remove(id);
                    log::debug!(
                        "Selecting {:?} as the valid transport configuration",
                        selected.config
                    );
                    client_socket.send(&selected.buf[..n]).await?;

                    // Announce the selected transport, so that the firewall can be restricted to
                    // the endpoint it committed to.
                    let _ = selected_transport_tx.send(selected.config);

                    return Ok(Some(selected.transport));
                }
            },

            Event::Spawn => {
                let Some(config) = pending.pop_front() else {
                    continue;
                };
                match spawn_transport(&bypass, config.clone()).await {
                    Ok(transport) => {
                        send_initial_packets(&transport, &initial_packets, &mut scratch).await;
                        running.push(RunningTransport {
                            config,
                            transport,
                            buf: vec![0u8; MAX_DATAGRAM_SIZE].into_boxed_slice(),
                        });
                    }
                    Err(err) => log::error!("Failed to spawn new transport: {err}"),
                }
            }
        }
    }
}

/// Receive from whichever transport produces a packet first, into that transport's own buffer.
///
/// Never resolves if there are no running transports. Cancelling this drops every in-flight
/// `recv`, which is why [ObfuscatedTransport] requires them to be cancel safe.
async fn recv_any(transports: &mut [RunningTransport]) -> (TransportId, io::Result<usize>) {
    let mut recvs: FuturesUnordered<_> = transports
        .iter_mut()
        .enumerate()
        .map(|(id, running)| async move { (id, running.transport.recv(&mut running.buf).await) })
        .collect();

    match recvs.next().await {
        Some(result) => result,
        None => std::future::pending().await,
    }
}

/// Create the [ObfuscatedTransport] for `transport`.
async fn spawn_transport(
    bypass: &Arc<dyn SocketBypass>,
    transport: Transport,
) -> crate::Result<Arc<dyn ObfuscatedTransport>> {
    match transport {
        Transport::Direct(addr) => {
            log::info!("Spawning direct forwarder");
            Ok(Arc::new(Direct::new(bypass, addr).await?))
        }
        Transport::Obfuscated(settings) => {
            log::info!("Spawning new obfuscator");
            crate::create_transport(Arc::clone(bypass), &settings).await
        }
    }
}

/// Replay the packets WireGuard sent before `transport` existed.
async fn send_initial_packets(
    transport: &Arc<dyn ObfuscatedTransport>,
    packets: &[Box<[u8]>],
    scratch: &mut [u8],
) {
    for packet in packets {
        let n = packet.len();
        scratch[..n].copy_from_slice(packet);
        if let Err(err) = transport.send(&mut scratch[..n]).await {
            log::error!("Failed to forward packet to new transport: {err}");
        }
    }
}

/// Switch to connected mode after a transport has been successfully selected.
///
/// In this mode, the multiplexer is a plain proxy between WireGuard and the selected transport.
/// Since it is the only transport left, packets are obfuscated in place with no copies.
///
/// Blocks until either direction fails.
async fn run_connected(
    client_socket: UdpSocket,
    transport: Arc<dyn ObfuscatedTransport>,
) -> io::Result<()> {
    let client_socket = Arc::new(client_socket);

    let mut tx_task = AbortOnDropHandle::new(tokio::spawn({
        let (client_socket, transport) = (Arc::clone(&client_socket), Arc::clone(&transport));
        async move {
            let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
            loop {
                let n = client_socket.recv(&mut buf).await?;
                transport.send(&mut buf[..n]).await?;
            }
        }
    }));

    let mut rx_task = AbortOnDropHandle::new(tokio::spawn(async move {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
        loop {
            let n = transport.recv(&mut buf).await?;
            client_socket.send(&buf[..n]).await?;
        }
    }));

    tokio::select! {
        Ok(result) = &mut tx_task => result,
        Ok(result) = &mut rx_task => result,
        else => Ok(()),
    }
}

/// Notifies interested parties about which transport the multiplexer has committed to.
pub type SelectedTransportTx = oneshot::Sender<Transport>;

/// Configuration settings for multiplexer obfuscation
#[derive(Debug)]
pub struct Settings {
    /// List of transports to try, ordered by priority (highest to lowest).
    /// Spawn these transports progressively and select
    /// the first one that successfully establishes a connection.
    pub transports: Vec<Transport>,
    /// Public key of the local WireGuard instance.
    pub client_public_key: PublicKey,
    /// Notified with the selected transport, once one has been selected. The value is only ever
    /// set once, as the multiplexer never reconsiders its choice.
    pub selected_transport: SelectedTransportTx,
}

/// Represents a transport method that the multiplexer can use.
#[derive(Clone, Debug)]
pub enum Transport {
    /// Direct UDP forwarding without any obfuscation
    Direct(SocketAddr),
    /// An obfuscated transport (UDP2TCP, Shadowsocks, QUIC, etc.)
    Obfuscated(crate::Settings),
}

impl Transport {
    /// The overhead (in bytes) that this transport adds to every packet.
    pub fn packet_overhead(&self) -> u16 {
        match self {
            Transport::Direct(_) => 0,
            Transport::Obfuscated(settings) => settings.packet_overhead(),
        }
    }
}

/// The largest overhead among `transports`.
///
/// Which of them is selected is not known until one of them answers, and the MTU has to be
/// decided before that, so every transport has to fit within it.
pub fn packet_overhead(transports: &[Transport]) -> u16 {
    transports
        .iter()
        .map(Transport::packet_overhead)
        .max()
        .unwrap_or(0)
}

#[async_trait]
impl crate::LocalSocketObfuscator for Multiplexer {
    fn endpoint(&self) -> SocketAddr {
        self.client_socket_addr
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.start()
            .await
            .map_err(crate::Error::RunMultiplexerObfuscator)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::{
        LocalSocketObfuscator,
        wireguard::{handshake_initiation, handshake_response},
    };
    use talpid_net::bypass::NoopBypass;
    use talpid_types::net::obfuscation::LwoVersion;

    /// The index that the handshakes in these tests are for.
    const SESSION: u32 = 7;

    fn client_key() -> PublicKey {
        PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap()
    }

    fn server_key() -> PublicKey {
        PublicKey::from_base64("4EkA4c160oQgN/YaNR9GN3gLMevXEfx5hnlc9jYmw14=").unwrap()
    }

    /// Test whether the multiplexer works with direct transports
    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_direct_forwarding() {
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let server_socket2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr2 = server_socket2.local_addr().unwrap();

        // Create multiplexer pointing to direct transports
        let (selected_tx, selected_rx) = oneshot::channel();
        let settings = Settings {
            transports: vec![
                Transport::Direct(server_addr),
                Transport::Direct(server_addr2),
            ],
            client_public_key: client_key(),
            selected_transport: selected_tx,
        };

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings)
            .await
            .unwrap();
        let multiplexer_endpoint = multiplexer.endpoint();

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        tokio::spawn(async move {
            let boxed_multiplexer = Box::new(multiplexer);
            boxed_multiplexer.run().await
        });

        // Send a handshake initiation from client to multiplexer and verify that it is received
        let test_data = handshake_initiation(SESSION);
        client_socket
            .send_to(&test_data, multiplexer_endpoint)
            .await
            .unwrap();

        let mut server_buf = vec![0u8; 1024];
        let (bytes_received, client_addr) = server_socket.recv_from(&mut server_buf).await.unwrap();

        assert_eq!(&server_buf[..bytes_received], test_data);

        // Our second socket should also receive this packet
        let (bytes_received, second_server_client_addr) =
            server_socket2.recv_from(&mut server_buf).await.unwrap();
        assert_eq!(&server_buf[..bytes_received], test_data);

        // A packet that answers nothing must neither select a transport nor be forwarded. If it
        // were forwarded, the client would read it below instead of the handshake response.
        server_socket.send_to(b"Pong!", client_addr).await.unwrap();
        tokio::task::yield_now().await;

        // Send a handshake response back from the first server
        let response_data = handshake_response(SESSION, &client_key());
        server_socket
            .send_to(&response_data, client_addr)
            .await
            .unwrap();

        // Verify that response was forwarded
        let mut client_buf = vec![0u8; 1024];
        let (bytes_received, _) = client_socket.recv_from(&mut client_buf).await.unwrap();

        assert_eq!(&client_buf[..bytes_received], response_data);

        // The first server, and not the second, should have been announced as selected
        let selected = selected_rx.await.unwrap();
        assert!(matches!(selected, Transport::Direct(addr) if addr == server_addr));

        // Packets from unselected transports should not be forwarded after the
        // multiplexer has picked a transport.
        let unexpected_data = b"Wrong server";
        server_socket2
            .send_to(unexpected_data, second_server_client_addr)
            .await
            .unwrap();
        tokio::task::yield_now().await;

        let selected_data = b"Selected server";
        server_socket
            .send_to(selected_data, client_addr)
            .await
            .unwrap();

        let (bytes_received, _) = client_socket.recv_from(&mut client_buf).await.unwrap();
        assert_eq!(&client_buf[..bytes_received], selected_data);

        // Test that packets are now forwarded directly (connected mode)
        let second_test_data = b"Connected!";
        client_socket
            .send_to(second_test_data, multiplexer_endpoint)
            .await
            .unwrap();

        let (bytes_received, _) = server_socket.recv_from(&mut server_buf).await.unwrap();

        assert_eq!(&server_buf[..bytes_received], second_test_data);
    }

    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_ignores_invalid_handshake_responses() {
        let liar = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let liar_addr = liar.local_addr().unwrap();

        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr = relay.local_addr().unwrap();

        let (selected_tx, mut selected_rx) = oneshot::channel();
        let settings = Settings {
            transports: vec![Transport::Direct(liar_addr), Transport::Direct(relay_addr)],
            client_public_key: client_key(),
            selected_transport: selected_tx,
        };

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings)
            .await
            .unwrap();
        let multiplexer_endpoint = multiplexer.endpoint();
        tokio::spawn(Box::new(multiplexer).run());

        let wg_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        wg_socket
            .send_to(&handshake_initiation(SESSION), multiplexer_endpoint)
            .await
            .unwrap();

        let mut buf = vec![0u8; 1024];
        let (_, liar_client_addr) = liar.recv_from(&mut buf).await.unwrap();
        let (_, relay_client_addr) = relay.recv_from(&mut buf).await.unwrap();

        // None of these answers the initiation that was sent.
        let junk = b"Pong!".to_vec();
        let wrong_session = handshake_response(SESSION + 1, &client_key()).to_vec();
        let wrong_key = handshake_response(SESSION, &server_key()).to_vec();
        let mut tampered = handshake_response(SESSION, &client_key()).to_vec();
        tampered[12] ^= 1;

        for packet in [junk, wrong_session, wrong_key, tampered] {
            liar.send_to(&packet, liar_client_addr).await.unwrap();
            // Time only advances once every task is idle, so this drains the multiplexer.
            tokio::time::sleep(Duration::from_millis(50)).await;
            assert_matches!(
                selected_rx.try_recv(),
                Err(oneshot::error::TryRecvError::Empty),
                "a packet that answers no initiation selected a transport",
            );
        }

        // This one does.
        let response = handshake_response(SESSION, &client_key()).to_vec();
        relay.send_to(&response, relay_client_addr).await.unwrap();

        // WireGuard should see the valid response, and nothing the liar sent before it.
        let (n, _) = wg_socket.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &response[..]);

        let selected = selected_rx.await.unwrap();
        assert_matches!(
            selected, Transport::Direct(addr) if addr == relay_addr,
            "the transport that answered the handshake should have been selected, got {selected:?}",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_obfuscated_transport() {
        use crate::lwo;

        let client_key = client_key();
        let server_key = server_key();

        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let (selected_tx, selected_rx) = oneshot::channel();
        let settings = Settings {
            transports: vec![Transport::Obfuscated(crate::Settings::Lwo(lwo::Settings {
                server_addr: server_socket.local_addr().unwrap(),
                client_public_key: client_key.clone(),
                server_public_key: server_key.clone(),
                // TODO: support v2
                version: LwoVersion::V1,
            }))],
            client_public_key: client_key.clone(),
            selected_transport: selected_tx,
        };

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings)
            .await
            .unwrap();
        let multiplexer_endpoint = multiplexer.endpoint();
        tokio::spawn(Box::new(multiplexer).run());

        let wg_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // A WireGuard handshake initiation, which LWO obfuscates the header of
        let packet = handshake_initiation(SESSION).to_vec();

        wg_socket
            .send_to(&packet, multiplexer_endpoint)
            .await
            .unwrap();

        // The server should see the packet obfuscated with the server's key
        let mut server_buf = vec![0u8; 1024];
        let (n, transport_addr) = server_socket.recv_from(&mut server_buf).await.unwrap();
        assert_ne!(&server_buf[..n], &packet[..], "packet was not obfuscated");
        lwo::deobfuscate(&mut server_buf[..n], server_key.as_bytes());
        assert_eq!(&server_buf[..n], &packet[..]);

        // The multiplexer should deobfuscate the response and hand it to WireGuard verbatim
        let response = handshake_response(SESSION, &client_key).to_vec();
        let mut obfuscated_response = response.clone();
        lwo::obfuscate(
            &mut rand::rng(),
            &mut obfuscated_response,
            client_key.as_bytes(),
        );
        server_socket
            .send_to(&obfuscated_response, transport_addr)
            .await
            .unwrap();

        let mut wg_buf = vec![0u8; 1024];
        let (n, _) = wg_socket.recv_from(&mut wg_buf).await.unwrap();
        assert_eq!(&wg_buf[..n], &response[..]);

        // The LWO transport should have been announced as selected
        let selected = selected_rx.await.unwrap();
        assert!(matches!(
            selected,
            Transport::Obfuscated(crate::Settings::Lwo(_))
        ));

        // The transport should now be selected: traffic keeps flowing in connected mode
        let packet = handshake_initiation(SESSION + 1).to_vec();
        wg_socket
            .send_to(&packet, multiplexer_endpoint)
            .await
            .unwrap();

        let (n, _) = server_socket.recv_from(&mut server_buf).await.unwrap();
        lwo::deobfuscate(&mut server_buf[..n], server_key.as_bytes());
        assert_eq!(&server_buf[..n], &packet[..]);
    }
}
