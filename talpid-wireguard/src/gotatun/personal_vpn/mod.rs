//! Personal VPN: a WireGuard tunnel of the user's own, carried inside the Mullvad tunnel.
//!
//! Traffic to the personal VPN's allowed IPs is NATed to the personal VPN tunnel address and
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
use merge::MergingIpRecv;
use nat::{NatIpRecv, NatIpSend};
use router::{SplitIpRecv, TunRxRouter, TunTxRouter};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::wireguard::PersonalVpnConfig;

mod merge;
mod nat;
mod packet;
mod router;
#[cfg(test)]
mod test_util;

/// Capacity of the channels between the router and the devices.
const CHANNEL_CAPACITY: usize = 4000;

/// The device that encrypts traffic to the personal VPN's allowed IPs.
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
/// device, or the personal VPN in front of it.
pub enum TunTx {
    Tun(GotaTunDevice),
    PersonalVpn(Box<OuterIpSend>),
}

/// What the outer device (the singlehop device, or the multihop exit device) reads from: the TUN
/// device, or the personal VPN in front of it.
pub enum TunRx {
    Tun(SourceFilter<GotaTunDevice>),
    PersonalVpn(OuterIpRecv),
}

impl IpSend for TunTx {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        match self {
            TunTx::Tun(tun) => tun.send(packet).await,
            TunTx::PersonalVpn(router) => router.send(packet).await,
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
            TunRx::PersonalVpn(merged) => merged.recv(pool).await.map(Either::Right),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        match self {
            TunRx::Tun(tun) => tun.mtu(),
            TunRx::PersonalVpn(merged) => merged.mtu(),
        }
    }
}

/// The parts of a personal VPN that are shared by the singlehop and multihop variants.
pub struct PersonalVpn {
    pub inner_device: InnerDevice,
    /// Splits reads from the TUN device between the inner and the outer device.
    router_task: AbortOnDrop,
}

