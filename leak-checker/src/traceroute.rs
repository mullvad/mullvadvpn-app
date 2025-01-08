use std::{
    ascii::escape_default,
    convert::Infallible,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::{Range, RangeFrom},
    time::Duration,
};

use anyhow::{anyhow, bail, ensure, Context};
use futures::{future::pending, select, stream, FutureExt, StreamExt, TryStreamExt};
use pnet_packet::{
    icmp::{self, time_exceeded::TimeExceededPacket, IcmpCode, IcmpPacket, IcmpTypes},
    icmpv6::{self, Icmpv6Code, Icmpv6Packet, Icmpv6Types},
    ip::IpNextHeaderProtocols as IpProtocol,
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    udp::UdpPacket,
    Packet,
};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::time::{sleep, timeout};

use crate::{Interface, LeakStatus};

mod platform;

use platform::{AsyncIcmpSocket, AsyncUdpSocket, Traceroute};

#[derive(Clone, clap::Args)]
pub struct TracerouteOpt {
    /// Try to bind to a specific interface
    #[clap(short, long)]
    pub interface: Interface,

    /// Destination IP of the probe packets
    #[clap(short, long)]
    pub destination: IpAddr,

    /// Avoid sending UDP probe packets to this port.
    #[clap(long, conflicts_with = "icmp")]
    pub exclude_port: Option<u16>,

    /// Send UDP probe packets only to this port, instead of the default ports.
    #[clap(long, conflicts_with = "icmp")]
    pub port: Option<u16>,

    /// Use ICMP-Echo for the probe packets instead of UDP.
    #[clap(long)]
    pub icmp: bool,
}

/// Type of the UDP payload of the probe packets
type ProbePayload = [u8; 32];

/// Value of the UDP payload of the probe packets
const PROBE_PAYLOAD: ProbePayload = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";

/// Timeout of the leak test as a whole. Should be more than [SEND_TIMEOUT] + [RECV_TIMEOUT].
const LEAK_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout of sending probe packets
const SEND_TIMEOUT: Duration = Duration::from_secs(1);

/// Timeout of receiving additional probe packets after the first one
const RECV_GRACE_TIME: Duration = Duration::from_millis(200);

/// Time in-between send of each probe packet.
const PROBE_INTERVAL: Duration = Duration::from_millis(100);

/// Default range of ports for the probe packets. Stolen from `traceroute`.
const DEFAULT_PORT_RANGE: RangeFrom<u16> = 33434..;

/// Range of TTL values for the probe packets.
const DEFAULT_TTL_RANGE: Range<u16> = 1..6;

/// [try_run_leak_test], but on an error, assume we aren't leaking.
pub async fn run_leak_test(opt: &TracerouteOpt) -> LeakStatus {
    try_run_leak_test(opt)
        .await
        .inspect_err(|e| log::debug!("Leak test errored, assuming no leak. {e:?}"))
        .unwrap_or(LeakStatus::NoLeak)
}

/// Run a traceroute-based leak test.
///
/// This test will try to create a socket and bind it to `interface`. Then it will send either UDP
/// or ICMP Echo packets to `destination` with very low TTL values. If any network nodes between
/// this one and `destination` see a packet with a TTL value of 0, they will _probably_ return an
/// ICMP/TimeExceeded response.
///
/// If we receive the response, we know the outgoing packet was NOT blocked by the firewall, and
/// therefore we are leaking. Since we set the TTL very low, this also means that in the event of a
/// leak, the packet will _probably_ not make it out of the users local network, e.g. the local
/// router will probably be the first node that gives a reply. Since the packet should not actually
/// reach `destination`, this testing method is resistant to being fingerprinted or censored.
///
/// This test needs a raw socket to be able to listen for the ICMP responses, therefore it requires
/// root/admin priviliges.
pub async fn try_run_leak_test(opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
    #[cfg(target_os = "android")]
    return try_run_leak_test_impl::<platform::android::TracerouteAndroid>(opt).await;

    #[cfg(target_os = "linux")]
    return try_run_leak_test_impl::<platform::linux::TracerouteLinux>(opt).await;

    #[cfg(target_os = "macos")]
    return try_run_leak_test_impl::<platform::macos::TracerouteMacos>(opt).await;

    #[cfg(target_os = "windows")]
    return platform::windows::traceroute_using_ping(opt).await;
}

/// IP version, v4 or v6, with some associated data.
#[derive(Clone, Copy)]
enum Ip<V4 = (), V6 = ()> {
    V4(V4),
    V6(V6),
}

