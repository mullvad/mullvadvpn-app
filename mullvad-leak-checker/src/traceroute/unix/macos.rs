use std::{ascii::escape_default, ffi::c_int, future::pending, io, net::IpAddr, num::NonZero};

use anyhow::{anyhow, bail, ensure, Context};
use nix::{
    net::if_::if_nametoindex,
    sys::socket::{setsockopt, sockopt::Ipv6Ttl},
};
use pnet_packet::{
    icmp::{self, time_exceeded::TimeExceededPacket, IcmpPacket, IcmpTypes},
    icmpv6::{Icmpv6Packet, Icmpv6Types},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    udp::UdpPacket,
    Packet,
};
use socket2::Socket;
use tokio::{
    select,
    time::{sleep_until, Instant},
};

use crate::{
    traceroute::{TracerouteOpt, RECV_GRACE_TIME},
    util::Ip,
    Interface, LeakInfo, LeakStatus,
};

use super::{parse_icmp_probe, too_small, AsyncIcmpSocket, Traceroute, PROBE_PAYLOAD};

pub struct TracerouteMacos;

pub struct AsyncIcmpSocketImpl {
    ip_version: Ip,
    inner: tokio::net::UdpSocket,
}

impl Traceroute for TracerouteMacos {
    type AsyncIcmpSocket = AsyncIcmpSocketImpl;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()> {
        // can't use the same method as desktop-linux here beacuse reasons
        bind_socket_to_interface(socket, interface, ip_version)
    }
}

impl AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket, ip_version: Ip) -> anyhow::Result<Self> {
        let std_socket = std::net::UdpSocket::from(socket);
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        Ok(AsyncIcmpSocketImpl {
            ip_version,
            inner: tokio_socket,
        })
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        match self.ip_version {
            Ip::V6(_) => {
                let ttl = ttl as c_int;
                setsockopt(&self.inner, Ipv6Ttl, &ttl).context("Failed to set TTL value for socket")
            }
            Ip::V4(..) => self
                .inner
                .set_ttl(ttl)
                .context("Failed to set TTL value for socket"),
        }
    }

    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize> {
        self.inner.send_to(packet, (destination.into(), 0)).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)> {
        self.inner
            .recv_from(buf)
            .await
            .map(|(n, source)| (n, source.ip()))
    }

    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
        recv_ttl_responses(self, opt).await
    }
}

fn bind_socket_to_interface(
    socket: &Socket,
    interface: &Interface,
    ip_version: Ip,
) -> anyhow::Result<()> {
    log::debug!("Binding socket to {interface:?}");

    let interface_index = match interface {
        &Interface::Index(index) => index,
        Interface::Name(interface) => if_nametoindex(interface.as_str())
            .map_err(anyhow::Error::from)
            .and_then(|code| NonZero::new(code).ok_or(anyhow!("Non-zero error code")))
            .context("Failed to get interface index")?,
    };

    match ip_version {
        Ip::V4(..) => socket.bind_device_by_index_v4(Some(interface_index))?,
        Ip::V6(..) => socket.bind_device_by_index_v6(Some(interface_index))?,
    }
    Ok(())
}

async fn recv_ttl_responses(
    socket: &impl AsyncIcmpSocket,
    opt: &TracerouteOpt,
) -> anyhow::Result<LeakStatus> {
    let interface = &opt.interface;

    // the list of node IP addresses from which we received a response to our probe packets.
    let mut reachable_nodes = vec![];

    // A time at which this function should exit. This is set when we receive the first probe
    // response, and allows us to wait a while to collect any additional probe responses before
    // returning.
    let mut timeout_at = None;

    let mut read_buf = vec![0u8; usize::from(u16::MAX)].into_boxed_slice();
    loop {
        let timer = async {
            match timeout_at {
                // resolve future at the timeout, if it's set
                Some(time) => sleep_until(time).await,

                // otherwise, never resolve
                None => pending().await,
            }
        };

        log::trace!("Reading from ICMP socket");

        let (n, source) = select! {
            result = socket.recv_from(&mut read_buf[..]) => result
                .context("Failed to read from raw socket")?,

            _timeout = timer => {
                return Ok(LeakStatus::LeakDetected(LeakInfo::NodeReachableOnInterface {
                    reachable_nodes,
                    interface: interface.clone(),
                }));
            }
        };

        let packet = &read_buf[..n];
        log::debug!("packet: {packet:02x?}");

        let result = match opt.destination {
            // Reading on an ICMPv6 raw socket returns ICMPv6 packets.
            IpAddr::V6(..) => parse_icmp_time_exceeded_raw(Ip::V6(packet)).map(|_| source),

            // Reading on an ICMPv4 raw socket returns whole IP packets.
            IpAddr::V4(..) => {
                parse_ipv4(packet).and_then(|ip_packet| parse_icmp4_time_exceeded(&ip_packet))
            }
        }
        .map_err(|e| anyhow!("Ignoring packet (len={n}, ip.src={source}): {e}"));

        match result {
            Ok(ip) => {
                log::debug!("Got a probe response, we are leaking!");
                timeout_at.get_or_insert_with(|| Instant::now() + RECV_GRACE_TIME);
                if !reachable_nodes.contains(&ip) {
                    reachable_nodes.push(ip);
                }
            }

            // an error means the packet wasn't the ICMP/TimeExceeded we're listening for.
            Err(e) => log::debug!("{e}"),
        }
    }
}

