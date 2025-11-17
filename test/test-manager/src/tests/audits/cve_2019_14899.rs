#![cfg(target_os = "linux")]
//! Test mitigation for cve-2019-14899
//!
//! The vulnerability allowed a malicious router to learn the victims private mullvad tunnel IP.
//! It is performed by sending a TCP packet to the victim with SYN and ACK flags set.
//!
//! If the destination_addr of the packet was the same as the private IP, the victims computer
//! would respond to the packet with the RST flag set.
//!
//! This test simply gets the private tunnel IP from the test runner and sends the SYN/ACK packet
//! targeted to that address. If the guest does not respond, the test passes.
//!
//! Note that only linux was susceptible to this vulnerability.

use std::{
    convert::Infallible,
    ffi::{c_int, c_uint, c_void},
    mem::size_of,
    net::{IpAddr, Ipv4Addr},
    os::fd::AsRawFd,
    time::Duration,
};

use anyhow::{Context, anyhow, bail};
use futures::{FutureExt, select};
use mullvad_management_interface::MullvadProxyClient;
use nix::{
    errno::Errno,
    sys::socket::{self, MsgFlags, SockProtocol},
};
use pnet_base::MacAddr;
use pnet_packet::{
    MutablePacket, Packet,
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    tcp::{MutableTcpPacket, TcpFlags, TcpPacket},
};
use socket2::Socket;
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{task::yield_now, time::sleep};

use crate::{
    tests::{TestContext, config::TEST_CONFIG, helpers},
    vm::network::linux::TAP_NAME,
};

/// The port number we set in the malicious packet.
const MALICIOUS_PACKET_PORT: u16 = 12345;

#[test_function(target_os = "linux")]
pub async fn test_cve_2019_14899_mitigation(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // The vulnerability required local network sharing to be enabled
    mullvad_client
        .set_allow_lan(true)
        .await
        .context("Failed to allow local network sharing")?;

    helpers::connect_and_wait(&mut mullvad_client).await?;

    let host_interface = TAP_NAME;
    let victim_tunnel_interface = helpers::get_tunnel_interface(&mut mullvad_client)
        .await
        .context("Failed to find tunnel interface")?;
    let victim_gateway_ip = TEST_CONFIG.host_bridge_ip;

    // Create a raw socket which let's us send custom ethernet packets
    log::info!("Creating raw socket");
    let socket = Socket::new(
        socket2::Domain::PACKET,
        socket2::Type::RAW,
        Some(socket2::Protocol::from(SockProtocol::EthAll as c_int)),
    )
    .with_context(|| "Failed to create raw socket")?;

    log::info!("Binding raw socket to tap interface");
    socket
        .bind_device(Some(host_interface.as_bytes()))
        .with_context(|| anyhow!("Failed to bind the socket to {host_interface:?}"))?;

    // Get the private IP address of the victims VPN tunnel
    let victim_tunnel_ip = rpc
        .get_interface_ip(victim_tunnel_interface.clone())
        .await
        .with_context(|| {
            anyhow!("Failed to get ip of guest tunnel interface {victim_tunnel_interface:?}")
        })?;

    let IpAddr::V4(victim_tunnel_ip) = victim_tunnel_ip else {
        bail!("I didn't ask for IPv6!");
    };

    let victim_default_interface = rpc
        .get_default_interface()
        .await
        .context("failed to get guest default interface")?;

    let victim_default_interface_mac = rpc
        .get_interface_mac(victim_default_interface.clone())
        .await
        .with_context(|| {
            anyhow!("Failed to get ip of guest default interface {victim_default_interface:?}")
        })?
        .ok_or(anyhow!(
            "No mac address for guest default interface {victim_default_interface:?}"
        ))?;

    // Get the MAC address and index of the tap interface
    let host_interface_index = helpers::get_interface_index(host_interface)?;
    let host_interface_mac = helpers::get_interface_mac(host_interface)?.ok_or(anyhow!(
        "No mac address for host interface {host_interface:?}"
    ))?;

    let malicious_packet = craft_malicious_packet(
        MacAddr::from(host_interface_mac),
        MacAddr::from(victim_default_interface_mac),
        victim_gateway_ip,
        victim_tunnel_ip,
    );

    let filter = |tcp: &TcpPacket<'_>| {
        let reset_flag_set = (tcp.get_flags() & TcpFlags::RST) != 0;
        let correct_source_port = tcp.get_source() == MALICIOUS_PACKET_PORT;
        let correct_destination_port = tcp.get_destination() == MALICIOUS_PACKET_PORT;

        reset_flag_set && correct_source_port && correct_destination_port
    };

    let rst_packet = select! {
        result = filter_for_packet(&socket, filter, Duration::from_secs(5)).fuse() => result?,

        result = spam_packet(&socket, host_interface_index, &malicious_packet).fuse() => match result {
            Err(e) => return Err(e),
            Ok(never) => match never {}, // I dream of ! being stabilized
        },
    };

    if let Some(rst_packet) = rst_packet {
        log::warn!("Victim responded with an RST packet: {rst_packet:?}");
        bail!("Managed to leak private tunnel IP");
    }

    Ok(())
}