async fn try_run_leak_test_impl<Impl: Traceroute>(
    opt: &TracerouteOpt,
) -> anyhow::Result<LeakStatus> {
    // create the socket used for receiving the ICMP/TimeExceeded responses

    // don't ask me why, but this is how it must be.
    let icmp_socket_type = if cfg!(target_os = "windows") {
        Type::RAW
    } else {
        Type::DGRAM
    };

    let (ip_version, domain, icmp_protocol) = match opt.destination {
        IpAddr::V4(..) => (Ip::V4(()), Domain::IPV4, Protocol::ICMPV4),
        IpAddr::V6(..) => (Ip::V6(()), Domain::IPV6, Protocol::ICMPV6),
    };

    let icmp_socket = Socket::new(domain, icmp_socket_type, Some(icmp_protocol))
        .context("Failed to open ICMP socket")?;

    icmp_socket
        .set_nonblocking(true)
        .context("Failed to set icmp_socket to nonblocking")?;

    Impl::bind_socket_to_interface(&icmp_socket, &opt.interface, ip_version)?;
    Impl::configure_icmp_socket(&icmp_socket, opt)?;

    let icmp_socket = Impl::AsyncIcmpSocket::from_socket2(icmp_socket);

    let send_probes = async {
        if opt.icmp {
            send_icmp_probes::<Impl>(opt, &icmp_socket).await?;
        } else {
            // create the socket used for sending the UDP probing packets
            let udp_socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
                .context("Failed to open UDP socket")?;

            Impl::bind_socket_to_interface(&udp_socket, &opt.interface, ip_version)
                .context("Failed to bind UDP socket to interface")?;

            udp_socket
                .set_nonblocking(true)
                .context("Failed to set udp_socket to nonblocking")?;

            let mut udp_socket = Impl::AsyncUdpSocket::from_socket2(udp_socket);

            send_udp_probes(opt, &mut udp_socket).await?;
        }

        anyhow::Ok(())
    };

    let send_probes = async {
        timeout(SEND_TIMEOUT, send_probes)
            .await
            .map_err(|_timeout| anyhow!("Timed out while trying to send probe packet"))??;
        Ok(pending::<Infallible>().await)
    };

    let recv_probe_responses = icmp_socket.recv_ttl_responses(opt);

    // wait until either future returns, or the timeout is reached
    // friendship ended with tokio::select. now futures::select is my best friend!
    let leak_status = select! {
        result = recv_probe_responses.fuse() => result?,
        Err(e) = send_probes.fuse() => return Err(e),
        _ = sleep(LEAK_TIMEOUT).fuse() => LeakStatus::NoLeak,
    };

    Ok(leak_status)
}

/// Send ICMP/Echo packets with a very low TTL to `opt.destination`.
///
/// Use [AsyncIcmpSocket::recv_ttl_responses] to receive replies.
async fn send_icmp_probes<Impl: Traceroute>(
    opt: &TracerouteOpt,
    socket: &impl AsyncIcmpSocket,
) -> anyhow::Result<()> {
    for ttl in DEFAULT_TTL_RANGE {
        log::debug!("sending probe packet (ttl={ttl})");

        socket
            .set_ttl(ttl.into())
            .context("Failed to set TTL on socket")?;

        // the first packet will sometimes get dropped on MacOS, thus we send two packets
        let number_of_sends = if cfg!(target_os = "macos") { 2 } else { 1 };

        // construct ICMP/ICMP6 echo request packet
        let mut packet_v4;
        let mut packet_v6;
        let packet_bytes;
        const ECHO_REQUEST_HEADER_LEN: usize = 8;
        match opt.destination {
            IpAddr::V4(..) => {
                let echo = icmp::echo_request::EchoRequest {
                    icmp_type: IcmpTypes::EchoRequest,
                    icmp_code: IcmpCode(0),
                    checksum: 0,
                    identifier: 1,
                    sequence_number: 1,
                    payload: PROBE_PAYLOAD.to_vec(),
                };

                let len = ECHO_REQUEST_HEADER_LEN + PROBE_PAYLOAD.len();
                packet_v4 =
                    icmp::echo_request::MutableEchoRequestPacket::owned(vec![0u8; len]).unwrap();
                packet_v4.populate(&echo);
                packet_v4.set_checksum(icmp::checksum(
                    &icmp::IcmpPacket::new(packet_v4.packet()).unwrap(),
                ));
                packet_bytes = packet_v4.packet();
            }
            IpAddr::V6(destination) => {
                let IpAddr::V6(source) = Impl::get_interface_ip(&opt.interface, Ip::V6(()))? else {
                    bail!("Tried to send IPv6 on IPv4 interface");
                };

                let echo = icmpv6::echo_request::EchoRequest {
                    icmpv6_type: Icmpv6Types::EchoRequest,
                    icmpv6_code: Icmpv6Code(0),
                    checksum: 0,
                    identifier: 1,
                    sequence_number: 1,
                    payload: PROBE_PAYLOAD.to_vec(),
                };

                let len = ECHO_REQUEST_HEADER_LEN + PROBE_PAYLOAD.len();
                packet_v6 =
                    icmpv6::echo_request::MutableEchoRequestPacket::owned(vec![0u8; len]).unwrap();
                packet_v6.populate(&echo);
                packet_v6.set_checksum(icmpv6::checksum(
                    &icmpv6::Icmpv6Packet::new(packet_v6.packet()).unwrap(),
                    &source,
                    &destination,
                ));
                packet_bytes = packet_v6.packet();
            }
        }

        let result: io::Result<()> = stream::iter(0..number_of_sends)
            // call `send_to` `number_of_sends` times
            .then(|_| socket.send_to(packet_bytes, opt.destination))
            .map_ok(drop)
            .try_collect() // abort on the first error
            .await;

        // if there was an error, handle it, otherwise continue probing.
        let Err(e) = result else {
            sleep(PROBE_INTERVAL).await;
            continue;
        };

        match e.kind() {
            io::ErrorKind::PermissionDenied | io::ErrorKind::ConnectionRefused => {
                // Linux returns one of these errors if our packet was rejected by nftables.
                log::debug!("send_to failed, was probably caught by firewall");
                break;
            }
            _ => return Err(e).context("Failed to send packet")?,
        }
    }

    Ok(())
}

