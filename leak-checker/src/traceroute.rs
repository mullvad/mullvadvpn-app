use std::{
    ascii::escape_default,
    io,
    net::{IpAddr, Ipv4Addr},
    ops::{Range, RangeFrom},
    time::Duration,
};

use eyre::{bail, ensure, eyre, OptionExt, WrapErr};
use futures::{future::pending, stream, StreamExt, TryFutureExt, TryStreamExt};
use match_cfg::match_cfg;
use pnet_packet::{
    icmp::{
        echo_request::EchoRequestPacket, time_exceeded::TimeExceededPacket, IcmpPacket, IcmpTypes,
    },
    ip::IpNextHeaderProtocols as IpProtocol,
    ipv4::Ipv4Packet,
    udp::UdpPacket,
    Packet,
};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    net::UdpSocket,
    select,
    time::{sleep, sleep_until, timeout, Instant},
};

use crate::{LeakInfo, LeakStatus};

#[derive(Clone, clap::Args)]
pub struct TracerouteOpt {
    /// Try to bind to a specific interface
    #[clap(short, long)]
    pub interface: String,

    /// Destination IP of the probe packets
    #[clap(short, long)]
    pub destination: IpAddr,

    /// Avoid sending probe packets to this port
    #[clap(long)]
    pub exclude_port: Option<u16>,

    /// Send probe packets only to this port, instead of the default ports.
    #[clap(long)]
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
const RECV_TIMEOUT: Duration = Duration::from_secs(1);

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
pub async fn try_run_leak_test(opt: &TracerouteOpt) -> eyre::Result<LeakStatus> {
    // create the socket used for receiving the ICMP/TimeExceeded responses
    let icmp_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))
        .wrap_err("Failed to open ICMP socket")?;

    icmp_socket
        .set_nonblocking(true)
        .wrap_err("Failed to set icmp_socket to nonblocking")?;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use std::ffi::c_void;
        use std::os::fd::{AsFd, AsRawFd};

        let n = 1;
        unsafe {
            nix::libc::setsockopt(
                icmp_socket.as_fd().as_raw_fd(),
                nix::libc::SOL_IP,
                nix::libc::IP_RECVERR,
                &n as *const _ as *const c_void,
                size_of_val(&n) as u32,
            )
        };
    }

    bind_socket_to_interface(&icmp_socket, &opt.interface)?;

    // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
    // SAFETY: `into_raw_fd()` consumes the socket and returns an owned & open file descriptor.
    let icmp_socket = unsafe { std::net::UdpSocket::from_raw_fd(icmp_socket.into_raw_fd()) };
    let mut icmp_socket = UdpSocket::from_std(icmp_socket)?;

    // on Windows, we need to do some additional configuration of the raw socket
    #[cfg(target_os = "windows")]
    configure_listen_socket(&icmp_socket, interface)?;

    if opt.icmp {
        timeout(SEND_TIMEOUT, send_icmp_probes(&mut icmp_socket, opt))
            .map_err(|_timeout| eyre!("Timed out while trying to send probe packet"))
            .await??;
    } else {
        // create the socket used for sending the UDP probing packets
        let udp_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .wrap_err("Failed to open UDP socket")?;
        bind_socket_to_interface(&udp_socket, &opt.interface)
            .wrap_err("Failed to bind UDP socket to interface")?;
        udp_socket
            .set_nonblocking(true)
            .wrap_err("Failed to set udp_socket to nonblocking")?;

        // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
        // SAFETY: `into_raw_fd()` consumes the socket and returns an owned & open file descriptor.
        let udp_socket = unsafe { std::net::UdpSocket::from_raw_fd(udp_socket.into_raw_fd()) };
        let mut udp_socket = UdpSocket::from_std(udp_socket)?;

        timeout(SEND_TIMEOUT, send_udp_probes(&mut udp_socket, opt))
            .map_err(|_timeout| eyre!("Timed out while trying to send probe packet"))
            .await??;
    }

    let recv_task = read_probe_responses_no_root(opt.destination, &opt.interface, icmp_socket);
    //let recv_task = read_probe_responses(&opt.interface, icmp_socket);

    // wait until either task exits, or the timeout is reached
    let leak_status = select! {
        _ = sleep(LEAK_TIMEOUT) => LeakStatus::NoLeak,
        result = recv_task => result?,
    };

    // let send_task = timeout(SEND_TIMEOUT, send_icmp_probes(&mut udp_socket, opt))
    //     .map_err(|_timeout| eyre!("Timed out while trying to send probe packet"))
    //     // never return on success
    //     .and_then(|_| pending());
    //
    // let recv_task = read_probe_responses(&opt.interface, icmp_socket);
    //
    // wait until either thread exits, or the timeout is reached
    // let leak_status = select! {
    //     _ = sleep(LEAK_TIMEOUT) => LeakStatus::NoLeak,
    //     result = recv_task => result?,
    //     result = send_task => result?,
    // };

    Ok(leak_status)
}

