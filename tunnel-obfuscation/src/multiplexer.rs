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
//! 2. **Transport Spawning**: It progressively spawns different obfuscation transports at timed intervals
//! 3. **Traffic Fanout**: All incoming WireGuard packets are fanned out to all active transports
//! 4. **First Response Wins**: The first transport to receive a response from the server is selected
//! 5. **Connection Establishment**: Once a transport is selected, the multiplexer switches to a
//!    direct forwarding mode between WireGuard and the selected transport
//!
//! ## Transport Types
//!
//! See the [Transport] enum.

use std::{
    collections::{BTreeMap, VecDeque},
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio_util::task::AbortOnDropHandle;

use crate::socket::create_remote_socket;

const MAX_DATAGRAM_SIZE: usize = u16::MAX as usize;

/// Max number of initial outgoing packets to buffer for replaying to new transports
const MAX_INITIAL_PACKETS: usize = 100;

/// An obfuscator that manages multiple other obfuscators and automatically
/// selects the first one that successfully establishes a connection.
///
/// The multiplexer operates in two phases:
/// 1. **Discovery Phase**: Spawn transports progressively and fan out traffic to all of them
/// 2. **Connected Phase**: Once a transport responds, switch to forwarding to that transport only
pub struct Multiplexer {
    /// Local UDP socket that WireGuard connects to
    client_socket: Arc<UdpSocket>,
    /// Address of the client socket that WireGuard should connect to
    client_socket_addr: SocketAddr,
    /// IPv4 socket for communicating with obfuscation proxies
    proxy_socket_v4: Arc<UdpSocket>,
    /// IPv6 socket for communicating with obfuscation proxies
    proxy_socket_v6: Arc<UdpSocket>,
    /// Map of currently active transport endpoints and their configurations
    running_endpoints: BTreeMap<SocketAddr, Transport>,
    /// Queue of transports to spawn (in priority order)
    transports: VecDeque<Transport>,
    /// Buffer of initial packets received from WireGuard to replay to new transports
    initial_packets_to_send: Vec<Vec<u8>>,
    /// Handles to spawned obfuscation tasks
    tasks: Vec<AbortOnDropHandle<()>>,
    /// Address of WG endpoint socket
    wg_addr: Option<SocketAddr>,
}

impl Multiplexer {
    /// Create a new multiplexer with the specified transports (obfuscators) and settings.
    ///
    /// # Arguments
    /// * `settings` - Configuration containing the list of transports to try and network settings
    ///
    /// # Returns
    /// A new multiplexer instance ready to start obfuscation discovery
    pub async fn new(settings: &Settings) -> crate::Result<Self> {
        let client_socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(crate::Error::CreateMultiplexerObfuscator)?;

        let client_socket_addr = client_socket
            .local_addr()
            .map_err(crate::Error::CreateMultiplexerObfuscator)?;

        let proxy_socket_v4 = create_remote_socket(
            true,
            #[cfg(target_os = "linux")]
            settings.fwmark,
        )
        .await?;
        let proxy_socket_v6 = create_remote_socket(
            false,
            #[cfg(target_os = "linux")]
            settings.fwmark,
        )
        .await?;

        Ok(Self {
            client_socket: Arc::new(client_socket),
            client_socket_addr,
            proxy_socket_v4: Arc::new(proxy_socket_v4),
            proxy_socket_v6: Arc::new(proxy_socket_v6),
            running_endpoints: BTreeMap::new(),
            transports: VecDeque::from(settings.transports.clone()),
            tasks: vec![],
            initial_packets_to_send: vec![],
            wg_addr: None,
        })
    }

    fn proxy_for_addr(&self, addr: SocketAddr) -> &Arc<UdpSocket> {
        if addr.is_ipv4() {
            &self.proxy_socket_v4
        } else {
            &self.proxy_socket_v6
        }
    }

    /// Start the multiplexer in discovery mode.
    ///
    /// Run the main event loop:
    /// 1. Receive packets from WireGuard and fan them out to all active transports
    /// 2. Receive responses from obfuscation proxies
    /// 3. Spawn new transports at timed intervals
    /// 4. Switch to connected mode when the first transport responds successfully
    async fn start(mut self) -> io::Result<()> {
        log::debug!("Running multiplexer obfuscation");

        let mut wg_recv_buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut obfs_recv_v4_buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut obfs_recv_v6_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        let mut delay = tokio::time::interval(Duration::from_secs(1));

        /// Helper to fan out a packet to all currently running endpoints
        async fn send_to_all<'a>(
            endpoints: &BTreeMap<SocketAddr, Transport>,
            get_socket: impl Fn(SocketAddr) -> &'a Arc<UdpSocket>,
            packet: &[u8],
        ) {
            let mut futs = vec![];
            for &addr in endpoints.keys() {
                let udp = get_socket(addr);
                futs.push(async move {
                    log::info!("Sending received packet to proxy {addr}");
                    if let Err(err) = udp.send_to(packet, addr).await {
                        log::error!("Failed to send received packet to proxy {addr}: {err}");
                    } else {
                        log::info!("Successfully sent traffic to obfuscator {addr}");
                    }
                });
            }
            futures::future::join_all(futs).await;
        }

        loop {
            tokio::select! {
                // From local WG
                socket_recv = self.client_socket.recv_from(&mut wg_recv_buf) => {
                    match socket_recv {
                        Ok((bytes_received, from_addr)) => {
                            if let Some(prev_addr) = self.wg_addr && prev_addr != from_addr {
                                log::debug!(
                                    "WireGuard endpoint address changed from {prev_addr} to {from_addr}"
                                );
                            }
                            self.wg_addr = Some(from_addr);
                            let pkt = &wg_recv_buf[..bytes_received];

                            if self.initial_packets_to_send.len() >= MAX_INITIAL_PACKETS {
                                // Initial packets should be handshake initiation packets, so we
                                // should not end up here if there's some reasonable timeout.
                                // If we do, fail so we don't use excessive memory.
                                return Err(io::Error::other("Too many initial packets"));
                            }

                            self.initial_packets_to_send.push(pkt.to_vec());

                            // Fan out latest WG packet to all currently spawned endpoints.
                            send_to_all(
                                &self.running_endpoints,
                                |addr| self.proxy_for_addr(addr),
                                pkt
                            ).await;
                        },
                        Err(err) => {
                            log::error!("Failed to receive traffic from local WireGuard instance: {err}");
                            return Ok(());
                        }
                    }
                },

                // From any IPv4 proxy
                obfuscator_recv = self.proxy_socket_v4.recv_from(&mut obfs_recv_v4_buf) => {
                    self.process_obfuscator_recv(obfuscator_recv.map(|(n, addr)| (&obfs_recv_v4_buf[..n], addr))).await?;
                },

                // From any IPv6 proxy
                obfuscator_recv = self.proxy_socket_v6.recv_from(&mut obfs_recv_v6_buf) => {
                    self.process_obfuscator_recv(obfuscator_recv.map(|(n, addr)| (&obfs_recv_v6_buf[..n], addr))).await?;
                },

                // Spawning the next transport
                _ = delay.tick() => {
                    let Some(transport) = self.transports.pop_front() else { continue; };
                    if let Err(err) = self.spawn_new_transport(transport).await {
                        log::error!("Failed to spawn new transport: {err}");
                    }
                }
            }
        }
    }

    /// Handler for packets received from any proxy.
    ///
    /// If received bytes were forwarded from an obfuscator back to wireguard, this indicates that
    /// a handshake response was received (hopefully) and that we should switch to connected mode.
    ///
    /// If a packet was received, this continues running until `run_connected` returns.
    async fn process_obfuscator_recv(
        &self,
        obfuscator_recv: io::Result<(&[u8], SocketAddr)>,
    ) -> io::Result<()> {
        match obfuscator_recv {
            Ok((received, obfuscator_addr)) => {
                let Some(transport_config) = self.running_endpoints.get(&obfuscator_addr) else {
                    log::trace!("Ignoring data from unexpected address {obfuscator_addr}");
                    return Ok(());
                };
                let Some(wg_addr) = self.wg_addr else {
                    log::trace!(
                        "Received data from {obfuscator_addr} before receiving any data from WireGuard"
                    );
                    return Ok(());
                };
                log::debug!(
                    "Selecting {:?} as valid transport configuration via {obfuscator_addr}",
                    transport_config
                );
                let _ = self.client_socket.send_to(received, wg_addr).await;
                self.run_connected(wg_addr, obfuscator_addr).await
            }
            Err(err) => {
                log::error!("Failed to receive traffic from obfuscators: {err}");
                Err(err)
            }
        }
    }

    /// Switch to connected mode after a transport has been successfully selected.
    ///
    /// In this mode, the multiplexer acts as a simple UDP proxy between WireGuard
    /// and the selected obfuscation transport.
    ///
    /// # Arguments
    /// * `local_address` - Address of the local WireGuard instance
    /// * `proxy_address` - Address of the selected obfuscation proxy
    async fn run_connected(
        &self,
        local_address: SocketAddr,
        proxy_address: SocketAddr,
    ) -> io::Result<()> {
        let mut wg_recv_buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut obfuscator_recv_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        self.client_socket
            .connect(local_address)
            .await
            .inspect_err(|err| {
                log::error!("Failed to connect client socket: {err}");
            })?;

        let proxy_socket = self.proxy_for_addr(proxy_address).clone();

        let tx_client_socket = self.client_socket.clone();
        let tx_proxy_socket = proxy_socket.clone();

        let tx_task = tokio::spawn(async move {
            loop {
                let n = tx_client_socket.recv(&mut wg_recv_buf).await?;
                tx_proxy_socket
                    .send_to(&wg_recv_buf[..n], proxy_address)
                    .await?;
            }
        });
        let mut tx_task = AbortOnDropHandle::new(tx_task);
        let client_socket = self.client_socket.clone();

        let rx_task = tokio::spawn(async move {
            loop {
                let (n, _src) = proxy_socket.recv_from(&mut obfuscator_recv_buf).await?;
                client_socket.send(&obfuscator_recv_buf[..n]).await?;
            }
        });
        let mut rx_task = AbortOnDropHandle::new(rx_task);

        tokio::select! {
            Ok(result) = &mut tx_task => result,
            Ok(result) = &mut rx_task => result,
            else => Ok(()),
        }
    }

    /// Spawn a new obfuscation transport and add it to the active set.
    ///
    /// For direct transports, simply register the endpoint. For obfuscated
    /// transports, start the obfuscation process in a background task.
    ///
    /// # Arguments
    /// * `transport` - The obfuscation type to spawn
    async fn spawn_new_transport(&mut self, transport: Transport) -> crate::Result<()> {
        let endpoint = match transport.clone() {
            Transport::Direct(addr) => {
                self.running_endpoints.insert(addr, transport);
                log::info!("Spawning direct forwarder");
                Ok(addr)
            }
            Transport::Obfuscated(obfuscator_settings) => {
                let obfuscator = crate::create_obfuscator(&obfuscator_settings).await?;
                let endpoint = obfuscator.endpoint();
                self.running_endpoints
                    .insert(endpoint, Transport::Obfuscated(obfuscator_settings));
                self.tasks
                    .push(AbortOnDropHandle::new(tokio::spawn(async move {
                        log::info!("Spawning new obfuscator");
                        let _ = obfuscator.run().await;
                    })));
                Ok(endpoint)
            }
        }?;

        self.send_initial_packets_to(endpoint).await;

        Ok(())
    }

    async fn send_initial_packets_to(&self, endpoint: SocketAddr) {
        let udp = self.proxy_for_addr(endpoint);
        for packet in &self.initial_packets_to_send {
            if let Err(err) = udp.send_to(packet, endpoint).await {
                log::error!("Failed to forward packet to new obfuscator {endpoint}: {err}");
            }
        }
    }
}

