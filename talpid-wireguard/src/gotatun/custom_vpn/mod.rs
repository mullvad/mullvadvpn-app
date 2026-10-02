//! Custom VPN: a WireGuard tunnel of the user's own, carried inside the Mullvad tunnel.
//!
//! Traffic to the custom VPN's allowed IPs is NATed to the custom VPN tunnel address and
//! encrypted by an inner device, and then carried by the outer (Mullvad) device. All other traffic
//! goes to the outer device directly. See [`build`].

use super::source_filter::SourceFilter;
use crate::{TunnelError, config::Config};
use either::Either;
use gotatun::{
    device::{Device, DeviceBuilder, Peer},
    packet::{Ip, Ipv4Header, Ipv6Header, Packet, PacketBufPool, UdpHeader, WgData},
    tun::{
        IpRecv, IpSend, MtuWatcher,
        channel::{TunChannelRx, TunChannelTx},
        tun_async_device::TunDevice as GotaTunDevice,
    },
    udp::channel::{UdpChannelFactory, new_udp_tun_channel},
    x25519::StaticSecret,
};
use ipnetwork::IpNetwork;
use merge::MergingIpRecv;
use nat::{NatIpRecv, NatIpSend};
use router::{SplitIpRecv, TunRxRouter, TunTxRouter};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::wireguard::CustomVpnConfig;

mod merge;
mod nat;
mod packet;
mod router;
#[cfg(test)]
mod test_util;

/// Capacity of the channels between the router and the devices.
const CHANNEL_CAPACITY: usize = 4000;

/// The device that encrypts traffic to the custom VPN's allowed IPs.
pub type InnerDevice = Device<(
    UdpChannelFactory,
    NatIpSend<GotaTunDevice>,
    NatIpRecv<SplitIpRecv>,
)>;

/// What the outer device (the singlehop device, or the multihop exit device) writes to instead of
/// the TUN device.
type OuterIpSend = TunTxRouter<TunChannelTx, GotaTunDevice>;

/// What the outer device (the singlehop device, or the multihop exit device) reads from instead
/// of the TUN device.
type OuterIpRecv = MergingIpRecv<SplitIpRecv, TunChannelRx>;

/// What the outer device (the singlehop device, or the multihop exit device) writes to: the TUN
/// device, or the custom VPN in front of it.
pub enum TunTx {
    Tun(GotaTunDevice),
    CustomVpn(Box<OuterIpSend>),
}

/// What the outer device (the singlehop device, or the multihop exit device) reads from: the TUN
/// device, or the custom VPN in front of it.
pub enum TunRx {
    Tun(SourceFilter<GotaTunDevice>),
    CustomVpn(OuterIpRecv),
}

impl IpSend for TunTx {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        match self {
            TunTx::Tun(tun) => tun.send(packet).await,
            TunTx::CustomVpn(router) => router.send(packet).await,
        }
    }
}

impl IpRecv for TunRx {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        match self {
            TunRx::Tun(tun) => tun.recv(pool).await.map(Either::Left),
            TunRx::CustomVpn(merged) => merged.recv(pool).await.map(Either::Right),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        match self {
            TunRx::Tun(tun) => tun.mtu(),
            TunRx::CustomVpn(merged) => merged.mtu(),
        }
    }
}

/// The parts of a custom VPN that are shared by the singlehop and multihop variants.
pub struct CustomVpn {
    pub inner_device: InnerDevice,
    /// Splits reads from the TUN device between the inner and the outer device.
    router_task: AbortOnDrop,
}

impl CustomVpn {
    pub async fn stop(self) {
        self.inner_device.stop().await;
        drop(self.router_task);
    }
}

/// Aborts a task when dropped.
struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// The IO that the outer device (the singlehop device, or the multihop exit device) uses instead
/// of the TUN device when there is a custom VPN.
pub struct Frontend {
    pub custom_vpn: CustomVpn,
    pub tun_tx: TunTx,
    pub tun_rx: TunRx,
    /// Per-packet overhead added by the custom VPN WireGuard encapsulation.
    pub overhead: u16,
}

