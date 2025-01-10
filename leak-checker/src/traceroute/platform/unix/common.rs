#![allow(dead_code)] // some code here is not used on some targets.

use std::{
    future::pending,
    net::{IpAddr, SocketAddr},
};

use anyhow::{anyhow, Context};
use socket2::Socket;
use tokio::{
    select,
    time::{sleep_until, Instant},
};

use crate::{
    traceroute::{
        platform::unix::{
            parse_icmp4_time_exceeded, parse_icmp6_time_exceeded, parse_ipv4, parse_ipv6,
        },
        Ip, RECV_GRACE_TIME,
    },
    Interface, LeakInfo, LeakStatus,
};

use super::AsyncIcmpSocket;

pub fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
    let Interface::Name(interface) = interface;

    for interface_address in nix::ifaddrs::getifaddrs()? {
        if &interface_address.interface_name != interface {
            continue;
        };
        let Some(address) = interface_address.address else {
            continue;
        };

        match ip_version {
            Ip::V4(()) => {
                if let Some(address) = address.as_sockaddr_in() {
                    return Ok(IpAddr::V4(address.ip()));
                };
            }
            Ip::V6(()) => {
                if let Some(address) = address.as_sockaddr_in6() {
                    return Ok(IpAddr::V6(address.ip()));
                };
            }
        }
    }

    anyhow::bail!("Interface {interface:?} has no valid IP to bind to");
}

pub fn bind_socket_to_interface(
    socket: &Socket,
    interface: &Interface,
    ip_version: Ip,
) -> anyhow::Result<()> {
    let interface_ip = get_interface_ip(interface, ip_version)?;

    log::info!("Binding socket to {interface_ip} ({interface:?})");

    socket
        .bind(&SocketAddr::new(interface_ip, 0).into())
        .context("Failed to bind socket to interface address")?;

    Ok(())
}

pub(crate) async fn recv_ttl_responses(
    socket: &impl AsyncIcmpSocket,
    interface: &Interface,
) -> anyhow::Result<LeakStatus> {
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

        let parsed = match source {
            IpAddr::V4(..) => parse_ipv4(packet)
                .and_then(|ip_packet| parse_icmp4_time_exceeded(&ip_packet))
                .map(IpAddr::from),
            IpAddr::V6(..) => parse_ipv6(packet)
                .and_then(|ip_packet| parse_icmp6_time_exceeded(&ip_packet))
                .map(IpAddr::from),
        };

        let result = parsed.map_err(|e| {
            anyhow!("Ignoring packet: (len={n}, ip.src={source}) {e} ({packet:02x?})")
        });

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
