#![allow(dead_code)] // some code here is not used on some targets.

use std::{
    future::pending,
    net::{IpAddr, SocketAddr},
};

use eyre::{eyre, Context};
use socket2::Socket;
use tokio::{
    select,
    time::{sleep_until, Instant},
};

use crate::{
    traceroute::{parse_icmp_time_exceeded, parse_ipv4, RECV_TIMEOUT},
    LeakInfo, LeakStatus,
};

use super::{AsyncIcmpSocket, Impl, Traceroute};

pub fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
    let interface_ip = Impl::get_interface_ip(interface)?;

    log::info!("Binding socket to {interface_ip} ({interface:?})");

    socket
        .bind(&SocketAddr::new(interface_ip, 0).into())
        .wrap_err("Failed to bind socket to interface address")?;

    Ok(())
}

pub async fn recv_ttl_responses(
    socket: &impl AsyncIcmpSocket,
    interface: &str,
) -> eyre::Result<LeakStatus> {
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
