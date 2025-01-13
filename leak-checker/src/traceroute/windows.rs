use std::{net::IpAddr, str};

use anyhow::{anyhow, Context};
use futures::{select, stream::FuturesUnordered, FutureExt, StreamExt};

use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};
use tokio::time::sleep;

use crate::{
    traceroute::{
        Ip, TracerouteOpt, DEFAULT_TTL_RANGE, LEAK_TIMEOUT, PROBE_INTERVAL, SEND_TIMEOUT,
    },
    Interface, LeakInfo, LeakStatus,
};

/// Implementation of traceroute using `ping.exe`
///
/// This monstrosity exists because the Windows firewall is not helpful enough to allow us to
/// permit a process (the daemon) to receive ICMP TimeExceeded packets. We can get around this by
/// using `ping.exe`, which does work for some reason. My best guess is that it has special kernel
/// access to be able to do this.
pub async fn traceroute_using_ping(opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
    let ip_version = match opt.destination {
        IpAddr::V4(..) => Ip::V4(()),
        IpAddr::V6(..) => Ip::V6(()),
    };

    let interface_ip = get_interface_ip(&opt.interface, ip_version)?;

    let mut ping_tasks = FuturesUnordered::new();

    for (i, ttl) in DEFAULT_TTL_RANGE.enumerate() {
        // Don't send all pings at once, wait a bit in between
        // each one to avoid sending more than necessary
        let probe_delay = PROBE_INTERVAL * i as u32;

        ping_tasks.push(async move {
            sleep(probe_delay).await;

            log::debug!("sending probe packet (ttl={ttl})");

            // ping.exe will send ICMP Echo packets to the destination, and since it's running in
            // the kernel it will be able to receive TimeExceeded responses.
            let ping_path = r"C:\Windows\System32\ping.exe";
            let output = tokio::process::Command::new(ping_path)
                .args(["-i", &ttl.to_string()])
                .args(["-n", "1"]) // number of pings
                .args(["-w", &SEND_TIMEOUT.as_millis().to_string()])
                .args(["-S", &interface_ip.to_string()]) // bind to interface IP
                .arg(opt.destination.to_string())
                .kill_on_drop(true)
                .output()
                .await
                .context(anyhow!("Failed to execute {ping_path}"))?;

            let output_err = || anyhow!("Unexpected output from `ping.exe`");

            let stdout = str::from_utf8(&output.stdout).with_context(output_err)?;
            let _stderr = str::from_utf8(&output.stderr).with_context(output_err)?;

            log::trace!("ping stdout: {stdout}");
            log::trace!("ping stderr: {_stderr}");

            // Dumbly parse stdout for a line that looks like this:
            // "Reply from <ip>: TTL expired"

            if !stdout.contains("TTL expired") {
                // No "TTL expired" means we did not receive any TimeExceeded replies.
                return Ok(None);
            }
            let (ip, ..) = stdout
                .split_once("Reply from ")
                .and_then(|(.., s)| s.split_once(": TTL expired"))
                .with_context(output_err)?;

            let ip: IpAddr = ip.parse().unwrap();

            anyhow::Ok(Some(ip))
        });
    }

    let wait_for_first_leak = async move {
        while let Some(result) = ping_tasks.next().await {
            let Some(ip) = result? else { continue };

            return Ok(LeakStatus::LeakDetected(
                LeakInfo::NodeReachableOnInterface {
                    reachable_nodes: vec![ip],
                    interface: opt.interface.clone(),
                },
            ));
        }

        anyhow::Ok(LeakStatus::NoLeak)
    };

    select! {
        _ = sleep(LEAK_TIMEOUT).fuse() => Ok(LeakStatus::NoLeak),
        result = wait_for_first_leak.fuse() => result,
    }
}

pub fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
    let interface_luid = match interface {
        Interface::Name(name) => luid_from_alias(name)?,
        Interface::Luid(luid) => *luid,
    };

    let address_family = match ip_version {
        Ip::V4(..) => AddressFamily::Ipv4,
        Ip::V6(..) => AddressFamily::Ipv6,
    };

    get_ip_address_for_interface(address_family, interface_luid)
        .with_context(|| anyhow!("Failed to get IP for interface {interface:?}"))?
        .ok_or(anyhow!("No IP for interface {interface:?}"))
}

// TODO: remove this
/*
use std::{
    ffi::c_void,
    io, mem,
    net::{IpAddr, SocketAddr},
    os::windows::io::{AsRawSocket, AsSocket, FromRawSocket, IntoRawSocket},
    ptr::null_mut,
    str,
};

use anyhow::{anyhow, bail, Context};
use futures::{select, stream::FuturesUnordered, FutureExt, StreamExt};
use socket2::Socket;
use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};

use tokio::time::sleep;
use windows_sys::Win32::Networking::WinSock::{
    WSAGetLastError, WSAIoctl, SIO_RCVALL, SOCKET, SOCKET_ERROR,
};

use crate::{
    traceroute::{
        Ip, TracerouteOpt, DEFAULT_TTL_RANGE, LEAK_TIMEOUT, PROBE_INTERVAL, SEND_TIMEOUT,
    },
    Interface, LeakInfo, LeakStatus,
};
use super::{common, AsyncIcmpSocket, AsyncUdpSocket, Traceroute};

pub struct TracerouteWindows;

pub struct AsyncIcmpSocketImpl(tokio::net::UdpSocket);

pub struct AsyncUdpSocketWindows(tokio::net::UdpSocket);

impl Traceroute for TracerouteWindows {
    type AsyncIcmpSocket = AsyncIcmpSocketImpl;
    type AsyncUdpSocket = AsyncUdpSocketWindows;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()> {
        common::bind_socket_to_interface::<Self>(socket, interface, ip_version)
    }

    fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
        get_interface_ip(interface, ip_version)
    }

    fn configure_icmp_socket(socket: &socket2::Socket, _opt: &TracerouteOpt) -> anyhow::Result<()> {
        configure_icmp_socket(socket)
    }
}

impl AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket) -> Self {
        let raw_socket = socket.as_socket().as_raw_socket();
        mem::forget(socket);
        let std_socket = unsafe { std::net::UdpSocket::from_raw_socket(raw_socket) };
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        AsyncIcmpSocketImpl(tokio_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for ICMP socket")
    }

    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize> {
        self.0.send_to(packet, (destination.into(), 0)).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr)> {
        let (n, source) = self.0.recv_from(buf).await?;
        Ok((n, source.ip()))
    }

    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
        common::recv_ttl_responses(self, &opt.interface).await
    }
}

impl AsyncUdpSocket for AsyncUdpSocketWindows {
    fn from_socket2(socket: socket2::Socket) -> Self {
        // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
        let udp_socket = unsafe { std::net::UdpSocket::from_raw_socket(socket.into_raw_socket()) };
        let udp_socket = tokio::net::UdpSocket::from_std(udp_socket).unwrap();
        AsyncUdpSocketWindows(udp_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for UDP socket")
    }

    async fn send_to(
        &self,
        packet: &[u8],
        destination: impl Into<SocketAddr>,
    ) -> std::io::Result<usize> {
        self.0.send_to(packet, destination.into()).await
    }
}

/// Configure the raw socket we use for listening to ICMP responses.
///
/// This will set the `SIO_RCVALL`-option.
pub fn configure_icmp_socket(socket: &Socket) -> anyhow::Result<()> {
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
*/
