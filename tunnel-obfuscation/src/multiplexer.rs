//! # Multiplexer Obfuscation
//!
//! This obfuscation module attempts to establish a connection through multiple obfuscation methods
//! simultaneously. It acts as a UDP proxy that forwards WireGuard traffic through other
//! obfuscation transports (UDP2TCP, Shadowsocks, QUIC, etc.)
//! and automatically selects the first one that successfully establishes a connection.
//!
//! ## How it works
//!
//! 1. **Transport Spawning**: It progressively spawns different obfuscation transports at timed
//!    intervals
//! 2. **Traffic Fanout**: All outgoing WireGuard packets are fanned out to all active transports
//! 3. **First Response Wins**: The first transport to receive a response from the server is
//!    selected
//! 4. **Connection Establishment**: Once a transport is selected, the multiplexer steps out of the
//!    data path and hands every datagram straight to it
//!
//! The multiplexer can be wrapped in a [crate::local_socket::LocalSocketRunner] if it needs to be
//! reached through a local UDP proxy.
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
use tokio::sync::{Mutex, mpsc, oneshot, watch};
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

/// Datagram queue len for send/recv during the discovery phase.
const DATAGRAM_QUEUE_LEN: usize = 128;

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
    /// Which phase the multiplexer is in.
    phase: watch::Receiver<Phase>,

    /// Datagrams to fan out, drained by the discovery task while it is still racing.
    outgoing: mpsc::Sender<Box<[u8]>>,

    /// Datagrams that a raced transport received before one was selected.
    ///
    /// The [Mutex] is uncontended in practice.
    incoming: Mutex<mpsc::Receiver<Box<[u8]>>>,

    /// Reported as the endpoint until discovery settles on a transport that has one of its own.
    fallback_endpoint: SocketAddr,

    packet_overhead: u16,

    /// Stops discovery when this transport is dropped.
    _discovery_task: AbortOnDropHandle<()>,
}

/// Which phase the multiplexer is in.
#[derive(Clone)]
enum Phase {
    /// Still trying different obfuscation transports.
    Discovery,
    /// This transport answered first, and carries the traffic from here on.
    Connected(Arc<dyn ObfuscatedTransport>),
    /// Discovery ended without selecting a transport.
    Failed(Arc<io::Error>),
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
    /// Discovery starts racing as soon as the first datagram is sent, and the multiplexer steps
    /// aside once it has a winner.
    ///
    /// # Arguments
    /// * `settings` - Configuration containing the list of transports to try and network settings
    pub fn new(bypass: Arc<dyn SocketBypass>, settings: Settings) -> Self {
        let Settings {
            transports,
            client_public_key,
            selected_transport,
        } = settings;

        // Use the largest transport overhead.
        let packet_overhead = packet_overhead(&transports);
        let fallback_endpoint = transports
            .first()
            .map(Transport::endpoint)
            .unwrap_or_else(|| SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)));

        let (outgoing, outgoing_rx) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let (incoming_tx, incoming) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let (phase_tx, phase) = watch::channel(Phase::Discovery);

        let discovery_task = tokio::spawn(discover(
            outgoing_rx,
            incoming_tx,
            phase_tx,
            VecDeque::from(transports),
            bypass,
            client_public_key,
            selected_transport,
        ));

        Self {
            phase,
            outgoing,
            incoming: Mutex::new(incoming),
            fallback_endpoint,
            packet_overhead,
            _discovery_task: AbortOnDropHandle::new(discovery_task),
        }
    }

    /// Which phase the multiplexer is in, as of now.
    fn phase(&self) -> Phase {
        self.phase.borrow().clone()
    }
}

/// Race the transports until one of them answers, and publish the result.
///
/// Whatever happens, `phase_tx` is told about it, since [Multiplexer::send] and
/// [Multiplexer::recv] have nowhere else to learn that the race is over.
async fn discover(
    mut outgoing: mpsc::Receiver<Box<[u8]>>,
    incoming: mpsc::Sender<Box<[u8]>>,
    phase_tx: watch::Sender<Phase>,
    pending: VecDeque<Transport>,
    bypass: Arc<dyn SocketBypass>,
    client_public_key: PublicKey,
    selected_transport_tx: SelectedTransportTx,
) {
    log::debug!("Running multiplexer obfuscation");

    let result = run_discovery(
        &mut outgoing,
        &incoming,
        pending,
        bypass,
        &client_public_key,
        selected_transport_tx,
    )
    .await;

    let _ = phase_tx.send(match result {
        Ok(Some(transport)) => Phase::Connected(transport),
        Ok(None) => Phase::Failed(Arc::new(stopped())),
        Err(err) => Phase::Failed(Arc::new(err)),
    });
}

fn stopped() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "the multiplexer stopped before selecting a transport",
    )
}

/// [io::Error] is not [Clone], and every caller that waited on the same failure needs one.
fn clone_error(error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), error.to_string())
}

fn copy_datagram(datagram: &[u8], buf: &mut [u8]) -> io::Result<usize> {
    buf.get_mut(..datagram.len())
        .ok_or_else(|| io::Error::other("datagram does not fit in the receive buffer"))?
        .copy_from_slice(datagram);
    Ok(datagram.len())
}

#[async_trait]
impl ObfuscatedTransport for Multiplexer {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        match self.phase() {
            // Just forward to the selected transport.
            Phase::Connected(transport) => transport.send(packet).await,
            Phase::Failed(err) => Err(clone_error(&err)),
            Phase::Discovery => self
                .outgoing
                .send(Box::from(&packet[..]))
                .await
                .map_err(|_| stopped()),
        }
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut incoming = self.incoming.lock().await;
        let mut phase = self.phase.clone();