async fn send_icmp_probes(socket: &mut UdpSocket, opt: &TracerouteOpt) -> eyre::Result<()> {
    use pnet_packet::icmp::{echo_request::*, *};

    let ports = DEFAULT_PORT_RANGE
        // ensure we don't send anything to `opt.exclude_port`
        .filter(|&p| Some(p) != opt.exclude_port)
        // `opt.port` overrides the default port range
        .map(|port| opt.port.unwrap_or(port));

    for (port, ttl) in ports.zip(DEFAULT_TTL_RANGE) {
        log::debug!("sending probe packet (ttl={ttl})");

        socket
            .set_ttl(ttl.into())
            .wrap_err("Failed to set TTL on socket")?;

        // the first packet will sometimes get dropped on MacOS, thus we send two packets
        let number_of_sends = if cfg!(target_os = "macos") { 2 } else { 1 };

        let echo = EchoRequest {
            icmp_type: IcmpTypes::EchoRequest,
            icmp_code: IcmpCode(0),
            checksum: 0,
            identifier: 1,
            sequence_number: 1,
            payload: PROBE_PAYLOAD.to_vec(),
        };
        let mut packet =
            MutableEchoRequestPacket::owned(vec![0u8; 8 + PROBE_PAYLOAD.len()]).unwrap();
        packet.populate(&echo);
        packet.set_checksum(checksum(&IcmpPacket::new(packet.packet()).unwrap()));

        let result: io::Result<()> = stream::iter(0..number_of_sends)
            // call `send_to` `number_of_sends` times
            .then(|_| socket.send_to(&packet.packet(), (opt.destination, port)))
            .map_ok(drop)
            .try_collect() // abort on the first error
            .await;

        let Err(e) = result else { continue };
        match e.kind() {
            io::ErrorKind::PermissionDenied => {
                // Linux returns this error if our packet was rejected by nftables.
                log::debug!("send_to failed with 'permission denied'");
            }
            _ => return Err(e).wrap_err("Failed to send packet")?,
        }
    }

    Ok(())
}

async fn send_udp_probes(socket: &mut UdpSocket, opt: &TracerouteOpt) -> eyre::Result<()> {
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
            .wrap_err("Failed to set TTL on socket")?;

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
            _ => return Err(e).wrap_err("Failed to send packet")?,
        }
    }

    Ok(())
}

/// Try to read ICMP/TimeExceeded error packets from an ICMP socket.
///
/// This method does not require root, but only works on Linux (including Android).
// TODO: double check if this works on MacOS
#[cfg(any(target_os = "linux", target_os = "android"))]
async fn read_probe_responses_no_root(
    destination: IpAddr,
    interface: &str,
    socket: UdpSocket,
) -> eyre::Result<LeakStatus> {
    use nix::errno::Errno;
    use nix::sys::socket::{ControlMessageOwned, MsgFlags, SockaddrIn};
    use nix::{cmsg_space, libc};
    use pnet_packet::icmp::time_exceeded::IcmpCodes;
    use pnet_packet::icmp::{IcmpCode, IcmpType};
    use std::io::IoSliceMut;
    use std::net::IpAddr;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd},

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
        // packets reached the destination, if we see an EHOSTUNREACH control message, it means the
        // packets was instead dropped along the way. Seeing this addres helps us identify that
        // this is a response to the ping we sent.
        // // FIXME: sockaddr_in only works for ipv4
        let source: SockaddrIn = recv.address.unwrap();
        let source = source.ip();
        debug_assert_eq!(source, destination);

        let mut control_messages = recv
            .cmsgs()
            .wrap_err("Failed to decode cmsgs from recvmsg")?;

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

        // NOTE: This is the original Echo packet that we sent.
        // TODO: make sure.
        let packet = recv.iovs().next().unwrap();
        let packet = Ipv4Packet::new(packet).ok_or_else(too_small)?;
        let _original_icmp_echo = parse_icmp_echo(&packet);

        log::debug!("Got a probe response, we are leaking!");
        timeout_at.get_or_insert_with(|| Instant::now() + RECV_TIMEOUT);
        reachable_nodes.push(IpAddr::from(error_source.ip()));
    }

    debug_assert!(!reachable_nodes.is_empty());

    Ok(LeakStatus::LeakDetected(
        LeakInfo::NodeReachableOnInterface {
            reachable_nodes,
            interface: interface.to_string(),
        },
    ))
}

