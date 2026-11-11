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

use std::{collections::VecDeque, fmt::Display, io, net::SocketAddr, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesUnordered};
use itertools::Itertools;
use talpid_net::bypass::SocketBypass;
use talpid_types::net::wireguard::PublicKey;
use tokio::{
    sync::{Mutex, mpsc, watch},
    time::Instant,
};

use crate::{
    direct::Direct,
    transport::{MAX_DATAGRAM_SIZE, ObfuscatedTransport},
    wireguard::{HandshakeFilter, Received},
};

/// How long to wait before spawning the next transport in the queue.
const SPAWN_INTERVAL: Duration = Duration::from_millis(500);

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
    outgoing_rx: mpsc::Receiver<Box<[u8]>>,
    incoming_tx: mpsc::Sender<Box<[u8]>>,
    phase_tx: watch::Sender<Phase>,
    bypass: Arc<dyn SocketBypass>,
    settings: Settings,
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

pub struct MultiplexerTransport {
    /// Which phase the multiplexer is in.
    phase: watch::Receiver<Phase>,

    /// The largest overhead among the raced transports.
    packet_overhead: u16,

    /// Datagrams to fan out, drained by [`Multiplexer::discover`] while it is still racing.
    outgoing: mpsc::Sender<Box<[u8]>>,

    /// Datagrams that a raced transport received before one was selected.
    ///
    /// The [Mutex] is uncontended in practice.
    incoming: Mutex<mpsc::Receiver<Box<[u8]>>>,
}

impl Multiplexer {
    /// Create a new multiplexer with the specified transports (obfuscators) and settings.
    ///
    /// Racing starts once [`Multiplexer::discover`] is called, and the multiplexer steps aside
    /// once a transport has won.
    ///
    /// # Arguments
    /// * `settings` - Configuration containing the list of transports to try and network settings
    pub fn new(bypass: Arc<dyn SocketBypass>, settings: Settings) -> (Self, MultiplexerTransport) {
        // Use the largest transport overhead.
        let packet_overhead = packet_overhead(&settings.transports);

        let (outgoing, outgoing_rx) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let (incoming_tx, incoming) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let (phase_tx, phase) = watch::channel(Phase::Discovery);

        let transport = MultiplexerTransport {
            phase: phase.clone(),
            packet_overhead,
            outgoing: outgoing.clone(),
            incoming: Mutex::new(incoming),
        };

        (
            Self {
                outgoing_rx,
                incoming_tx,
                phase_tx,
                bypass,
                settings,
            },
            transport,
        )
    }

    /// Race the transports until one of them answers, and return the selected obfuscation method.
    pub async fn discover(mut self) -> io::Result<Selected> {
        let Settings {
            transports,
            client_public_key,
            timeout,
        } = self.settings;
        log::debug!(
            "Multiplexing {} anti-censorship transport(s): {}",
            transports.len(),
            transports.iter().join(", ")
        );

        let result = run_discovery(
            &mut self.outgoing_rx,
            &self.incoming_tx,
            VecDeque::from(transports),
            self.bypass,
            &client_public_key,
            timeout,
        )
        .await;

        let _ = self.phase_tx.send(match &result {
            Ok(selected) => Phase::Connected(selected.transport.clone()),
            Err(err) => Phase::Failed(Arc::new(clone_error(err))),
        });

        result
    }
}

fn stopped() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "the multiplexer stopped before selecting a transport",
    )
}

