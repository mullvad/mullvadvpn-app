use std::{
    ffi::c_int,
    io::{self, IoSliceMut},
    net::IpAddr,
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

use anyhow::{Context, anyhow};
use nix::{
    cmsg_space,
    errno::Errno,
    libc,
    sys::socket::{
        ControlMessageOwned, MsgFlags, SockaddrIn, SockaddrIn6, SockaddrLike, recvmsg, setsockopt,
        sockopt::{Ipv4RecvErr, Ipv4Ttl, Ipv6RecvErr, Ipv6Ttl},
    },
};
use pnet_packet::{
    icmp::{IcmpCode, IcmpType, IcmpTypes, time_exceeded::IcmpCodes},
    icmpv6::{Icmpv6Code, Icmpv6Type, Icmpv6Types},
};
use socket2::Socket;
use tokio::time::{Instant, sleep};

use crate::{
    Interface, LeakInfo, LeakStatus,
    traceroute::{RECV_GRACE_TIME, TracerouteOpt, unix::parse_icmp_probe},
    util::Ip,
};

pub struct AsyncIcmpSocketImpl {
    ip_version: Ip,
    inner: tokio::net::UdpSocket,
}

impl super::AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket, ip_version: Ip) -> anyhow::Result<Self> {
        // IP_RECVERR tells Linux to pass any error packets received over ICMP to us through `recvmsg` control messages.
        match ip_version {
            Ip::V4(_) => {
                setsockopt(&socket, Ipv4RecvErr, &true).context("Failed to set IP_RECVERR")?
            }
            Ip::V6(_) => {
                setsockopt(&socket, Ipv6RecvErr, &true).context("Failed to set IPV6_RECVERR")?
            }
        }

        let std_socket = std::net::UdpSocket::from(socket);
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        Ok(AsyncIcmpSocketImpl {
            ip_version,
            inner: tokio_socket,
        })
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        let ttl = ttl as c_int;
        match self.ip_version {
            Ip::V4(_) => setsockopt(&self.inner, Ipv4Ttl, &ttl),
            Ip::V6(_) => setsockopt(&self.inner, Ipv6Ttl, &ttl),
        }
        .context("Failed to set TTL value for socket")
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
        recv_ttl_responses(opt.destination, &opt.interface, &self.inner).await
    }
}