/// Send UDP packets with a very low TTL to `opt.destination`.
///
/// Use [Impl::recv_ttl_responses] to receive replies.
async fn send_udp_probes(
    opt: &TracerouteOpt,
    socket: &mut impl AsyncUdpSocket,
) -> anyhow::Result<()> {
    // ensure we don't send anything to `opt.exclude_port`
    let ports = DEFAULT_PORT_RANGE
        // skip the excluded port
        .filter(|&p| Some(p) != opt.exclude_port)
        // `opt.port` overrides the default port range
        .map(|port| opt.port.unwrap_or(port));

    for (port, ttl) in ports.zip(DEFAULT_TTL_RANGE) {
        log::debug!("sending probe packet (ttl={ttl})");

        socket
            .set_ttl(ttl.into())
            .context("Failed to set TTL on socket")?;

        // the first packet will sometimes get dropped on MacOS, thus we send two packets
        let number_of_sends = if cfg!(target_os = "macos") { 2 } else { 1 };

        let result: io::Result<()> = stream::iter(0..number_of_sends)
            // call `send_to` `number_of_sends` times
            .then(|_| socket.send_to(&PROBE_PAYLOAD, (opt.destination, port)))
            .map_ok(drop)
            .try_collect() // abort on the first error
            .await;

        let Err(e) = result else { continue };
        match e.kind() {
            io::ErrorKind::PermissionDenied => {
                // Linux returns this error if our packet was rejected by nftables.
                log::debug!("send_to failed with 'permission denied'");
            }
            _ => return Err(e).context("Failed to send packet")?,
        }
    }

    Ok(())
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

/// Try to parse the bytes as an IPv4 packet.
///
/// This only valdiates the IPv4 header, not the payload.
fn parse_ipv4(packet: &[u8]) -> anyhow::Result<Ipv4Packet<'_>> {
    let ip_packet = Ipv4Packet::new(packet).ok_or_else(too_small)?;
    ensure!(ip_packet.get_version() == 4, "Not IPv4");
    anyhow::Ok(ip_packet)
}

/// Try to parse the bytes as an IPv6 packet.
///
/// This only valdiates the IPv6 header, not the payload.
fn parse_ipv6(packet: &[u8]) -> anyhow::Result<Ipv6Packet<'_>> {
    let ip_packet = Ipv6Packet::new(packet).ok_or_else(too_small)?;
    ensure!(ip_packet.get_version() == 6, "Not IPv6");
    anyhow::Ok(ip_packet)
}

/// Try to parse an [Ipv4Packet] as an ICMP/TimeExceeded response to a packet sent by
/// [send_udp_probes] or [send_icmp_probes]. If successful, returns the [Ipv4Addr] of the packet
/// source.
///
/// If the packet fails to parse, or is not a reply to a packet sent by us, this function returns
/// an error.
fn parse_icmp4_time_exceeded(ip_packet: &Ipv4Packet<'_>) -> anyhow::Result<Ipv4Addr> {
    let ip_protocol = ip_packet.get_next_level_protocol();
    ensure!(ip_protocol == IpProtocol::Icmp, "Not ICMP");
    parse_icmp_time_exceeded_raw(Ip::V4(ip_packet.payload()))?;
    Ok(ip_packet.get_source())
}