fn all_failed() -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, "every transport failed")
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
impl ObfuscatedTransport for MultiplexerTransport {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        let phase = self.phase.borrow().clone();
        match phase {
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
/// selected. Fails with [io::ErrorKind::TimedOut] if no transport answers within `timeout` of the
/// last one being spawned.
async fn run_discovery(
    outgoing: &mut mpsc::Receiver<Box<[u8]>>,
    incoming: &mpsc::Sender<Box<[u8]>>,
    mut pending: VecDeque<Transport>,
    bypass: Arc<dyn SocketBypass>,
    client_public_key: &PublicKey,
    timeout: Option<Duration>,
) -> io::Result<Selected> {
    enum Event {
        /// A packet from the local WireGuard instance, or `None` if it went away.
        Wireguard(Option<Box<[u8]>>),
        /// A packet from a running transport, deobfuscated into that transport's buffer.
        Transport(TransportId, io::Result<usize>),
        /// Time to spawn the next pending transport.
        Spawn,
        /// No transport answered in time.
        TimedOut,
    }

    let mut running: Vec<RunningTransport> = vec![];
    let mut initial_packets: Vec<Box<[u8]>> = vec![];
    let mut handshakes = HandshakeFilter::new(client_public_key);

    let mut scratch = vec![0u8; MAX_DATAGRAM_SIZE];

    let mut spawn_timer = tokio::time::interval(SPAWN_INTERVAL);
    // Set once the last transport has been spawned.
    let mut deadline = None;

    loop {
        let event = tokio::select! {
            // From local WG
            packet = outgoing.recv() => Event::Wireguard(packet),

            // From any running transport
            (id, result) = recv_any(&mut running) => Event::Transport(id, result),

            // Spawning the next transport
            _ = spawn_timer.tick() => Event::Spawn,

            // Giving up on all transports
            _ = sleep_until(deadline) => Event::TimedOut,
        };

        match event {
            Event::Wireguard(None) => {
                log::debug!("The local WireGuard instance went away before a transport answered");
                return Err(stopped());
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
                for running in &running {
                    scratch[..n].copy_from_slice(&packet);
                    if let Err(err) = running.transport.send(&mut scratch[..n]).await {
                        log::error!(
                            "Failed to send packet to transport {}: {err}",
                            running.config
                        );
                    }
                }

                initial_packets.push(packet);
            }

            Event::Transport(id, Err(err)) => {
                // A single failing transport is not fatal; the others may still connect.
                log::error!(
                    "Dropping transport {}, which failed to receive traffic: {err}",
                    running[id].config
                );
                running.remove(id);
                // Try the next transport without waiting for its turn.
                spawn_timer.reset_immediately();
                if running.is_empty() && pending.is_empty() {
                    return Err(all_failed());
                }
            }
            Event::Transport(id, Ok(n)) => match handshakes.classify(&running[id].buf[..n]) {
                // Not an answer to anything WireGuard sent. Whatever it is, it is no reason to
                // believe this transport reaches the relay.
                Received::Unrecognized => {
                    log::trace!(
                        "Ignoring unsolicited packet from transport {}",
                        running[id].config
                    );
                }

                // The relay is rate limiting us. WireGuard needs the cookie in order to retry the
                // initiation with a valid `mac2`, but reaching a rate limiter is not proof that
                // this transport carries a handshake, so keep fanning out.
                Received::CookieReply => {
                    log::trace!(
                        "Forwarding cookie reply from transport {}",
                        running[id].config
                    );
                    forward(incoming, &running[id].buf[..n]).await?;
                }

                // A handshake response means this transport reached the relay, so use it from
                // now on.
                Received::HandshakeResponse => {
                    let running_transport = running.swap_remove(id);
                    forward(incoming, &running_transport.buf[..n]).await?;

                    return Ok(Selected {
                        config: running_transport.config,
                        transport: running_transport.transport,
                    });
                }
            },

            Event::Spawn => {
                let Some(config) = pending.pop_front() else {
                    continue;
                };
                log::trace!(
                    "Spawning {config:?} at #{}/{}",
                    running.len() + 1,
                    running.len() + pending.len() + 1,
                );

                match spawn_transport(&bypass, config.clone()).await {
                    Ok(transport) => {
                        send_initial_packets(&transport, &initial_packets, &mut scratch).await;
                        running.push(RunningTransport {
                            config,
                            transport,
                            buf: vec![0u8; MAX_DATAGRAM_SIZE].into_boxed_slice(),
                        });
                    }
                    Err(err) => {
                        log::error!("Failed to spawn transport {config}: {err}");
                        // Try the next transport without waiting for its turn.
                        spawn_timer.reset_immediately();
                    }
                }
                if pending.is_empty() {
                    if running.is_empty() {
                        return Err(all_failed());
                    }
                    if let Some(timeout) = timeout {
                        deadline = Some(Instant::now() + timeout);
                    }
                }
            }

            Event::TimedOut => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "no transport answered in time",
                ));
            }
        }
    }
}

