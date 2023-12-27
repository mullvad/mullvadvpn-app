#[cfg(not(target_os = "ios"))]
use std::net::Ipv4Addr;
use std::{
    io,
    net::{IpAddr, SocketAddr},
};

use boringtun::noise::{errors::WireGuardError, Tunn, TunnResult};
use ios::{data::SwiftDataArray, IOOutput};

// #[cfg(target_os = "ios")]
pub mod ios;
#[cfg(all(unix, not(target_os = "ios")))]
pub mod unix;

pub struct WgInstance {
    peers: Vec<Peer>,
    send_buf: Box<[u8; u16::MAX as usize]>,
}

impl WgInstance {
    pub fn new(config: Config) -> Self {
        let peers = config.create_peers();

        Self {
            peers,
            send_buf: new_send_buf(),
        }
    }
}

impl WgInstance {
    pub fn handle_host_traffic(&mut self, packet: &[u8], output: &mut IoBuffer) {
        // best not to store u16::MAX bytes on the stack if we want to run on iOS
        let mut send_buf = vec![0u8; 2400];

        match self.peers[0].tun.encapsulate(packet, &mut send_buf) {
            TunnResult::WriteToNetwork(buf) => {
                output.send_udp(self.peers[0].endpoint, buf);
            }
            TunnResult::Err(e) => {
                log::error!("Failed to encapsulate IP packet: {e:?}");
            }
            TunnResult::Done => {}
            other => {
                log::error!("Unexpected WireGuard state during encapsulation: {other:?}");
            }
        }
        std::mem::drop(send_buf);
    }

    pub fn handle_timer_tick(&mut self, output: &mut IoBuffer) {
        let mut send_buf = new_send_buf();
        let tun_result = self.peers[0].tun.update_timers(send_buf.as_mut_slice());
        self.inner_handle_timer_tick(tun_result, output);
    }

    fn inner_handle_timer_tick<'a>(&mut self, first_result: TunnResult<'a>, output: &mut IoBuffer) {
        let mut send_buf;
        let mut current_result;
        current_result = first_result;
        loop {
            match current_result {
                TunnResult::Err(WireGuardError::ConnectionExpired) => {
                    log::warn!("WireGuard handshake has expired");
                    send_buf = new_send_buf();
                    current_result = self.peers[0]
                        .tun
                        .format_handshake_initiation(send_buf.as_mut_slice(), false);
                }

                TunnResult::Err(e) => {
                    log::error!("Failed to prepare routine packet for WireGuard: {e:?}");
                    break;
                }

                TunnResult::WriteToNetwork(packet) => {
                    output.send_udp(self.peers[0].endpoint, packet);
                    break;
                }

                TunnResult::Done => {
                    break;
                }
                other => {
                    log::error!("Unexpected WireGuard state {other:?}");
                    break;
                }
            }
        }
    }
}

impl WgInstance {
    pub fn handle_tunnel_traffic(&mut self, packet: &[u8], output: &mut IoBuffer) {
        match self.peers[0]
            .tun
            .decapsulate(None, packet, self.send_buf.as_mut_slice())
        {
            TunnResult::WriteToNetwork(data) => {
                output.send_udp(self.peers[0].endpoint, data);

                match self.peers[0]
                    .tun
                    .decapsulate(None, &[], self.send_buf.as_mut_slice())
                {
                    TunnResult::WriteToNetwork(data) => {
                        output.send_udp(self.peers[0].endpoint, data)
                    }
                    _ => {}
                }
            }
            TunnResult::WriteToTunnelV4(clear_packet, _addr) => {
                output.tun_v4_output.append(clear_packet);
            }
            TunnResult::WriteToTunnelV6(clear_packet, _addr) => {
                output.tun_v6_output.append(clear_packet);
            }
            anything_else => {
                log::error!("Unexpected WireGuard result: {anything_else:?}");
            }
        }
    }
}

struct Peer {
    endpoint: SocketAddr,
    tun: Tunn,
}

pub struct Config {
    pub private_key: [u8; 32],
    #[cfg(not(target_os = "ios"))]
    pub address: Ipv4Addr,
    pub peers: Vec<PeerConfig>,
}

impl Config {
    fn create_peers(&self) -> Vec<Peer> {
        self.peers
            .iter()
            .enumerate()
            .map(|(idx, peer)| {
                let tun = Tunn::new(
                    x25519_dalek::StaticSecret::from(self.private_key),
                    x25519_dalek::PublicKey::from(peer.pub_key),
                    None,
                    None,
                    idx.try_into().expect("more than u32::MAX peers"),
                    None,
                )
                .expect("in practice this should never fail");
                Peer {
                    endpoint: peer.endpoint,
                    tun,
                }
            })
            .collect()
    }
}

pub struct PeerConfig {
    pub endpoint: SocketAddr,
    pub pub_key: [u8; 32],
}

pub struct IoBuffer {
    udp_v4_output: SwiftDataArray,
    udp_v6_output: SwiftDataArray,
    tun_v4_output: SwiftDataArray,
    tun_v6_output: SwiftDataArray,
}

impl IoBuffer {
    pub fn new() -> Self {
        Self {
            udp_v4_output: SwiftDataArray::new(),
            udp_v6_output: SwiftDataArray::new(),
            tun_v4_output: SwiftDataArray::new(),
            tun_v6_output: SwiftDataArray::new(),
        }
    }

    pub fn send_udp(&mut self, addr: SocketAddr, buffer: &[u8]) {
        match addr.ip() {
            IpAddr::V4(_) => self.udp_v4_output.append(buffer),
            IpAddr::V6(_) => self.udp_v6_output.append(buffer),
        }
    }

    /// Effectively leaks buffers, whoever consumes IOOutput must release it's memory
    pub fn to_output(self) -> IOOutput {
        IOOutput {
            udp_v4_output: self.udp_v4_output.into_raw(),
            udp_v6_output: self.udp_v6_output.into_raw(),
            tun_v4_output: self.tun_v4_output.into_raw(),
            tun_v6_output: self.tun_v6_output.into_raw(),
        }
    }
}

impl IoBuffer {}

#[async_trait::async_trait]
pub trait AsyncUdpTransport {
    async fn send_packet(&self, addr: IpAddr, buffer: &[u8]) -> io::Result<()>;
    async fn receive_packet(&self, addr: IpAddr, buffer: &[u8]) -> io::Result<()>;
}

fn new_send_buf() -> Box<[u8; u16::MAX as usize]> {
    Box::<[u8; u16::MAX as usize]>::try_from(vec![0u8; u16::MAX as usize]).unwrap()
}