async fn read_probe_responses(interface: &str, socket: UdpSocket) -> eyre::Result<LeakStatus> {
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

        log::debug!("Reading from ICMP socket");

        // let n = socket
        //    .recv(unsafe { &mut *(&mut read_buf[..] as *mut [u8] as *mut [MaybeUninit<u8>]) })
        //    .wrap_err("Failed to read from raw socket")?;

        let (n, source) = select! {
            result = socket.recv_from(&mut read_buf[..]) => result
                .wrap_err("Failed to read from raw socket")?,

            _timeout = timer => {
                return Ok(LeakStatus::LeakDetected(LeakInfo::NodeReachableOnInterface {
                    reachable_nodes,
                    interface: interface.to_string(),
                }));
            }
        };

        let source = source.ip();
        let packet = &read_buf[..n];
        let result = parse_ipv4(packet)
            .map_err(|e| eyre!("Ignoring packet: (len={n}, ip.src={source}) {e} ({packet:02x?})"))
            .and_then(|ip_packet| {
                parse_icmp_time_exceeded(&ip_packet).map_err(|e| {
                    eyre!(
                        "Ignoring packet (len={n}, ip.src={source}, ip.dest={}): {e}",
                        ip_packet.get_destination(),
                    )
                })
            });

        match result {
            Ok(ip) => {
                log::debug!("Got a probe response, we are leaking!");
                timeout_at.get_or_insert_with(|| Instant::now() + RECV_TIMEOUT);
                let ip = IpAddr::from(ip);
                if !reachable_nodes.contains(&ip) {
                    reachable_nodes.push(ip);
                }
            }

            // an error means the packet wasn't the ICMP/TimeExceeded we're listening for.
            Err(e) => log::debug!("{e}"),
        }
    }
}

/// Configure the raw socket we use for listening to ICMP responses.
///
/// This will bind the socket to an interface, and set the `SIO_RCVALL`-option.
#[cfg(target_os = "windows")]
fn configure_listen_socket(socket: &Socket, interface: &str) -> eyre::Result<()> {
    use std::{ffi::c_void, os::windows::io::AsRawSocket, ptr::null_mut};
    use windows_sys::Win32::Networking::WinSock::{
        WSAGetLastError, WSAIoctl, SIO_RCVALL, SOCKET, SOCKET_ERROR,
    };

    bind_socket_to_interface(&socket, interface)
        .wrap_err("Failed to bind listen socket to interface")?;

    let j = 1;
    let mut _in: u32 = 0;
    let result = unsafe {
        WSAIoctl(
            socket.as_raw_socket() as SOCKET,
            SIO_RCVALL,
            &j as *const _ as *const c_void,
            size_of_val(&j) as u32,
            null_mut(),
            0,
            &mut _in as *mut u32,
            null_mut(),
            None,
        )
    };

    if result == SOCKET_ERROR {
        let code = unsafe { WSAGetLastError() };
        bail!("Failed to call WSAIoctl(listen_socket, SIO_RCVALL, ...), code = {code}");
    }

    Ok(())
}