/// Wait until `deadline`, or forever if there is none.
async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
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
        Transport::Direct(addr) => Ok(Arc::new(Direct::new(bypass, addr).await?)),
        Transport::Obfuscated(settings) => {
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
            log::trace!("Failed to forward initial packet to new transport: {err}");
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

/// The transport that the multiplexer committed to.
pub struct Selected {
    /// The configuration that the transport was spawned from.
    pub config: Transport,
    /// The running transport, which has reached the relay. It can be used on its own, without
    /// the multiplexer.
    pub transport: Arc<dyn ObfuscatedTransport>,
}

impl std::fmt::Debug for Selected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Selected")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Configuration settings for multiplexer obfuscation
#[derive(Debug)]
pub struct Settings {
    /// List of transports to try, ordered by priority (highest to lowest).
    /// Spawn these transports progressively and select
    /// the first one that successfully establishes a connection.
    pub transports: Vec<Transport>,
    /// Public key of the local WireGuard instance.
    pub client_public_key: PublicKey,
    /// How long to wait for a transport to answer after the last one was spawned, before giving
    /// up on all of them. `None` waits indefinitely.
    pub timeout: Option<Duration>,
}

/// Represents a transport method that the multiplexer can use.
#[derive(Clone, Debug)]
pub enum Transport {
    /// Direct UDP forwarding without any obfuscation
    Direct(SocketAddr),
    /// An obfuscated transport (UDP2TCP, Shadowsocks, QUIC, etc.)
    Obfuscated(crate::Settings),
}

impl Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip_version = |addr| match addr {
            SocketAddr::V4(_) => "IPv4",
            SocketAddr::V6(_) => "IPv6",
        };
        use crate::Settings::*;
        use Transport::*;
        match self {
            Direct(addr) => {
                write!(f, "Direct WireGuard over {}", ip_version(*addr))
            }
            Obfuscated(Udp2Tcp(c)) => write!(f, "Udp2Tcp over {}", ip_version(c.peer)),
            Obfuscated(Shadowsocks(c)) => {
                write!(f, "Shadowsocks over {}", ip_version(c.shadowsocks_endpoint))
            }
            Obfuscated(Quic(c)) => write!(f, "Quic over {}", ip_version(c.quic_endpoint())),
            Obfuscated(Lwo(c)) => write!(f, "Lwo over {}", ip_version(c.server_addr)),
        }
    }
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

    async fn local_socket(transport: MultiplexerTransport) -> Box<dyn LocalSocketObfuscator> {
        let runner = LocalSocketRunner::new(Arc::new(transport)).await.unwrap();
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
        let settings = Settings {
            transports: vec![
                Transport::Direct(server_addr),
                Transport::Direct(server_addr2),
            ],
            client_public_key: client_key(),
            timeout: None,
        };

        let (multiplexer, transport) = Multiplexer::new(Arc::new(NoopBypass), settings);
        let discover = tokio::spawn(multiplexer.discover());
        let obfuscator = local_socket(transport).await;
        let multiplexer_endpoint = obfuscator.endpoint();

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        tokio::spawn(obfuscator.run());

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

        // The first server, and not the second, should have been selected
        let selected = discover.await.unwrap().unwrap();
        assert!(
            matches!(selected.config, Transport::Direct(addr) if addr == server_addr),
            "the first transport should have been selected, got {:?}",
            selected.config
        );

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

    /// If every transport fails, the multiplexer should give up right away, rather than wait for
    /// the next transport's turn or for a timeout.
    #[tokio::test]
    async fn test_multiplexer_fails_when_every_transport_fails() {
        // Nothing listens on these ports, so the transports' receives fail as soon as they have
        // sent something.
        let closed_port = || async {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            socket.local_addr().unwrap()
        };

        let settings = Settings {
            transports: vec![
                Transport::Direct(closed_port().await),
                Transport::Direct(closed_port().await),
            ],
            client_public_key: client_key(),
            timeout: None,
        };

        let (multiplexer, transport) = Multiplexer::new(Arc::new(NoopBypass), settings);
        let start = Instant::now();
        transport
            .send(&mut handshake_initiation(SESSION))
            .await
            .unwrap();

        let error = tokio::time::timeout(SPAWN_INTERVAL, multiplexer.discover())
            .await
            .expect("the multiplexer should give up before the next transport's turn")
            .expect_err("no transport should be selected");
        assert_eq!(error.kind(), io::ErrorKind::NotConnected);
        assert!(start.elapsed() < SPAWN_INTERVAL);

        let mut buf = [0u8; 64];
        let error = transport.recv(&mut buf).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotConnected);
    }

    /// If no transport answers within the timeout of the last one being spawned, the multiplexer
    /// should give up without selecting one.
    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_times_out() {
        const TIMEOUT: Duration = Duration::from_secs(4);

        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let settings = Settings {
            transports: vec![
                Transport::Direct(relay.local_addr().unwrap()),
                Transport::Direct(relay2.local_addr().unwrap()),
            ],
            client_public_key: client_key(),
            timeout: Some(TIMEOUT),
        };

        let (multiplexer, transport) = Multiplexer::new(Arc::new(NoopBypass), settings);
        let start = Instant::now();

        let error = multiplexer.discover().await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);

        // The second transport is spawned one interval after the first.
        assert_eq!(start.elapsed(), SPAWN_INTERVAL + TIMEOUT);

        let mut buf = [0u8; 64];
        let error = transport.recv(&mut buf).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_ignores_invalid_handshake_responses() {
        let liar = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let liar_addr = liar.local_addr().unwrap();

        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr = relay.local_addr().unwrap();

        let settings = Settings {
            transports: vec![Transport::Direct(liar_addr), Transport::Direct(relay_addr)],
            client_public_key: client_key(),
            timeout: None,
        };

        let (multiplexer, transport) = Multiplexer::new(Arc::new(NoopBypass), settings);
        let discover = tokio::spawn(multiplexer.discover());
        // The phase carries the multiplexer's progress, e.g. whether it has selected a transport.
        let phase = transport.phase.clone();
        let obfuscator = local_socket(transport).await;
        let multiplexer_endpoint = obfuscator.endpoint();
        tokio::spawn(obfuscator.run());

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
            assert!(
                matches!(*phase.borrow(), Phase::Discovery),
                "a packet that answers no initiation selected a transport",
            );
        }

        // This one does.
        let response = handshake_response(SESSION, &client_key()).to_vec();
        relay.send_to(&response, relay_client_addr).await.unwrap();

        // WireGuard should see the valid response, and nothing the liar sent before it.
        let (n, _) = wg_socket.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &response[..]);

        let selected = discover.await.unwrap().unwrap().config;
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

        let settings = Settings {
            transports: vec![Transport::Obfuscated(crate::Settings::Lwo(lwo::Settings {
                server_addr: server_socket.local_addr().unwrap(),
                client_public_key: client_key.clone().into(),
                server_public_key: server_key.clone(),
                // TODO: support v2
                version: LwoVersion::V1,
            }))],
            client_public_key: client_key.clone(),
            timeout: None,
        };

        let (multiplexer, transport) = Multiplexer::new(Arc::new(NoopBypass), settings);
        let discover = tokio::spawn(multiplexer.discover());
        let obfuscator = local_socket(transport).await;
        let multiplexer_endpoint = obfuscator.endpoint();
        tokio::spawn(obfuscator.run());

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

        // The LWO transport should have been selected
        let selected = discover.await.unwrap().unwrap().config;
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