/// Try to parse the bytes as an IPv4 packet.
///
/// This only valdiates the IP header, not the payload.
fn parse_ipv4(packet: &[u8]) -> anyhow::Result<Ipv4Packet<'_>> {
    let packet = Ipv4Packet::new(packet).ok_or_else(too_small)?;
    let version = packet.get_version();
    if version != 4 {
        bail!("Invalid IP version: {version}")
    }
    Ok(packet)
}

/// Try to parse the bytes as an IPv4 or IPv6 packet.
///
/// This only valdiates the IP header, not the payload.
fn parse_ip(packet: &[u8]) -> anyhow::Result<Ip<Ipv4Packet<'_>, Ipv6Packet<'_>>> {
    let ipv4_packet = Ipv4Packet::new(packet).ok_or_else(too_small)?;

    // ipv4-packets are smaller than ipv6, so we use an Ipv4Packet to check the version.
    Ok(match ipv4_packet.get_version() {
        4 => Ip::V4(ipv4_packet),
        6 => {
            let ipv6_packet = Ipv6Packet::new(packet).ok_or_else(too_small)?;
            Ip::V6(ipv6_packet)
        }
        _ => bail!("Not a valid IP header"),
    })
}

/// Try to parse an [Ipv4Packet] as an ICMP/TimeExceeded response to a packet sent by
/// [send_udp_probes] or [send_icmp_probes]. If successful, returns the [Ipv4Addr] of the packet
/// source.
///
/// If the packet fails to parse, or is not a reply to a packet sent by us, this function returns
/// an error.
fn parse_icmp4_time_exceeded(ip_packet: &Ipv4Packet<'_>) -> anyhow::Result<IpAddr> {
    let ip_protocol = ip_packet.get_next_level_protocol();
    ensure!(ip_protocol == IpNextHeaderProtocols::Icmp, "Not ICMP");
    parse_icmp_time_exceeded_raw(Ip::V4(ip_packet.payload()))?;
    Ok(ip_packet.get_source().into())
}

/// Try to parse some bytes into an ICMP or ICMP6 TimeExceeded response to a probe packet sent by
/// [send_udp_probes] or [send_icmp_probes].
///
/// If the packet fails to parse, or is not a reply to a packet sent by us, this function returns
/// an error.
fn parse_icmp_time_exceeded_raw(ip_payload: Ip<&[u8], &[u8]>) -> anyhow::Result<()> {
    let icmpv4_packet;
    let icmpv6_packet;
    let icmp_packet: &[u8] = match ip_payload {
        Ip::V4(ipv4_payload) => {
            icmpv4_packet = IcmpPacket::new(ipv4_payload).ok_or(anyhow!("Too small"))?;

            let correct_type = icmpv4_packet.get_icmp_type() == IcmpTypes::TimeExceeded;
            ensure!(correct_type, "Not ICMP/TimeExceeded");

            icmpv4_packet.packet()
        }
        Ip::V6(ipv6_payload) => {
            icmpv6_packet = Icmpv6Packet::new(ipv6_payload).ok_or(anyhow!("Too small"))?;

            let correct_type = icmpv6_packet.get_icmpv6_type() == Icmpv6Types::TimeExceeded;
            ensure!(correct_type, "Not ICMP6/TimeExceeded");

            icmpv6_packet.packet()
        }
    };

    // TimeExceededPacket looks the same for both ICMP and ICMP6.
    let time_exceeded = TimeExceededPacket::new(icmp_packet).ok_or_else(too_small)?;
    ensure!(
        time_exceeded.get_icmp_code()
            == icmp::time_exceeded::IcmpCodes::TimeToLiveExceededInTransit,
        "Not TTL Exceeded",
    );

    let original_ip_packet = parse_ip(time_exceeded.payload()).context("ICMP-wrapped IP packet")?;

    let (original_ip_protocol, original_ip_payload) = match &original_ip_packet {
        Ip::V4(ipv4_packet) => (ipv4_packet.get_next_level_protocol(), ipv4_packet.payload()),
        Ip::V6(ipv6_packet) => (ipv6_packet.get_next_header(), ipv6_packet.payload()),
    };

    match original_ip_protocol {
        IpNextHeaderProtocols::Udp => {
            let original_udp_packet = UdpPacket::new(original_ip_payload).ok_or_else(too_small)?;

            // check if payload looks right
            // some network nodes will strip the payload, that's fine.
            if !original_udp_packet.payload().is_empty() {
                let udp_len = usize::from(original_udp_packet.get_length());
                let udp_payload = udp_len
                    .checked_sub(UdpPacket::minimum_packet_size())
                    .and_then(|len| original_udp_packet.payload().get(..len))
                    .ok_or(anyhow!("Invalid UDP length"))?;
                if udp_payload != PROBE_PAYLOAD {
                    let udp_payload: String = udp_payload
                        .iter()
                        .copied()
                        .flat_map(escape_default)
                        .map(char::from)
                        .collect();
                    bail!("Wrong UDP payload: {udp_payload:?}");
                }
            }

            Ok(())
        }

        IpNextHeaderProtocols::Icmp => parse_icmp_probe(Ip::V4(original_ip_payload)),

        IpNextHeaderProtocols::Icmpv6 => parse_icmp_probe(Ip::V6(original_ip_payload)),

        _ => bail!("Not UDP/ICMP"),
    }
}