/// Try to read ICMP/TimeExceeded error packets from an ICMP socket.
///
/// This method does not require root, but only works on Linux (including Android).
async fn recv_ttl_responses(
    destination: IpAddr,
    interface: &Interface,
    socket: &impl AsRawFd,
) -> anyhow::Result<LeakStatus> {
    // the list of node IP addresses from which we received a response to our probe packets.
    let mut reachable_nodes = vec![];

    // A time at which this function should exit. This is set when we receive the first probe
    // response, and allows us to wait a while to collect any additional probe responses before
    // returning.
    let mut timeout_at = None;

    // Allocate buffer for receiving packets.
    let mut recv_buf = vec![0u8; usize::from(u16::MAX)].into_boxed_slice();
    let mut io_vec = [IoSliceMut::new(&mut recv_buf)];

    // Allocate space for EHOSTUNREACH errors caused by ICMP/TimeExceeded packets.
    let mut control_buf = match destination {
        // This is the size of ControlMessageOwned::Ipv4RecvErr(sock_extended_err, sockaddr_in).
        IpAddr::V4(..) => cmsg_space!(libc::sock_extended_err, libc::sockaddr_in),

        // This is the size of ControlMessageOwned::Ipv6RecvErr(sock_extended_err, sockaddr_in6).
        IpAddr::V6(..) => cmsg_space!(libc::sock_extended_err, libc::sockaddr_in6),
    };

    'outer: loop {
        log::trace!("Reading from ICMP socket");

        // Call recvmsg in a loop
        let recv_packet = loop {
            if let Some(timeout_at) = timeout_at
                && Instant::now() >= timeout_at
            {
                break 'outer;
            }

            let recv_packet = match destination {
                IpAddr::V4(..) => recvmsg_with_control_message::<SockaddrIn>(
                    socket.as_raw_fd(),
                    &mut io_vec,
                    &mut control_buf,
                )?
                .map(|packet| packet.map_source_addr(|a| IpAddr::from(a.ip()))),
                IpAddr::V6(..) => recvmsg_with_control_message::<SockaddrIn6>(
                    socket.as_raw_fd(),
                    &mut io_vec,
                    &mut control_buf,
                )?
                .map(|packet| packet.map_source_addr(|a| IpAddr::from(a.ip()))),
            };

            let Some(recv_packet) = recv_packet else {
                // poor-mans async IO :'(
                sleep(Duration::from_millis(10)).await;
                continue;
            };

            break recv_packet;
        };

        let RecvPacket {
            source_addr,
            packet,
            control_message,
        } = recv_packet;

        macro_rules! skip_if {
            ($skip_condition:expr, $note:expr) => {{
                if $skip_condition {
                    log::debug!("Ignoring received message: {}", $note);
                    continue 'outer;
                }
            }};
        }

        // NOTE: This should be the IP destination of our ping packets. That does NOT mean the
        // packets reached the destination. Instead, if we see an EHOSTUNREACH control message,
        // it means the packets was instead dropped along the way. Seeing this address helps us
        // identify that this is a response to the ping we sent.
        skip_if!(source_addr != destination, "Unknown source");

        let error_source = match control_message {
            ControlMessageOwned::Ipv4RecvErr(socket_error, source_addr) => {
                let libc::sock_extended_err {
                    ee_errno,  // Error Number: Should be EHOSTUNREACH
                    ee_origin, // Error Origin: 2 = Icmp
                    ee_type,   // ICMP Type: 11 = ICMP/TimeExceeded.
                    ee_code,   // ICMP Code. 0 = TTL exceeded in transit.
                    ..
                } = socket_error;

                let errno = Errno::from_raw(ee_errno as i32);
                skip_if!(errno != Errno::EHOSTUNREACH, "Unexpected errno");
                skip_if!(
                    ee_origin != nix::libc::SO_EE_ORIGIN_ICMP,
                    "Unexpected origin"
                );

                let icmp_type = IcmpType::new(ee_type);
                skip_if!(icmp_type != IcmpTypes::TimeExceeded, "Unexpected ICMP type");

                let icmp_code = IcmpCode::new(ee_code);
                skip_if!(
                    icmp_code != IcmpCodes::TimeToLiveExceededInTransit,
                    "Unexpected ICMP code"
                );

                // NOTE: This is the IP of the node that dropped the packet due to TTL exceeded.
                let error_source = SockaddrIn::from(source_addr.unwrap());
                log::debug!("addr: {error_source}");

                // Ensure that this is the original Echo packet that we sent.
                skip_if!(
                    parse_icmp_probe(Ip::V4(packet)).is_err(),
                    "Not a response to us"
                );

                IpAddr::from(error_source.ip())
            }
            ControlMessageOwned::Ipv6RecvErr(socket_error, source_addr) => {
                let libc::sock_extended_err {
                    ee_errno,  // Error Number: Should be EHOSTUNREACH
                    ee_origin, // Error Origin: 3 = Icmp6.
                    ee_type,   // ICMP Type: 3 = ICMP6/TimeExceeded
                    ee_code,   // ICMP Code. 0 = TTL exceeded in transit.
                    ..
                } = socket_error;

                let errno = Errno::from_raw(ee_errno as i32);
                skip_if!(errno != Errno::EHOSTUNREACH, "Unexpected errno");
                skip_if!(
                    ee_origin != nix::libc::SO_EE_ORIGIN_ICMP6,
                    "Unexpected origin"
                );

                let icmp_type = Icmpv6Type::new(ee_type);
                skip_if!(
                    icmp_type != Icmpv6Types::TimeExceeded,
                    "Unexpected ICMP type"
                );

                let icmp_code = Icmpv6Code::new(ee_code);
                skip_if!(icmp_code != Icmpv6Code::new(0), "Unexpected ICMP code");

                // NOTE: This is the IP of the node that dropped the packet due to TTL exceeded.
                let error_source = SockaddrIn6::from(source_addr.unwrap());
                log::debug!("addr: {error_source}");

                // Ensure that this is the original Echo packet that we sent.
                skip_if!(
                    parse_icmp_probe(Ip::V6(packet)).is_err(),
                    "Not a response to us"
                );

                IpAddr::from(error_source.ip())
            }
            other_message => {
                log::debug!("Unhandled control message: {other_message:?}");
                continue 'outer;
            }
        };

        log::debug!("Got a probe response, we are leaking!");
        timeout_at.get_or_insert_with(|| Instant::now() + RECV_GRACE_TIME);
        reachable_nodes.push(error_source);
    }

    debug_assert!(!reachable_nodes.is_empty());

    Ok(LeakStatus::LeakDetected(
        LeakInfo::NodeReachableOnInterface {
            reachable_nodes,
            interface: interface.clone(),
        },
    ))
}

struct RecvPacket<'a, S> {
    pub source_addr: S,
    pub packet: &'a [u8],
    pub control_message: ControlMessageOwned,
}

impl<'a, S> RecvPacket<'a, S> {
    /// Convert the type of [RecvPacket::source_addr], e.g. from [SockaddrIn6] to [IpAddr].
    fn map_source_addr<T>(self, f: impl FnOnce(S) -> T) -> RecvPacket<'a, T> {
        RecvPacket {
            source_addr: f(self.source_addr),
            packet: self.packet,
            control_message: self.control_message,
        }
    }
}

/// Call recvmsg and expect exactly one control message.
///
/// See [ControlMessageOwned] for details on control messages.
/// Returns `Ok(None)` on `EWOULDBLOCK`, or if recvmsg returns no control message.
fn recvmsg_with_control_message<'a, S: SockaddrLike + Copy>(
    socket: RawFd,
    io_vec: &'a mut [IoSliceMut<'_>; 1],
    control_buf: &mut [u8],
) -> anyhow::Result<Option<RecvPacket<'a, S>>> {
    // MSG_ERRQUEUE asks linux to tell us if we get any ICMP error replies to
    // our Echo packets.
    let flags = MsgFlags::MSG_ERRQUEUE;

    let result = recvmsg::<S>(socket.as_raw_fd(), io_vec, Some(control_buf), flags);

    let recv = match result {
        Ok(recv) => recv,
        Err(Errno::EWOULDBLOCK) => return Ok(None),
        Err(e) => return Err(anyhow!("Failed to read from socket: {e}")),
    };

    let source_addr = recv.address.unwrap();

    let mut control_messages = recv
        .cmsgs()
        .context("Failed to decode cmsgs from recvmsg")?;

    let Some(control_message) = control_messages.next() else {
        // We're looking for EHOSTUNREACH errors. No errors means skip.
        log::debug!("Skipping recvmsg that produced no control messages.");
        return Ok(None);
    };

    let Some(packet) = recv.iovs().next() else {
        log::debug!("Skipping recvmsg that produced no data.");
        return Ok(None);
    };

    Ok(Some(RecvPacket {
        source_addr,
        packet,
        control_message,
    }))
}