/// Configuration settings for multiplexer obfuscation
#[derive(Debug, Clone)]
pub struct Settings {
    /// List of transports to try, ordered by priority (highest to lowest).
    /// Spawn these transports progressively and select
    /// the first one that successfully establishes a connection.
    pub transports: Vec<Transport>,
    /// Linux-specific firewall mark for outgoing connections
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

/// Represents a transport method that the multiplexer can use.
#[derive(Clone, Debug)]
pub enum Transport {
    /// Direct UDP forwarding without any obfuscation
    Direct(SocketAddr),
    /// An obfuscated transport (UDP2TCP, Shadowsocks, QUIC, etc.)
    Obfuscated(crate::Settings),
}

#[async_trait]
impl crate::Obfuscator for Multiplexer {
    fn endpoint(&self) -> SocketAddr {
        self.client_socket_addr
    }

    fn packet_overhead(&self) -> u16 {
        // FIXME: This should ideally be the max overhead of all transports,
        // and be lowered when a transport is selected.
        60
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.start()
            .await
            .map_err(crate::Error::RunMultiplexerObfuscator)
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        unimplemented!("must return the socket fd of every obfuscator here")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Obfuscator;

    /// Test whether the multiplexer works with a direct transports
    #[tokio::test(start_paused = true)]
    async fn test_multiplexer_direct_forwarding() {
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let server_socket2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr2 = server_socket2.local_addr().unwrap();

        // Create multiplexer pointing to a single direct transport
        let settings = Settings {
            transports: vec![
                Transport::Direct(server_addr),
                Transport::Direct(server_addr2),
            ],
            #[cfg(target_os = "linux")]
            fwmark: None,
        };

        let multiplexer = Multiplexer::new(&settings).await.unwrap();
        let multiplexer_endpoint = multiplexer.endpoint();

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        tokio::spawn(async move {
            let boxed_multiplexer = Box::new(multiplexer);
            boxed_multiplexer.run().await
        });

        // Send a test packet from client to multiplexer and verify that it is received
        // NOTE: This may have to be an actual WireGuard handshake packet in the future
        let test_data = b"Ping!";
        client_socket
            .send_to(test_data, multiplexer_endpoint)
            .await
            .unwrap();

        let mut server_buf = vec![0u8; 1024];
        let (bytes_received, client_addr) = server_socket.recv_from(&mut server_buf).await.unwrap();

        assert_eq!(&server_buf[..bytes_received], test_data);

        // Our second socket should also receive this packet
        let (bytes_received, _) = server_socket2.recv_from(&mut server_buf).await.unwrap();
        assert_eq!(&server_buf[..bytes_received], test_data);

        // Send a response back from the first server
        let response_data = b"Pong!";
        server_socket
            .send_to(response_data, client_addr)
            .await
            .unwrap();

        // Verify that response was forwarded
        let mut client_buf = vec![0u8; 1024];
        let (bytes_received, _) = client_socket.recv_from(&mut client_buf).await.unwrap();

        assert_eq!(&client_buf[..bytes_received], response_data);

        // Test that packets are now forwarded directly (connected mode)
        let second_test_data = b"Connected!";
        client_socket
            .send_to(second_test_data, multiplexer_endpoint)
            .await
            .unwrap();

        let (bytes_received, _) = server_socket.recv_from(&mut server_buf).await.unwrap();

        assert_eq!(&server_buf[..bytes_received], second_test_data);
    }
}