impl PersonalVpn {
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
/// of the TUN device when there is a personal VPN.
pub struct Frontend {
    pub personal_vpn: PersonalVpn,
    pub tun_tx: TunTx,
    pub tun_rx: TunRx,
    /// Per-packet overhead added by the personal VPN WireGuard encapsulation.
    pub overhead: u16,
}

/// Build the personal VPN [`InnerDevice`], and the IO for the outer device.
///
/// Reads from the TUN device are split by destination: traffic to the personal VPN's allowed IPs
/// is NATed to the personal VPN tunnel address and encrypted by the inner device, whose output is
/// merged with all other traffic into the outer device. Decrypted packets from the outer device are
/// sent to the inner device if they come from the personal VPN endpoint, and to the TUN device
/// otherwise.
pub async fn build(
    personal_vpn: &PersonalVpnConfig,
    config: &Config,
    tun_dev: &GotaTunDevice,
) -> Result<Frontend, TunnelError> {
    let overhead = match personal_vpn.peer.endpoint.ip() {
        IpAddr::V4(..) => (Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD) as u16,
        IpAddr::V6(..) => (Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD) as u16,
    };
    let bridge_mtu = tun_dev
        .mtu()
        .increase(overhead)
        .ok_or(TunnelError::InvalidPersonalVpnConfig("MTU overflow"))?;

    let outer_v4 = config.tunnel_ipv4();
    let outer_v6 = config.tunnel_ipv6();

    // Channel bridge (inner device UDP <-> outer device TUN). The source addresses must be the
    // outer tunnel addresses, so that packets emitted by the inner device land on the outer device
    // with the correct source address.
    let (bridge_tun_tx, bridge_tun_rx, inner_udp) = new_udp_tun_channel(
        CHANNEL_CAPACITY,
        outer_v4.unwrap_or(Ipv4Addr::UNSPECIFIED),
        outer_v6.unwrap_or(Ipv6Addr::UNSPECIFIED),
        bridge_mtu,
    );

    // Split TUN reads by destination
    let tun_rx = SourceFilter::new(tun_dev.clone(), outer_v4, outer_v6);
    let mut router = TunRxRouter::new(tun_rx);
    let default_output = router.add_default_route(CHANNEL_CAPACITY);
    let alt_output = router.add_routes(&personal_vpn.peer.allowed_ip, CHANNEL_CAPACITY);
    let router_task = AbortOnDrop(tokio::spawn(router.run(PacketBufPool::new(100))));

    // Outer device IpRecv: merge direct traffic and traffic encrypted by the inner device
    let outer_ip_recv = MergingIpRecv::new(default_output, bridge_tun_rx, PacketBufPool::new(100));

    // Outer device IpSend: demux decrypted packets
    let outer_ip_send = TunTxRouter::new(
        bridge_tun_tx,
        tun_dev.clone(),
        personal_vpn.peer.endpoint,
        &personal_vpn.peer.allowed_ip,
    );

    // NAT: rewrite addresses between the inner and outer tunnel addresses.
    //
    // The inner tunnel may carry one IPv4, one IPv6, or both. Each family is enabled only when both
    // the inner tunnel and the outer Mullvad tunnel have an address for it. At least one family
    // must be usable.
    let inner_v4 = personal_vpn.tunnel.ips.iter().find_map(|ip| match ip {
        &IpAddr::V4(ip) => Some(ip),
        IpAddr::V6(_) => None,
    });
    let inner_v6 = personal_vpn.tunnel.ips.iter().find_map(|ip| match ip {
        &IpAddr::V6(ip) => Some(ip),
        IpAddr::V4(_) => None,
    });

    let nat_v4 = match (inner_v4, outer_v4) {
        (Some(inner), Some(outer)) => Some((inner, outer)),
        (Some(_), None) => {
            log::warn!(
                "Personal VPN inner tunnel has IPv4 but outer tunnel IPv4 is disabled; skipping IPv4"
            );
            None
        }
        _ => None,
    };
    let nat_v6 = match (inner_v6, outer_v6) {
        (Some(inner), Some(outer)) => Some((inner, outer)),
        (Some(_), None) => {
            log::warn!(
                "Personal VPN inner tunnel has IPv6 but outer tunnel IPv6 is disabled; skipping IPv6"
            );
            None
        }
        _ => None,
    };

    if nat_v4.is_none() && nat_v6.is_none() {
        return Err(TunnelError::InvalidPersonalVpnConfig(
            "no usable tunnel address family",
        ));
    }

    let mut tun_send = NatIpSend::new(tun_dev.clone());
    let mut alt_recv = NatIpRecv::new(alt_output);
    if let Some((inner_v4, outer_v4)) = nat_v4 {
        tun_send = tun_send.with_v4(inner_v4, outer_v4);
        alt_recv = alt_recv.with_v4(outer_v4, inner_v4);
    }
    if let Some((inner_v6, outer_v6)) = nat_v6 {
        tun_send = tun_send.with_v6(inner_v6, outer_v6);
        alt_recv = alt_recv.with_v6(outer_v6, inner_v6);
    }

    let inner_device = DeviceBuilder::new()
        .with_udp(inner_udp)
        .with_ip_pair(tun_send, alt_recv)
        .with_private_key(StaticSecret::from(
            personal_vpn.tunnel.private_key.to_bytes(),
        ))
        .with_peer(
            Peer::new((*personal_vpn.peer.public_key.as_bytes()).into())
                .with_endpoint(personal_vpn.peer.endpoint)
                .with_allowed_ips(personal_vpn.peer.allowed_ip.iter().cloned()),
        )
        .build()
        .await
        .map_err(TunnelError::GotaTunDevice)?;

    Ok(Frontend {
        personal_vpn: PersonalVpn {
            inner_device,
            router_task,
        },
        tun_tx: TunTx::PersonalVpn(Box::new(outer_ip_send)),
        tun_rx: TunRx::PersonalVpn(outer_ip_recv),
        overhead,
    })
}
