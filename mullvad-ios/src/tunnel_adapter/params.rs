use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use gotatun::{
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::MtuWatcher,
    x25519::StaticSecret,
};
use ipnetwork::IpNetwork;

use crate::gotatun::smoltcp_network::SmoltcpNetworkConfig;

/// WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
pub const WIREGUARD_OVERHEAD: u16 = 8 + 32;

/// Parameters for a tunnel, as handed over by the FFI.
///
/// These are never modified once handed over. The configuration of each GotaTun device is
/// derived from them together with the state negotiated with the relays and the obfuscator.
#[derive(Debug, Clone)]
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

    pub(super) fn smoltcp_network_config(&self) -> SmoltcpNetworkConfig {
        SmoltcpNetworkConfig {
            ipv4_addr: self.ipv4_addr,
            ipv6_addr: Some(self.ipv6_addr),
            mtu: self.smoltcp_mtu(),
        }
    }

    /// Timeout for establishing connectivity, clamped to at least one second.
    pub(super) fn establish_timeout(&self) -> Duration {
        Duration::from_secs(self.establish_timeout_secs.max(1) as u64)
    }

    pub(super) fn is_multihop(&self) -> bool {
        self.entry_peer.is_some()
    }

    /// The relay the device talks to directly: the entry in multihop, otherwise the exit.
    pub(super) fn ingress_peer(&self) -> &PeerParameters {
        self.entry_peer.as_ref().unwrap_or(&self.exit_peer)
    }

    pub(super) fn device_key(&self) -> StaticSecret {
        StaticSecret::from(self.private_key)
    }

    /// MTU of the entry device in multihop: the tunnel MTU plus the entry hop's per-packet
    /// overhead.
    pub(super) fn entry_mtu(&self, entry: &PeerParameters) -> MtuWatcher {
        MtuWatcher::new(self.mtu)
            .increase(multihop_overhead(entry.endpoint))
            .expect("MTU overflow")
    }
}

/// Per-packet overhead the entry hop adds to the exit device's MTU budget.
pub(super) fn multihop_overhead(entry_endpoint: SocketAddr) -> u16 {
    let overhead = match entry_endpoint.ip() {
        IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
        IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
    };
    overhead as u16
}

#[derive(Clone, Debug)]
pub struct PeerParameters {
    pub public_key: [u8; 32],
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNetwork>,
}

/// Obfuscation method for the connection to the ingress relay.
#[derive(Debug, Clone)]
pub enum ObfuscationParameters {
    Off,
    UdpOverTcp,
    Shadowsocks,
    Quic {
        hostname: String,
        token: String,
    },
    Lwo {
        server_public_key: [u8; 32],
    },
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

    #[test]
    fn multihop_overhead_is_larger_for_ipv6() {
        let v4 = multihop_overhead("1.2.3.4:51820".parse().unwrap());
        let v6 = multihop_overhead("[2001:db8::1]:51820".parse().unwrap());
        assert!(
            v6 > v4,
            "IPv6 header is larger than IPv4 (v4={v4}, v6={v6})"
        );
        assert_eq!((v6 - v4) as usize, Ipv6Header::LEN - Ipv4Header::LEN);
    }

    #[test]
    fn ingress_peer_is_entry_in_multihop_else_exit() {
        let mut p = params();
        assert_eq!(p.ingress_peer().endpoint, p.exit_peer.endpoint);
        p.entry_peer = Some(peer("9.9.9.9:51820"));
        assert_eq!(p.ingress_peer().endpoint, "9.9.9.9:51820".parse().unwrap());
    }
}