        loop {
            // Drain any packets received during the discovery phase.
            // This will be emptied forever when connected.
            if let Ok(datagram) = incoming.try_recv() {
                return copy_datagram(&datagram, buf);
            }

            let current = phase.borrow_and_update().clone();
            match current {
                // In the connected phase, just forward from the selected transport.
                Phase::Connected(transport) => return transport.recv(buf).await,
                Phase::Failed(err) => return Err(clone_error(&err)),
                // Discovery phase
                Phase::Discovery => {
                    tokio::select! {
                        biased;
                        Some(datagram) = incoming.recv() => return copy_datagram(&datagram, buf),
                        result = phase.changed() => {
                            // Discovery task failed to report a result.
                            if result.is_err() {
                                return Err(stopped());
                            }
                        }
                    }
                }
            }
        }
    }

    fn endpoint(&self) -> SocketAddr {
        match self.state() {
            Discovery::Selected(transport) => transport.endpoint(),
            _ => self.fallback_endpoint,
        }
    }

    fn packet_overhead(&self) -> u16 {
        self.packet_overhead
    }
}

/// Fan traffic out to all transports until one of them responds.
///
/// Run the main event loop:
/// 1. Receive packets from WireGuard and fan them out to all active transports
/// 2. Receive responses from the transports
/// 3. Spawn new transports at timed intervals
///
/// Returns the selected transport, or `None` if the caller went away before any transport was
/// selected.
async fn run_discovery(
    outgoing: &mut mpsc::Receiver<Box<[u8]>>,
    incoming: &mpsc::Sender<Box<[u8]>>,
    mut pending: VecDeque<Transport>,
    bypass: Arc<dyn SocketBypass>,
    client_public_key: &PublicKey,
    selected_transport_tx: SelectedTransportTx,
) -> io::Result<Option<Arc<dyn ObfuscatedTransport>>> {
    enum Event {
        /// A packet from the local WireGuard instance, or `None` if it went away.
        Wireguard(Option<Box<[u8]>>),
        /// A packet from a running transport, deobfuscated into that transport's buffer.
        Transport(TransportId, io::Result<usize>),
        /// Time to spawn the next pending transport.
        Spawn,
    }

    let mut running: Vec<RunningTransport> = vec![];
    let mut initial_packets: Vec<Box<[u8]>> = vec![];
    let mut handshakes = HandshakeFilter::new(client_public_key);

    let mut scratch = vec![0u8; MAX_DATAGRAM_SIZE];

    let mut spawn_timer = tokio::time::interval(SPAWN_INTERVAL);

    loop {
        let event = tokio::select! {
            // From local WG
            packet = outgoing.recv() => Event::Wireguard(packet),

            // From any running transport
            (id, result) = recv_any(&mut running) => Event::Transport(id, result),

            // Spawning the next transport
            _ = spawn_timer.tick() => Event::Spawn,
        };

        match event {
            Event::Wireguard(None) => {
                log::debug!("The local WireGuard instance went away before a transport answered");
                return Ok(None);
            }
            Event::Wireguard(Some(packet)) => {
                let n = packet.len();

                if initial_packets.len() >= MAX_INITIAL_PACKETS {
                    // Initial packets should be handshake initiation packets, so we
                    // should not end up here if there's some reasonable timeout.
                    // If we do, fail so we don't use excessive memory.
                    return Err(io::Error::other("Too many initial packets"));
                }
                handshakes.record_initiation(&packet);

                // Fan out the latest WG packet to all currently spawned transports.
                for (id, running) in running.iter().enumerate() {
                    scratch[..n].copy_from_slice(&packet);
                    if let Err(err) = running.transport.send(&mut scratch[..n]).await {
                        log::error!("Failed to send packet to transport {id}: {err}");
                    }
                }

                initial_packets.push(packet);
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
                    forward(incoming, &running[id].buf[..n]).await?;
                }

                // A handshake response means this transport reached the relay, so use it from
                // now on.
                Received::HandshakeResponse => {
                    let selected = running.swap_remove(id);
                    log::debug!(
                        "Selecting {:?} as the valid transport configuration",
                        selected.config
                    );
                    forward(incoming, &selected.buf[..n]).await?;

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

/// Hand a datagram that discovery received back to whoever is driving the multiplexer.
async fn forward(incoming: &mpsc::Sender<Box<[u8]>>, datagram: &[u8]) -> io::Result<()> {
    incoming
        .send(Box::from(datagram))
        .await
        .map_err(|_| stopped())
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

    /// The address that this transport talks to.
    pub fn endpoint(&self) -> SocketAddr {
        match self {
            Transport::Direct(addr) => *addr,
            Transport::Obfuscated(settings) => settings.remote_endpoint(),
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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::{
        LocalSocketObfuscator,
        local_socket::LocalSocketRunner,
        wireguard::{handshake_initiation, handshake_response},
    };
    use talpid_net::bypass::NoopBypass;
    use talpid_types::net::obfuscation::LwoVersion;
    use tokio::net::UdpSocket;

    async fn local_socket(multiplexer: Multiplexer) -> Box<dyn LocalSocketObfuscator> {
        let runner = LocalSocketRunner::new(Arc::new(multiplexer)).await.unwrap();
        Box::new(runner)
    }

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

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings);
        let multiplexer = local_socket(multiplexer).await;
        let multiplexer_endpoint = multiplexer.endpoint();

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        tokio::spawn(multiplexer.run());

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

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings);
        let multiplexer = local_socket(multiplexer).await;
        let multiplexer_endpoint = multiplexer.endpoint();
        tokio::spawn(multiplexer.run());

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

        let multiplexer = Multiplexer::new(Arc::new(NoopBypass), settings);
        let multiplexer = local_socket(multiplexer).await;
        let multiplexer_endpoint = multiplexer.endpoint();
        tokio::spawn(multiplexer.run());

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