/// Try to parse the bytes as an IPv4 packet.
///
/// This only valdiates the IPv4 header, not the payload.
fn parse_ipv4(packet: &[u8]) -> eyre::Result<Ipv4Packet<'_>> {
    let ip_packet = Ipv4Packet::new(packet).ok_or_else(too_small)?;
    ensure!(ip_packet.get_version() == 4, "Not IPv4");
    eyre::Ok(ip_packet)
}

/// Try to parse an [Ipv4Packet] as an ICMP/TimeExceeded response to a packet sent by
/// [send_probes]. If successful, returns the [Ipv4Addr] of the packet source.
///
/// If the packet fails to parse, or is not a reply to a packet sent by [send_probes], this
/// function returns an error.
fn parse_icmp_time_exceeded(ip_packet: &Ipv4Packet<'_>) -> eyre::Result<Ipv4Addr> {
    let ip_protocol = ip_packet.get_next_level_protocol();
    ensure!(ip_protocol == IpProtocol::Icmp, "Not ICMP");
    parse_icmp_time_exceeded_raw(ip_packet.payload())?;
    Ok(ip_packet.get_source())
}

fn parse_icmp_time_exceeded_raw(bytes: &[u8]) -> eyre::Result<()> {
    let icmp_packet = IcmpPacket::new(bytes).ok_or(eyre!("Too small"))?;

    let correct_type = icmp_packet.get_icmp_type() == IcmpTypes::TimeExceeded;
    ensure!(correct_type, "Not ICMP/TimeExceeded");

    let time_exceeeded = TimeExceededPacket::new(icmp_packet.packet()).ok_or_else(too_small)?;

    let original_ip_packet = Ipv4Packet::new(time_exceeeded.payload()).ok_or_else(too_small)?;
    let original_ip_protocol = original_ip_packet.get_next_level_protocol();
    ensure!(original_ip_packet.get_version() == 4, "Not IPv4");

    match original_ip_protocol {
        IpProtocol::Udp => {
            let original_udp_packet =
                UdpPacket::new(original_ip_packet.payload()).ok_or_else(too_small)?;

            // check if payload looks right
            // some network nodes will strip the payload, that's fine.
            if !original_udp_packet.payload().is_empty() {
                let udp_len = usize::from(original_udp_packet.get_length());
                let udp_payload = udp_len
                    .checked_sub(UdpPacket::minimum_packet_size())
                    .and_then(|len| original_udp_packet.payload().get(..len))
                    .ok_or_eyre("Invalid UDP length")?;
                if udp_payload != &PROBE_PAYLOAD {
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

        IpProtocol::Icmp => {
            let original_icmp_packet =
                EchoRequestPacket::new(original_ip_packet.payload()).ok_or_else(too_small)?;

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

fn parse_icmp_echo(ip_packet: &Ipv4Packet<'_>) -> eyre::Result<()> {
    let ip_protocol = ip_packet.get_next_level_protocol();

    match ip_protocol {
        IpProtocol::Icmp => {
            let echo_packet = EchoRequestPacket::new(ip_packet.payload()).ok_or_else(too_small)?;

            ensure!(
                echo_packet.get_icmp_type() == IcmpTypes::EchoRequest,
                "Not ICMP/EchoRequest"
            );

            // check if payload looks right
            // some network nodes will strip the payload.
            // some network nodes will add a bunch of zeros at the end.
            let echo_payload = echo_packet.payload();
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

match_cfg! {
    #[cfg(any(target_os = "windows", target_os = "android"))] => {
        fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
            use crate::util::get_interface_ip;
            use std::net::SocketAddr;

            let interface_ip = get_interface_ip(interface)?;

            log::info!("Binding socket to {interface_ip} ({interface:?})");

            socket.bind(&SocketAddr::new(interface_ip, 0).into())
                .wrap_err("Failed to bind socket to interface address")?;

            return Ok(());
        }
    }
    #[cfg(target_os = "linux")] => {
        fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
            log::info!("Binding socket to {interface:?}");

            socket
                .bind_device(Some(interface.as_bytes()))
                .wrap_err("Failed to bind socket to interface")?;

            Ok(())
        }
    }
    #[cfg(target_os = "macos")] => {
        fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
            use nix::net::if_::if_nametoindex;
            use std::num::NonZero;

            log::info!("Binding socket to {interface:?}");

            let interface_index = if_nametoindex(interface)
                .map_err(eyre::Report::from)
                .and_then(|code| NonZero::new(code).ok_or_eyre("Non-zero error code"))
                .wrap_err("Failed to get interface index")?;

            socket.bind_device_by_index_v4(Some(interface_index))?;
            Ok(())
        }
    }
}

// OLD ICMP SEND CODE
//
// use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};
// let interface_luid = luid_from_alias(INTERFACE)?;
// let IpAddr::V4(interface_ip) =
// get_ip_address_for_interface(AddressFamily::Ipv4, interface_luid)?
// .ok_or(eyre!("No IP for interface {INTERFACE:?}"))?
// else {
// panic!()
// };
//
// for ttl in 1..=5 {
// let mut packet = Packet {
//    ip: Ipv4Header {
//        version_and_ihl: 0x45,
//        dscp_and_ecn: 0, // should be fine
//        total_length: (size_of::<Packet>() as u16).to_be_bytes(),
//        _stuff: Default::default(), // should be fine
//        ttl,
//        protocol: 1, // icmp
//        header_checksum: Default::default(),
//        source_address: interface_ip.octets(),
//        destination_address: destination.octets(),
//    },
//    icmp: Icmpv4Header {
//        icmp_type: 8, // echo
//        code: 0,
//        checksum: Default::default(),
//    },
// };
// let icmp = Icmpv4Header {
// icmp_type: 8, // echo
// code: 0,
// checksum: Default::default(),
// };
//
// packet.ip.header_checksum = checksum(packet.ip.as_bytes());
// let mut packet = Icmpv4Packet {
// header: icmp,
// payload: Icmpv4EchoPayload {
// identifier: 0u16.to_be_bytes(),
// sequence_number: (ttl as u16).to_be_bytes(),
// data: [0x77; 32],
// },
// };
//
// packet.header.checksum = checksum(packet.as_bytes());
//
// let packet = packet;
//
// listen_socket.set_ttl(ttl).wrap_err("Failed to set TTL")?;
// listen_socket
// .send_to(
// packet.as_bytes(),
// &SocketAddrV4::new(destination, 0u16).into(),
// )
// .wrap_err("Failed to send on raw socket")?;
// }

// use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};
// let interface_luid = luid_from_alias(INTERFACE)?;
// let IpAddr::V4(interface_ip) =
// get_ip_address_for_interface(AddressFamily::Ipv4, interface_luid)?
// .ok_or(eyre!("No IP for interface {INTERFACE:?}"))?
// else {
// panic!()
// };
//
// for ttl in 1..=5 {
// let mut packet = Packet {
//    ip: Ipv4Header {
//        version_and_ihl: 0x45,
//        dscp_and_ecn: 0, // should be fine
//        total_length: (size_of::<Packet>() as u16).to_be_bytes(),
//        _stuff: Default::default(), // should be fine
//        ttl,
//        protocol: 1, // icmp
//        header_checksum: Default::default(),
//        source_address: interface_ip.octets(),
//        destination_address: destination.octets(),
//    },
//    icmp: Icmpv4Header {
//        icmp_type: 8, // echo
//        code: 0,
//        checksum: Default::default(),
//    },
// };
// let icmp = Icmpv4Header {
// icmp_type: 8, // echo
// code: 0,
// checksum: Default::default(),
// };
//
// packet.ip.header_checksum = checksum(packet.ip.as_bytes());
// let mut packet = Icmpv4Packet {
// header: icmp,
// payload: Icmpv4EchoPayload {
// identifier: 0u16.to_be_bytes(),
// sequence_number: (ttl as u16).to_be_bytes(),
// data: [0x77; 32],
// },
// };
//
// packet.header.checksum = checksum(packet.as_bytes());
//
// let packet = packet;
//
// listen_socket.set_ttl(ttl).wrap_err("Failed to set TTL")?;
// listen_socket
// .send_to(
// packet.as_bytes(),
// &SocketAddrV4::new(destination, 0u16).into(),
// )
// .wrap_err("Failed to send on raw socket")?;
// }

fn too_small() -> eyre::Report {
    eyre!("Too small")
}
