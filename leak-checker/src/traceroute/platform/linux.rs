use std::io::{self, IoSliceMut};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::{net::IpAddr, time::Duration};

use anyhow::{bail, Context};
use nix::errno::Errno;
use nix::sys::socket::sockopt::Ipv4RecvErr;
use nix::sys::socket::{setsockopt, ControlMessageOwned, MsgFlags, SockaddrIn};
use nix::{cmsg_space, libc};
use pnet_packet::icmp::time_exceeded::IcmpCodes;
use pnet_packet::icmp::IcmpTypes;
use pnet_packet::icmp::{IcmpCode, IcmpType};
use socket2::Socket;
use tokio::time::{sleep, Instant};

use crate::traceroute::{parse_icmp_echo_raw, TracerouteOpt, RECV_GRACE_TIME};
use crate::{Interface, LeakInfo, LeakStatus};

use super::{unix, AsyncIcmpSocket, Traceroute};

pub struct TracerouteLinux;

pub struct AsyncIcmpSocketImpl(tokio::net::UdpSocket);

impl Traceroute for TracerouteLinux {
    type AsyncIcmpSocket = AsyncIcmpSocketImpl;
    type AsyncUdpSocket = unix::AsyncUdpSocketUnix;

    fn bind_socket_to_interface(socket: &Socket, interface: &Interface) -> anyhow::Result<()> {
        bind_socket_to_interface(socket, interface)
    }

    fn get_interface_ip(interface: &Interface) -> anyhow::Result<IpAddr> {
        super::unix::get_interface_ip(interface)
    }

    fn configure_icmp_socket(socket: &socket2::Socket, _opt: &TracerouteOpt) -> anyhow::Result<()> {
        // IP_RECVERR tells Linux to pass any error packets received over ICMP to us through `recvmsg` control messages.
        setsockopt(socket, Ipv4RecvErr, &true).context("Failed to set IP_RECVERR")
    }
}

impl AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket) -> Self {
        let raw_socket = socket.into_raw_fd();
        let std_socket = unsafe { std::net::UdpSocket::from_raw_fd(raw_socket) };
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        AsyncIcmpSocketImpl(tokio_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for socket")
    }

    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize> {
        self.0.send_to(packet, (destination.into(), 0)).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)> {
        self.0
            .recv_from(buf)
            .await
            .map(|(n, source)| (n, source.ip()))
    }

    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
        recv_ttl_responses(opt.destination, &opt.interface, &self.0).await
    }
}

fn bind_socket_to_interface(socket: &Socket, interface: &Interface) -> anyhow::Result<()> {
    log::info!("Binding socket to {interface:?}");

    let Interface::Name(interface) = interface;

    socket
        .bind_device(Some(interface.as_bytes()))
        .context("Failed to bind socket to interface")?;

    Ok(())
}

/// Try to read ICMP/TimeExceeded error packets from an ICMP socket.
///
/// This method does not require root, but only works on Linux (including Android).
// TODO: double check if this works on MacOS
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
    // This is the size of ControlMessageOwned::Ipv4RecvErr(sock_extended_err, sockaddr_in).
    // FIXME: sockaddr_in only works for ipv4
    let mut control_buf = cmsg_space!(libc::sock_extended_err, libc::sockaddr_in);

    'outer: loop {
        log::debug!("Reading from ICMP socket");

        let recv = loop {
            if let Some(timeout_at) = timeout_at {
                if Instant::now() >= timeout_at {
                    break 'outer;
                }
            }

            match nix::sys::socket::recvmsg::<SockaddrIn>(
                socket.as_raw_fd(),
                &mut io_vec,
                Some(&mut control_buf),
                // NOTE: MSG_ERRQUEUE asks linux to tell us if we get any ICMP error replies to
                // our Echo packets.
                MsgFlags::MSG_ERRQUEUE,
            ) {
                Ok(recv) => break recv,

                // poor-mans async IO :'(
                Err(Errno::EWOULDBLOCK) => {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }

                Err(e) => bail!("Faileed to read from socket {e}"),
            };
        };

        // NOTE: This should be the IP destination of our ping packets. That does NOT mean the
        // packets reached the destination. Instead, if we see an EHOSTUNREACH control message,
        // it means the packets was instead dropped along the way. Seeing this address helps us
        // identify that this is a response to the ping we sent.
        // // FIXME: sockaddr_in only works for ipv4
        let source: SockaddrIn = recv.address.unwrap();
        let source = source.ip();
        debug_assert_eq!(source, destination);

        let mut control_messages = recv
            .cmsgs()
            .context("Failed to decode cmsgs from recvmsg")?;

        let error_source = match control_messages.next() {
            Some(ControlMessageOwned::Ipv6RecvErr(_socket_error, _source_addr)) => {
                bail!("IPv6 not implemented");
            }
            Some(ControlMessageOwned::Ipv4RecvErr(socket_error, source_addr)) => {
                let libc::sock_extended_err {
                    ee_errno,   // Error Number: Should be EHOSTUNREACH
                    ee_origin,  // Error Origin: 2 = Icmp, 3 = Icmp6.
                    ee_type,    // ICMP Type: 11 = ICMP/TimeExceeded.
                    ee_code,    // ICMP Code. 0 = TTL exceeded in transit.
                    ee_pad: _,  // padding
                    ee_info: _, // N/A
                    ee_data: _, // N/A
                } = socket_error;

                let errno = Errno::from_raw(ee_errno as i32);
                debug_assert_eq!(errno, Errno::EHOSTUNREACH);
                debug_assert_eq!(ee_origin, nix::libc::SO_EE_ORIGIN_ICMP); // TODO: or SO_EE_ORIGIN_ICMP6

                // TODO: Icmp6Types
                let icmp_type = IcmpType::new(ee_type);
                debug_assert_eq!(icmp_type, IcmpTypes::TimeExceeded);

                let icmp_code = IcmpCode::new(ee_code);
                debug_assert_eq!(icmp_code, IcmpCodes::TimeToLiveExceededInTransit);

                // NOTE: This is the IP of the node that dropped the packet due to TTL exceeded.
                let error_source = SockaddrIn::from(source_addr.unwrap());
                log::debug!("addr: {error_source}");

                error_source
            }
            Some(other_message) => {
                // TODO: We might want to not error in this case, and just ignore the cmsg.
                // If so, we should loop over the iterator instead of taking the first elem.
                bail!("Unhandled control message: {other_message:?}");
            }
            None => {
                // We're looking for EHOSTUNREACH errors. No errors means skip.
                log::debug!("Skipping recvmsg that produced no control messages.");
                continue;
            }
        };

        let packet = recv.iovs().next().unwrap();

        // Ensure that this is the original Echo packet that we sent.
        // TODO: skip on error
        parse_icmp_echo_raw(packet).context("")?;

        log::debug!("Got a probe response, we are leaking!");
        timeout_at.get_or_insert_with(|| Instant::now() + RECV_GRACE_TIME);
        reachable_nodes.push(IpAddr::from(error_source.ip()));
    }

    debug_assert!(!reachable_nodes.is_empty());

    Ok(LeakStatus::LeakDetected(
        LeakInfo::NodeReachableOnInterface {
            reachable_nodes,
            interface: interface.clone(),
        },
    ))
}