/// Try to parse an [Ipv6Packet] as an ICMP6/TimeExceeded response to a packet sent by
/// [send_udp_probes] or [send_icmp_probes]. If successful, returns the [Ipv6Addr] of the packet
/// source.
///
/// If the packet fails to parse, or is not a reply to a packet sent by us, this function returns
/// an error.
fn parse_icmp6_time_exceeded(ip_packet: &Ipv6Packet<'_>) -> anyhow::Result<Ipv6Addr> {
    let ip_protocol = ip_packet.get_next_header();
    ensure!(ip_protocol == IpProtocol::Icmpv6, "Not ICMP6");
    parse_icmp_time_exceeded_raw(Ip::V6(ip_packet.payload()))?;
    Ok(ip_packet.get_source())
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
        IpProtocol::Udp => {
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

        IpProtocol::Icmpv6 => {
            let original_icmp_packet =
                icmpv6::echo_request::EchoRequestPacket::new(original_ip_payload)
                    .ok_or_else(too_small)?;

            ensure!(
                original_icmp_packet.get_icmpv6_type() == Icmpv6Types::EchoRequest,
                "Not ICMP6/EchoRequest"
            );

            // check if payload looks right
            // some network nodes will strip the payload, that's fine.
            let echo_payload = original_icmp_packet.payload();
            if !echo_payload.is_empty() && !echo_payload.starts_with(&PROBE_PAYLOAD) {
                let echo_payload: String = echo_payload
                    .iter()
                    .copied()
                    .flat_map(escape_default)
                    .map(char::from)
                    .collect();
                bail!("Wrong ICMP6/Echo payload: {echo_payload:?}");
            }

            Ok(())
        }

        IpProtocol::Icmp => {
            let original_icmp_packet =
                icmp::echo_request::EchoRequestPacket::new(original_ip_payload)
                    .ok_or_else(too_small)?;

            ensure!(
                original_icmp_packet.get_icmp_type() == IcmpTypes::EchoRequest,
                "Not ICMP/EchoRequest"
            );

            // check if payload looks right
            // some network nodes will strip the payload, that's fine.
            let echo_payload = original_icmp_packet.payload();
            if !echo_payload.is_empty() && !echo_payload.starts_with(&PROBE_PAYLOAD) {
                let echo_payload: String = echo_payload
                    .iter()
                    .copied()
                    .flat_map(escape_default)
                    .map(char::from)
                    .collect();
                bail!("Wrong ICMP/Echo payload: {echo_payload:?}");
            }

            Ok(())
        }

        _ => bail!("Not UDP/ICMP"),
    }
}

fn parse_icmp_echo_raw(icmp_bytes: Ip<&[u8], &[u8]>) -> anyhow::Result<()> {
    let echo_packet_v4;
    let echo_packet_v6;
    let echo_payload = match icmp_bytes {
        Ip::V4(icmpv4_bytes) => {
            echo_packet_v4 =
                icmp::echo_request::EchoRequestPacket::new(icmpv4_bytes).ok_or_else(too_small)?;

            ensure!(
                echo_packet_v4.get_icmp_type() == IcmpTypes::EchoRequest,
                "Not ICMP/EchoRequest"
            );

            echo_packet_v4.payload()
        }
        Ip::V6(icmpv6_bytes) => {
            echo_packet_v6 =
                icmpv6::echo_request::EchoRequestPacket::new(icmpv6_bytes).ok_or_else(too_small)?;

            ensure!(
                echo_packet_v6.get_icmpv6_type() == Icmpv6Types::EchoRequest,
                "Not ICMP6/EchoRequest"
            );

            echo_packet_v6.payload()
        }
    };

    // check if payload looks right
    // some network nodes will strip the payload.
    // some network nodes will add a bunch of zeros at the end.
    if !echo_payload.is_empty() && !echo_payload.starts_with(&PROBE_PAYLOAD) {
        let echo_payload: String = echo_payload
            .iter()
            .copied()
            .flat_map(escape_default)
            .map(char::from)
            .collect();
        bail!("Wrong ICMP6/Echo payload: {echo_payload:?}");
    }

    Ok(())
}

fn too_small() -> anyhow::Error {
    anyhow!("Too small")
}
