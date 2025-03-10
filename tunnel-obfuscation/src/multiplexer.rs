use std::{
    collections::BTreeMap,
    error::Error,
    future::{self, Future},
    net::{Ipv4Addr, SocketAddr},
    ops::Mul,
    time::{Duration},
};

use async_trait::async_trait;
use tokio::{net::UdpSocket, task::JoinHandle, time::Instant};

pub struct Multiplexer {
    local_socket: UdpSocket,
    proxy_socket: UdpSocket,
    running_endpoints: BTreeMap<SocketAddr, Transport>,
    transports: Vec<Transport>,
    packets_to_send: Vec<Vec<u8>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Multiplexer {
    pub async fn new(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let local_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        let proxy_socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;

        Ok(Self {
            local_socket,
            proxy_socket,
            running_endpoints: BTreeMap::new(),
            transports: settings.transports,
            tasks: vec![],
            packets_to_send: vec![],
        })
    }
    pub async fn start(mut self) {        let mut wg_recv_buf: Vec<u8> = vec![0u8; 1600];
        let mut obfuscator_recv_buf: Vec<u8> = vec![0u8; 1600];

        let mut last_wg_packet: Option<Vec<u8>> = None;
        let delay = tokio::time::interval_at(Instant::now(), Duration::from_secs(1));
        let mut wg_addr = None;
        tokio::pin!(delay);
        loop {
            tokio::select! {
                socket_recv = self.local_socket.recv_from(&mut wg_recv_buf) => {                    
                    match socket_recv {
                        Ok((bytes_received, from_addr)) => {
                            wg_addr = Some(from_addr);
                            let received_packet = wg_recv_buf[..bytes_received].to_vec();
                            self.packets_to_send.push(received_packet.clone());
                            last_wg_packet = Some(received_packet.clone());
                            for addr in self.running_endpoints.keys() {
                               if let Err(err) = self.proxy_socket.send_to(&received_packet, addr).await{
                                log::error!("Failed to send received packet to proxy {addr}: {err}");
                               } else {
                                log::info!("Successfully sent traffic to obfuscator {addr}");
                               }
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to receive traffic from local WireGuard instance: {err}");
                            return;
                        }
                    };

                },
                obfuscator_recv = self.proxy_socket.recv_from(&mut obfuscator_recv_buf) => {
                    log::info!("Did receive local socket: {obfuscator_recv:?}");

                    match obfuscator_recv {
                        Ok((bytes_received, obfuscator_addr)) => {
                            let Some(wg_addr) = wg_addr else {
                                log::error!("Received from proxy before ever receiving any traffic from local WireGuard");
                                return;
                            };
                            
                            log::info!("Did receive bytes from obfuscator: °{obfuscator_addr:?}");

                            let _ = self.local_socket.send_to(&obfuscator_recv_buf[..bytes_received], wg_addr).await;
                            if let Some(transport_config) = self.running_endpoints.get(&obfuscator_addr) {
                                log::debug!("Selecting {:?} as valid transport configuration, via {}",  transport_config, obfuscator_addr);
                            }

                            // run proxy in _connected_ mode
                            self.run_connected(wg_addr, obfuscator_addr).await;
                            return;
                        },
                        Err(err) => {
                            log::error!("Failed to receive traffic from obfuscators: {err}");
                            return;
                        }
                    }

                },
               _ = delay.tick() => {
                let Some(transport) = self.transports.pop() else {
                    continue;
                };

                log::info!("Spawned new obfuscator {transport:?}");

                if let Err(err) = self.spawn_new_transport(transport).await {
                    log::error!("Failed to spawn new transport: {err}");
                }

                }
            }
        }
    }

    async fn run_connected(self, local_address: SocketAddr, proxy_address: SocketAddr) {
        let mut wg_recv_buf: Vec<u8> = vec![0u8; 1600];
        let mut obfuscator_recv_buf: Vec<u8> = vec![0u8; 1600];

        loop {
            tokio::select! {
                Ok((bytes_received, _)) = self.local_socket.recv_from(&mut wg_recv_buf) => {
                    let _ = self.proxy_socket.send_to(&wg_recv_buf[..bytes_received], proxy_address).await;
                }
                Ok((bytes_received, _)) = self.proxy_socket.recv_from(&mut obfuscator_recv_buf) => {

                    let _ = self.local_socket.send_to(&obfuscator_recv_buf[..bytes_received], local_address).await;
                }
            };
        }
    }

    async fn spawn_new_transport(&mut self, transport: Transport) -> crate::Result<()> {
        match transport {
            settings @ Transport::Direct(addr) => {
                self.running_endpoints.insert(addr, settings);

                Ok(())
            }
            Transport::Obfuscated(obfuscator_settings) => {
                let obfuscator = crate::create_obfuscator(&obfuscator_settings).await?;
                let endpoint = obfuscator.endpoint();
                self.running_endpoints
                    .insert(endpoint, Transport::Obfuscated(obfuscator_settings));
                self.tasks.push(tokio::spawn(async move { let _ = obfuscator.run().await; }));

                for packet in &self.packets_to_send {
                    let _ = self.proxy_socket.send_to(&packet, endpoint).await;
                }

                Ok(())
            }
        }
    }
}

pub struct Settings {
    pub transports: Vec<Transport>,
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
    #[doc = " Returns the address of the local socket."]
    fn endpoint(&self) -> SocketAddr {
        self.local_socket
            .local_addr()
            .expect("Failed to obtain local address")
    }

    fn packet_overhead(&self) -> u16 {
        60
    }
    
    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.start().await;
        Ok(())
    }
}
