use std::{
    ascii::escape_default,
    convert::Infallible,
    io,
    net::{IpAddr, SocketAddr},
    ops::RangeFrom,
    os::fd::{FromRawFd, IntoRawFd},
};

use crate::{
    Interface, LeakStatus,
    traceroute::{DEFAULT_TTL_RANGE, LEAK_TIMEOUT, PROBE_INTERVAL, SEND_TIMEOUT, TracerouteOpt},
    util::{Ip, get_interface_ip},
};

use anyhow::{Context, anyhow, bail, ensure};
use futures::{FutureExt, StreamExt, TryStreamExt, future::pending, select, stream};
use pnet_packet::{
    Packet,
    icmp::{self, IcmpCode, IcmpTypes},
    icmpv6::{self, Icmpv6Code, Icmpv6Types},
};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::time::{sleep, timeout};

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "linux")]
pub mod linux;

/// Helper module for Linux-like OSes (Linux, Android).
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux_like;

#[cfg(target_os = "macos")]
pub mod macos;

/// Type of the UDP payload of the probe packets
type ProbePayload = [u8; 32];

/// Value of the UDP payload of the probe packets
const PROBE_PAYLOAD: ProbePayload = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";

/// Default range of ports for the UDP probe packets. Stolen from `traceroute`.
const DEFAULT_PORT_RANGE: RangeFrom<u16> = 33434..;

/// Private trait that let's us define the platform-specific implementations and types required for
/// tracerouting.
pub trait Traceroute {
    type AsyncIcmpSocket: AsyncIcmpSocket;

    fn bind_socket_to_interface(
        socket: &socket2::Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()>;
}

pub trait AsyncIcmpSocket: Sized {
    fn from_socket2(socket: socket2::Socket, ip_version: Ip) -> anyhow::Result<Self>;

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()>;

    /// Send an ICMP packet to the destination.
    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize>;

    /// Receive an ICMP packet.
    #[cfg_attr(any(target_os = "linux", target_os = "android"), expect(dead_code))]
    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)>;

    /// Try to read ICMP/TimeExceeded error packets to see if probe packets leaked.
    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus>;
}

struct AsyncUdpSocket(tokio::net::UdpSocket);

pub async fn try_run_leak_test<Impl: Traceroute>(
    opt: &TracerouteOpt,
) -> anyhow::Result<LeakStatus> {
    // If we ever change this to support windows, this probably needs to be Type::DGRAM.
    let icmp_socket_type = Type::RAW;

    let (ip_version, domain, icmp_protocol) = match opt.destination {
        IpAddr::V4(..) => (Ip::v4(), Domain::IPV4, Protocol::ICMPV4),
        IpAddr::V6(..) => (Ip::v6(), Domain::IPV6, Protocol::ICMPV6),
    };

    // create the socket used for receiving the ICMP/TimeExceeded responses
    let icmp_socket = Socket::new(domain, icmp_socket_type, Some(icmp_protocol))
        .context("Failed to open ICMP socket")?;

    icmp_socket
        .set_nonblocking(true)
        .context("Failed to set icmp_socket to nonblocking")?;

    Impl::bind_socket_to_interface(&icmp_socket, &opt.interface, ip_version)?;

    let icmp_socket = Impl::AsyncIcmpSocket::from_socket2(icmp_socket, ip_version)?;

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

            let mut udp_socket = AsyncUdpSocket::from_socket2(udp_socket);

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

async fn send_icmp_probes<Impl: Traceroute>(
    opt: &TracerouteOpt,
    socket: &impl AsyncIcmpSocket,
) -> anyhow::Result<()> {
    log::debug!("Sending probe packets (ttl={DEFAULT_TTL_RANGE:?})");
    for ttl in DEFAULT_TTL_RANGE {
        log::trace!("Sending probe packet (ttl={ttl})");

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
                let IpAddr::V6(source) = get_interface_ip(&opt.interface, Ip::V6(()))? else {
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

impl AsyncUdpSocket {
    pub fn from_socket2(socket: socket2::Socket) -> Self {
        // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
        // SAFETY: `into_raw_fd()` consumes the socket and returns an owned & open file descriptor.
        let udp_socket = unsafe { std::net::UdpSocket::from_raw_fd(socket.into_raw_fd()) };
        let udp_socket = tokio::net::UdpSocket::from_std(udp_socket).unwrap();
        AsyncUdpSocket(udp_socket)
    }

    pub fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for UDP socket")
    }

    pub async fn send_to(
        &self,
        packet: &[u8],
        destination: impl Into<SocketAddr>,
    ) -> std::io::Result<usize> {
        self.0.send_to(packet, destination.into()).await
    }
}

/// Send ICMP/Echo packets with a very low TTL to `opt.destination`.
///
/// Use [AsyncIcmpSocket::recv_ttl_responses] to receive replies.
/// Send UDP packets with a very low TTL to `opt.destination`.
///
/// Use [Impl::recv_ttl_responses] to receive replies.
async fn send_udp_probes(opt: &TracerouteOpt, socket: &mut AsyncUdpSocket) -> anyhow::Result<()> {
    // ensure we don't send anything to `opt.exclude_port`
    let ports = DEFAULT_PORT_RANGE
        // skip the excluded port
        .filter(|&p| Some(p) != opt.exclude_port)
        // `opt.port` overrides the default port range
        .map(|port| opt.port.unwrap_or(port));

    log::debug!("Sending probe packets (ttl={DEFAULT_TTL_RANGE:?})");
    for (port, ttl) in ports.zip(DEFAULT_TTL_RANGE) {
        log::trace!("Sending probe packet (ttl={ttl})");

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

/// Try to parse bytes as an ICMP/ICMP6 Echo Request matching the probe packets send by
/// [send_icmp_probes].
fn parse_icmp_probe(icmp_bytes: Ip<&[u8], &[u8]>) -> anyhow::Result<()> {
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