/// Build the custom VPN [`InnerDevice`], and the IO for the outer device.
///
/// Reads from the TUN device are split by destination: traffic to the custom VPN's allowed IPs
/// is NATed to the custom VPN tunnel address and encrypted by the inner device, whose output is
/// merged with all other traffic into the outer device. Decrypted packets from the outer device are
/// sent to the inner device if they come from the custom VPN endpoint, and to the TUN device
/// otherwise.
pub async fn build(
    custom_vpn: &CustomVpnConfig,
    config: &Config,
    tun_dev: &GotaTunDevice,
) -> Result<Frontend, TunnelError> {
    let outer_v4 = config.tunnel_ipv4();
    let outer_v6 = config.tunnel_ipv6();
    let (bridge_mtu, overhead) =
        calculate_mtu_and_overhead(custom_vpn.peer.endpoint.ip(), tun_dev)?;
    let nat_mappings = resolve_nat_mappings(custom_vpn, outer_v4, outer_v6)?;

    // Reads from the system TUN are routed either to the custom VPN or the default outer VPN.
    let mut router = {
        let filtered_tun = SourceFilter::new(tun_dev.clone(), outer_v4, outer_v6);
        TunRxRouter::new(filtered_tun)
    };

    let gateway_v4_net: IpNetwork = IpAddr::from(config.ipv4_gateway).into();
    let gateway_v6_net = config.ipv6_gateway.map(IpAddr::from).map(IpNetwork::from);

    let default_routes: Vec<_> = [
        IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap(),
        IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap(),
        gateway_v4_net,
    ]
    .into_iter()
    .chain(gateway_v6_net)
    .collect();

    let default_output = router.add_routes(&default_routes, CHANNEL_CAPACITY);

    let pvpn_output = router.add_routes(&custom_vpn.peer.allowed_ip, CHANNEL_CAPACITY);
    let router_task = AbortOnDrop(tokio::spawn(router.run(PacketBufPool::new(100))));

    // Channel bridge (inner device UDP <-> outer device TUN). The source addresses must be the
    // outer tunnel addresses, so that packets emitted by the inner device land on the outer device
    // with the correct source address.
    let (bridge_tun_tx, bridge_tun_rx, inner_udp) = new_udp_tun_channel(
        CHANNEL_CAPACITY,
        outer_v4.unwrap_or(Ipv4Addr::UNSPECIFIED),
        outer_v6.unwrap_or(Ipv6Addr::UNSPECIFIED),
        bridge_mtu,
    );

    // Outer device IpRecv: merge direct traffic and traffic encrypted by the inner device
    let outer_ip_recv = MergingIpRecv::new(default_output, bridge_tun_rx, PacketBufPool::new(100));

    // Outer device IpSend: Demux decrypted packets coming back from the outer device.
    // Send to custom VPN if it's from the inner endpoint, otherwise to the standard TUN.
    let outer_ip_send = TunTxRouter::new(
        bridge_tun_tx,
        tun_dev.clone(),
        custom_vpn.peer.endpoint,
        &custom_vpn.peer.allowed_ip,
        config.ipv4_gateway,
        config.ipv6_gateway,
    );

    // Rewrite IP addresses between the inner and outer tunnel environments.
    let mut tun_send = NatIpSend::new(tun_dev.clone());
    let mut pvpn_recv = NatIpRecv::new(pvpn_output);

    if let Some((inner, outer)) = nat_mappings.v4 {
        tun_send = tun_send.with_v4(inner, outer);
        pvpn_recv = pvpn_recv.with_v4(outer, inner);
    }
    if let Some((inner, outer)) = nat_mappings.v6 {
        tun_send = tun_send.with_v6(inner, outer);
        pvpn_recv = pvpn_recv.with_v6(outer, inner);
    }

    let inner_device = DeviceBuilder::new()
        .with_udp(inner_udp)
        .with_ip_pair(tun_send, pvpn_recv)
        .with_private_key(StaticSecret::from(custom_vpn.tunnel.private_key.to_bytes()))
        .with_peer(
            Peer::new((*custom_vpn.peer.public_key.as_bytes()).into())
                .with_endpoint(custom_vpn.peer.endpoint)
                .with_allowed_ips(custom_vpn.peer.allowed_ip.iter().cloned()),
        )
        .build()
        .await
        .map_err(TunnelError::GotaTunDevice)?;

    Ok(Frontend {
        custom_vpn: CustomVpn {
            inner_device,
            router_task,
        },
        tun_tx: TunTx::CustomVpn(Box::new(outer_ip_send)),
        tun_rx: TunRx::CustomVpn(outer_ip_recv),
        overhead,
    })
}

struct NatMappings {
    v4: Option<(Ipv4Addr, Ipv4Addr)>,
    v6: Option<(Ipv6Addr, Ipv6Addr)>,
}

/// Resolves and validates the address mappings needed for NAT rewriting
fn resolve_nat_mappings(
    custom_vpn: &CustomVpnConfig,
    outer_v4: Option<Ipv4Addr>,
    outer_v6: Option<Ipv6Addr>,
) -> Result<NatMappings, TunnelError> {
    let inner_v4 = custom_vpn.tunnel.ips.iter().find_map(|ip| match ip {
        &IpAddr::V4(ip) => Some(ip),
        IpAddr::V6(_) => None,
    });
    let inner_v6 = custom_vpn.tunnel.ips.iter().find_map(|ip| match ip {
        &IpAddr::V6(ip) => Some(ip),
        IpAddr::V4(_) => None,
    });

    let v4 = match (inner_v4, outer_v4) {
        (Some(inner), Some(outer)) => Some((inner, outer)),
        (Some(_), None) => {
            log::warn!(
                "Custom VPN inner tunnel has IPv4 but outer tunnel IPv4 is disabled; skipping IPv4"
            );
            None
        }
        _ => None,
    };

    let v6 = match (inner_v6, outer_v6) {
        (Some(inner), Some(outer)) => Some((inner, outer)),
        (Some(_), None) => {
            log::warn!(
                "Custom VPN inner tunnel has IPv6 but outer tunnel IPv6 is disabled; skipping IPv6"
            );
            None
        }
        _ => None,
    };

    if v4.is_none() && v6.is_none() {
        return Err(TunnelError::InvalidCustomVpnConfig(
            "no usable tunnel address family",
        ));
    }

    Ok(NatMappings { v4, v6 })
}

/// Calculates MTU and Overhead based on the IP address type
fn calculate_mtu_and_overhead(
    endpoint_ip: IpAddr,
    tun_dev: &GotaTunDevice,
) -> Result<(MtuWatcher, u16), TunnelError> {
    let overhead = match endpoint_ip {
        IpAddr::V4(..) => (Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD) as u16,
        IpAddr::V6(..) => (Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD) as u16,
    };

    let bridge_mtu = tun_dev
        .mtu()
        .increase(overhead)
        .ok_or(TunnelError::InvalidCustomVpnConfig("MTU overflow"))?;

    Ok((bridge_mtu, overhead))
}
