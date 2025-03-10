use std::{
    collections::{BTreeMap, VecDeque},
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::{net::UdpSocket, task::JoinHandle, time::Instant};
use tokio_util::task::AbortOnDropHandle;

use crate::socket::create_remote_socket;

const MAX_DATAGRAM_SIZE: usize = u16::MAX as usize;

pub struct Multiplexer {
    client_socket: Arc<UdpSocket>,
    proxy_socket: Arc<UdpSocket>,
    client_socket_addr: SocketAddr,
    running_endpoints: BTreeMap<SocketAddr, Transport>,
    transports: VecDeque<Transport>,
    initial_packets_to_send: Vec<Vec<u8>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Multiplexer {
    pub async fn new(settings: &Settings) -> crate::Result<Self> {
        let client_socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(crate::Error::CreateMultiplexerObfuscator)?;

        let client_socket_addr = client_socket
            .local_addr()
            .map_err(crate::Error::CreateMultiplexerObfuscator)?;

        // TODO: Support IPv6
        let proxy_socket = create_remote_socket(
            true,
            #[cfg(target_os = "linux")]
            settings.fwmark,
        )
        .await?;

        Ok(Self {
            client_socket: Arc::new(client_socket),
            proxy_socket: Arc::new(proxy_socket),
            client_socket_addr,
            running_endpoints: BTreeMap::new(),
            transports: VecDeque::from(settings.transports.clone()),
            tasks: vec![],
            initial_packets_to_send: vec![],
        })
    }

    async fn start(mut self) -> io::Result<()> {
        log::debug!("Running multiplexer obfuscation");

        let mut wg_recv_buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut obfuscator_recv_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        // let mut last_wg_packet: Option<Vec<u8>> = None;
        let delay = tokio::time::interval_at(Instant::now(), Duration::from_secs(1));
        let mut wg_addr = None;
        tokio::pin!(delay);
        loop {
            tokio::select! {
                socket_recv = self.client_socket.recv_from(&mut wg_recv_buf) => {

                    match socket_recv {
                        Ok((bytes_received, from_addr)) => {
                            wg_addr = Some(from_addr);
                            let received_packet = &wg_recv_buf[..bytes_received];
                            self.initial_packets_to_send.push(received_packet.to_vec());

                            // last_wg_packet = Some(received_packet.clone());
                            for (addr, obfs) in &self.running_endpoints {
                                log::info!("Sending received packet to proxy {obfs:?}");
                               if let Err(err) = self.proxy_socket.send_to(received_packet, addr).await{
                                log::error!("Failed to send received packet to proxy {addr}: {err}");
                               } else {
                                log::info!("Successfully sent traffic to obfuscator {addr}");
                               }
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to receive traffic from local WireGuard instance: {err}");
                            return Ok(());
                        }
                    };

                },
                obfuscator_recv = self.proxy_socket.recv_from(&mut obfuscator_recv_buf) => {
                    log::info!("Did receive local socket: {obfuscator_recv:?}");

                    match obfuscator_recv {
                        Ok((bytes_received, obfuscator_addr)) => {
                            let Some(wg_addr) = wg_addr else {
                                log::error!("Received from proxy before ever receiving any traffic from local WireGuard");
                                return Ok(());
                            };

                            log::info!("Did receive bytes from obfuscator: °{obfuscator_addr:?}");

                            let _ = self.client_socket.send_to(&obfuscator_recv_buf[..bytes_received], wg_addr).await;
                            if let Some(transport_config) = self.running_endpoints.get(&obfuscator_addr) {
                                log::debug!("Selecting {:?} as valid transport configuration, via {}",  transport_config, obfuscator_addr);
                            }

                            // run proxy in _connected_ mode
                            self.run_connected(wg_addr, obfuscator_addr).await?;
                            return Ok(())
                        },
                        Err(err) => {
                            log::error!("Failed to receive traffic from obfuscators: {err}");
                            return Err(err);
                        }
                    }

                },
               _ = delay.tick() => {
                let Some(transport) = self.transports.pop_front() else {
                    continue;
                };

                if let Err(err) = self.spawn_new_transport(transport).await {
                    log::error!("Failed to spawn new transport: {err}");
                }

                }
            }
        }
    }

    async fn run_connected(
        self,
        local_address: SocketAddr,
        proxy_address: SocketAddr,
    ) -> io::Result<()> {
        let mut wg_recv_buf: Vec<u8> = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut obfuscator_recv_buf: Vec<u8> = vec![0u8; MAX_DATAGRAM_SIZE];

        self.client_socket
            .connect(local_address)
            .await
            .inspect_err(|err| {
                log::error!("Failed to connect client socket: {err}");
            })?;

        let tx_client_socket = self.client_socket.clone();
        let tx_proxy_socket = self.proxy_socket.clone();

        let tx_task: JoinHandle<io::Result<()>> = tokio::spawn(async move {
            loop {
                let n = tx_client_socket.recv(&mut wg_recv_buf).await?;
                tx_proxy_socket
                    .send_to(&wg_recv_buf[..n], proxy_address)
                    .await?;
            }
        });
        let mut tx_task = AbortOnDropHandle::new(tx_task);

        let rx_task: JoinHandle<io::Result<()>> = tokio::spawn(async move {
            loop {
                let (n, _src) = self
                    .proxy_socket
                    .recv_from(&mut obfuscator_recv_buf)
                    .await?;
                self.client_socket.send(&obfuscator_recv_buf[..n]).await?;
            }
        });
        let mut rx_task = AbortOnDropHandle::new(rx_task);

        tokio::select! {
            Ok(result) = &mut tx_task => result,
            Ok(result) = &mut rx_task => result,
            else => Ok(()),
        }
    }

    async fn spawn_new_transport(&mut self, transport: Transport) -> crate::Result<()> {
        let endpoint = match transport {
            settings @ Transport::Direct(addr) => {
                self.running_endpoints.insert(addr, settings);
                log::info!("Spawning direct forwarder");
                Ok(addr)
            }
            Transport::Obfuscated(obfuscator_settings) => {
                let obfuscator = crate::create_obfuscator(&obfuscator_settings).await?;
                let endpoint = obfuscator.endpoint();
                self.running_endpoints
                    .insert(endpoint, Transport::Obfuscated(obfuscator_settings));
                self.tasks.push(tokio::spawn(async move {
                    log::info!("Spawning new obfuscator");
                    let _ = obfuscator.run().await;
                }));
                Ok(endpoint)
            }
        }?;

        // Send initial packets
        for packet in &self.initial_packets_to_send {
            if let Err(err) = self.proxy_socket.send_to(packet, endpoint).await {
                log::error!("Failed to forward packet to new obfuscator: {err}");
            }
        }

        Ok(())
    }
}

impl Drop for Multiplexer {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// Transports to use, ordered by highest to lowest priority
    pub transports: Vec<Transport>,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

#[derive(Clone, Debug)]
pub enum Transport {
    Direct(SocketAddr),
    Obfuscated(crate::Settings),
}

#[tokio::test]
async fn test_multi_sleep() {
    let sleep = tokio::time::sleep(Duration::from_secs(0));
    sleep.await;
}

#[async_trait]
impl crate::Obfuscator for Multiplexer {
    fn endpoint(&self) -> SocketAddr {
        self.client_socket_addr
    }

    fn packet_overhead(&self) -> u16 {
        60
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.start()
            .await
            .map_err(crate::Error::RunMultiplexerObfuscator)
    }
}