/// Read from the socket and return the first packet that passes the filter.
/// Returns `None` if we don't see such a packet within the timeout.
async fn filter_for_packet(
    socket: &Socket,
    filter: impl Fn(&TcpPacket<'_>) -> bool,
    timeout: Duration,
) -> anyhow::Result<Option<TcpPacket<'static>>> {
    let mut buf = vec![0u8; usize::from(u16::MAX)];

    let result = tokio::time::timeout(timeout, async {
        loop {
            let packet = poll_for_packet(socket, &mut buf).await?;
            if filter(&packet) {
                return anyhow::Ok(packet);
            }
        }
    });

    match result.await {
        Ok(packet) => Ok(Some(packet?)),
        Err(_timed_out) => Ok(None),
    }
}

/// Repeatedly poll the raw socket until we receives an Ethernet/IPv4/TCP packet.
/// Drops any non-TCP packets.
///
/// # Returns
/// - `Err` if the `read` system call failed.
/// - A single TCP packet otherwise.
async fn poll_for_packet(socket: &Socket, buf: &mut [u8]) -> anyhow::Result<TcpPacket<'static>> {
    loop {
        // yield so we don't end up hogging the runtime while polling the socket
        yield_now().await;

        let result = socket::recv(socket.as_raw_fd(), &mut buf[..], MsgFlags::MSG_DONTWAIT);

        let n = match result {
            Ok(0) | Err(Errno::EWOULDBLOCK) => {
                sleep(Duration::from_millis(10)).await;
                continue;
            }
            Err(e) => return Err(e).context("Failed to read from socket"),
            Ok(n) => n,
        };

        let packet = &buf[..n];

        let Some(eth_packet) = EthernetPacket::new(packet) else {
            continue;
        };

        if eth_packet.get_ethertype() != EtherTypes::Ipv4 {
            continue;
        }

        let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) else {
            continue;
        };

        let valid_ip_version = ipv4_packet.get_version() == 4;
        let protocol_is_tcp = ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp;

        if !valid_ip_version || !protocol_is_tcp {
            continue;
        }

        if let Some(tcp_packet) = TcpPacket::owned(ipv4_packet.payload().to_vec()) {
            return Ok(tcp_packet);
        };
    }
}

/// Send `packet` on the socket in a loop.
// NOTE: Replace return type with ! if/when stable.
async fn spam_packet(
    socket: &Socket,
    interface_index: c_uint,
    packet: &EthernetPacket<'_>,
) -> anyhow::Result<Infallible> {
    loop {
        send_packet(socket, interface_index, packet)?;
        sleep(Duration::from_millis(50)).await;
    }
}

/// Send an ethernet packet on the raw socket.
fn send_packet(
    socket: &Socket,
    interface_index: c_uint,
    packet: &EthernetPacket<'_>,
) -> anyhow::Result<()> {
    let result = {
        let mut destination = libc::sockaddr_ll {
            sll_family: 0,
            sll_protocol: 0,
            sll_ifindex: interface_index as c_int,
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: size_of::<MacAddr>() as u8,
            sll_addr: [0; 8],
        };
        destination.sll_addr[..6].copy_from_slice(&packet.get_destination().octets());
        unsafe {
            // NOTE: since you're reading this, consider using https://docs.rs/pnet_datalink
            // instead of whatever you're planning...
            libc::sendto(
                socket.as_raw_fd(),
                packet.packet().as_ptr() as *const c_void,
                packet.packet().len(),
                0,
                (&destination as *const libc::sockaddr_ll).cast(),
                size_of::<libc::sockaddr_ll>() as u32,
            )
        }
    };

    if result < 0 {
        let err = Errno::last();
        bail!("Failed to send ethernet packet: {err}");
    }

    Ok(())
}

fn craft_malicious_packet(
    source_mac: MacAddr,
    destination_mac: MacAddr,
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
) -> EthernetPacket<'static> {
    // length of the various parts of the malicious packet we'll be crafting.
    const TCP_LEN: usize = 20; // a TCP packet is 20 bytes
    const IPV4_LEN: usize = 20 + TCP_LEN; // an IPv4 packet is 20 bytes + payload
    const ETH_LEN: usize = 14 + IPV4_LEN; // an ethernet packet is 14 bytes + payload

    let mut eth_packet =
        MutableEthernetPacket::owned(vec![0u8; ETH_LEN]).expect("ETH_LEN bytes is enough");
    eth_packet.set_destination(destination_mac);
    eth_packet.set_source(source_mac);
    eth_packet.set_ethertype(EtherTypes::Ipv4);

    let mut ipv4_packet =
        MutableIpv4Packet::new(eth_packet.payload_mut()).expect("IPV4_LEN bytes is enough");
    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    ipv4_packet.set_total_length(IPV4_LEN as u16);
    ipv4_packet.set_identification(0x77);
    ipv4_packet.set_ttl(0xff);
    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ipv4_packet.set_source(source_ip);
    ipv4_packet.set_destination(destination_ip);
    ipv4_packet.set_checksum(pnet_packet::ipv4::checksum(&ipv4_packet.to_immutable()));

    let mut tcp_packet =
        MutableTcpPacket::new(ipv4_packet.payload_mut()).expect("TCP_LEN bytes is enough");
    tcp_packet.set_source(MALICIOUS_PACKET_PORT);
    tcp_packet.set_destination(MALICIOUS_PACKET_PORT);
    tcp_packet.set_data_offset(5); // 5 is smallest possible value
    tcp_packet.set_window(0xff);
    tcp_packet.set_flags(TcpFlags::SYN | TcpFlags::ACK);
    tcp_packet.set_checksum(pnet_packet::tcp::ipv4_checksum(
        &tcp_packet.to_immutable(),
        &source_ip,
        &destination_ip,
    ));

    eth_packet.consume_to_immutable()
}
