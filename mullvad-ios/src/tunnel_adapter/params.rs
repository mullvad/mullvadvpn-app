use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use ipnetwork::IpNetwork;

/// WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
pub const WIREGUARD_OVERHEAD: u16 = 8 + 32;

/// Configuration for a single tunnel connection attempt.
pub struct TunnelParameters {
    pub tun_fd: i32,
    pub private_key: [u8; 32],
    pub ipv4_addr: Ipv4Addr,
    pub ipv6_addr: Ipv6Addr,
    pub mtu: u16,
    pub exit_peer: PeerParameters,
    pub entry_peer: Option<PeerParameters>,
    pub ipv4_gateway: Ipv4Addr,
    pub establish_timeout_secs: u32,
    pub enable_pq: bool,
    pub enable_daita: bool,
    pub obfuscation: ObfuscationParameters,
}

impl TunnelParameters {
    /// MTU available to the inner smoltcp stack after WireGuard overhead.
    pub(super) fn smoltcp_mtu(&self) -> u16 {
        self.mtu.saturating_sub(WIREGUARD_OVERHEAD)
    }

    /// Timeout for establishing connectivity, clamped to at least one second.
    pub(super) fn establish_timeout(&self) -> Duration {
        Duration::from_secs(self.establish_timeout_secs.max(1) as u64)
    }
}

/// Obfuscation configuration for the tunnel.
#[cfg_attr(test, derive(Debug))]
pub enum ObfuscationParameters {
    Off,
    UdpOverTcp,
    Shadowsocks,
    Quic { hostname: String, token: String },
    Lwo { server_public_key: [u8; 32] },
}

pub struct PeerParameters {
    pub public_key: [u8; 32],
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNetwork>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn peer(endpoint: &str) -> PeerParameters {
        PeerParameters {
            public_key: [7u8; 32],
            endpoint: endpoint.parse().unwrap(),
            allowed_ips: vec!["0.0.0.0/0".parse().unwrap()],
        }
    }

    pub(crate) fn params() -> TunnelParameters {
        TunnelParameters {
            tun_fd: -1,
            private_key: [0u8; 32],
            ipv4_addr: Ipv4Addr::new(10, 0, 0, 2),
            ipv6_addr: "fd00::2".parse().unwrap(),
            mtu: 1280,
            exit_peer: peer("1.2.3.4:51820"),
            entry_peer: None,
            ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
            establish_timeout_secs: 4,
            enable_pq: false,
            enable_daita: false,
            obfuscation: ObfuscationParameters::Off,
        }
    }

    #[test]
    fn establish_timeout_clamps_to_at_least_one_second() {
        let mut p = params();
        p.establish_timeout_secs = 0;
        assert_eq!(p.establish_timeout(), Duration::from_secs(1));
        p.establish_timeout_secs = 7;
        assert_eq!(p.establish_timeout(), Duration::from_secs(7));
    }

    #[test]
    fn smoltcp_mtu_subtracts_wireguard_overhead_and_saturates() {
        let mut p = params();
        p.mtu = 1280;
        assert_eq!(p.smoltcp_mtu(), 1280 - WIREGUARD_OVERHEAD);
        p.mtu = 10; // smaller than the overhead
        assert_eq!(p.smoltcp_mtu(), 0);
    }
}
