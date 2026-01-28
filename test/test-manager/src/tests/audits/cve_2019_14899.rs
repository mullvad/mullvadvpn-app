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
    net::{IpAddr, Ipv4Addr},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, anyhow, bail};
use futures::{FutureExt, select};
use mullvad_management_interface::MullvadProxyClient;
use pnet_base::MacAddr;
use pnet_datalink::{Channel, DataLinkReceiver, DataLinkSender, channel, linux::interfaces};
use pnet_packet::{
    MutablePacket, Packet,
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    tcp::{MutableTcpPacket, TcpFlags, TcpPacket},
};
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::time::sleep;

use crate::{
    tests::{TestContext, config::TEST_CONFIG, helpers},
    vm::network::linux::TAP_NAME,
};

/// The port number we set in the malicious packet.
const MALICIOUS_PACKET_PORT: u16 = 12345;

/// Timeout to use for finding the malicious packet.
const FILTER_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout to use for receiving a single packet from the link.
const RECV_TIMEOUT: Duration = Duration::from_secs(5);

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

    // Send malicious TCP packets to the supposed private IP while looking for a TCP Reset from the
    // victim.
    let io = IO::send_recv_on_interface(host_interface_index)?;
    let rst_packet = select! {
        result = filter_for_malicious_packet(io.recv, FILTER_TIMEOUT).fuse() => result?,
        Err(e) = spam_packet(io.send, &malicious_packet).fuse() => return Err(e),
    };

    if let Some(rst_packet) = rst_packet {
        log::warn!("Victim responded with an RST packet: {rst_packet:?}");
        bail!("Managed to leak private tunnel IP");
    }

    Ok(())
}

/// One specific instance of the return value of [`channel`].
struct IO {
    recv: Box<dyn DataLinkReceiver>,
    send: Box<dyn DataLinkSender>,
}

impl IO {
    /// Create channels for sending/receiving Ethernet frames on a given interface.
    fn send_recv_on_interface(interface: std::ffi::c_uint) -> anyhow::Result<IO> {
        let ifs = interfaces();
        let interface = ifs.iter().find(|i| i.index == interface).context(anyhow!(
            "Could not find network interface with index {interface}"
        ))?;
        let config = pnet_datalink::Config {
            // NOTE: We must set a timeout here, or `recv()` will never return
            // if there is nothing to receive, and the `spawn_blocking` thread will never stop.
            read_timeout: Some(RECV_TIMEOUT),
            ..Default::default()
        };
        let Channel::Ethernet(send, recv) = channel(interface, config).unwrap() else {
            unimplemented!("there are no other Channel variants yet")
        };
        Ok(IO { recv, send })
    }
}

/// Classify a given TCP packet, try to pinpoint if it is a reply to a packet created by
/// [`craft_malicious_packet`].
fn is_malicious_packet(tcp: &TcpPacket<'_>) -> bool {
    let reset_flag_set = (tcp.get_flags() & TcpFlags::RST) != 0;
    let correct_source_port = tcp.get_source() == MALICIOUS_PACKET_PORT;
    let correct_destination_port = tcp.get_destination() == MALICIOUS_PACKET_PORT;

    reset_flag_set && correct_source_port && correct_destination_port
}

/// Read from the link and return the first packet marked as malicious by [`is_malicious_packet`].
/// Returns `None` if we don't see such a packet within the timeout.
async fn filter_for_malicious_packet(
    mut recv: Box<dyn DataLinkReceiver>,
    timeout: Duration,
) -> anyhow::Result<Option<TcpPacket<'static>>> {
    let should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_thread = should_stop.clone();

    let mut thread = tokio::task::spawn_blocking(move || {
        loop {
            if should_stop_thread.load(Ordering::SeqCst) {
                bail!("Timed out waiting for malicious packet");
            }
            let packet = match recv.next() {
                Err(e) => return Err(e).context("Failed to read from data link"),
                Ok(p) => p,
            };
            log::trace!("Received Ethernet frame");
            let Some(packet) = ethernetframe_to_tcp(packet) else {
                continue;
            };
            log::trace!("Parsed Ethernet frame into TCP-packet!");
            if is_malicious_packet(&packet) {
                log::debug!("Identified TCP-packet as the malicious one!");
                return anyhow::Ok(packet);
            }
        }
    });

    match tokio::time::timeout(timeout, &mut thread).await {
        Ok(packet) => Ok(Some(packet??)),
        Err(_timed_out) => {
            should_stop.store(true, Ordering::SeqCst);
            // Avoid leaking thread
            let _ = thread.await;
            Ok(None)
        }
    }
}

/// Try to parse the bytes received on a [`channel`] data link.
///
/// # Returns
/// - `None` if the bytes are not a valid Ethernet/IPv4/TCP packet
/// - A single TCP packet otherwise.
fn ethernetframe_to_tcp(packet: &[u8]) -> Option<TcpPacket<'static>> {
    let eth_packet = EthernetPacket::new(packet)?;

    if eth_packet.get_ethertype() != EtherTypes::Ipv4 {
        return None;
    }

    let ipv4_packet = Ipv4Packet::new(eth_packet.payload())?;

    let valid_ip_version = ipv4_packet.get_version() == 4;
    let protocol_is_tcp = ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp;

    if !valid_ip_version || !protocol_is_tcp {
        return None;
    }

    TcpPacket::owned(ipv4_packet.payload().to_vec())
}

/// Send `packet` on the link in a loop.
// NOTE: Replace return type with ! if/when stable.
async fn spam_packet(
    mut send: Box<dyn DataLinkSender>,
    packet: &EthernetPacket<'_>,
) -> anyhow::Result<Infallible> {
    loop {
        send
        // destination is part of packet.
        .send_to(packet.packet(), None)
        .unwrap()
        .context("Failed to send ethernet packet")?;

        sleep(Duration::from_millis(50)).await;
    }
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
