use std::{
    ffi::{c_char, c_int, c_void},
    mem::{self, size_of},
    net::IpAddr,
    os::fd::AsRawFd,
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use futures::{select, FutureExt};
use libc::{ETH_ALEN, ETH_P_IP};
use mullvad_management_interface::MullvadProxyClient;
use nix::{
    errno::Errno,
    ioctl_readwrite_bad,
    sys::socket::{MsgFlags, SockProtocol},
};
use pnet_packet::{
    ethernet::{EtherType, EthernetPacket, MutableEthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    tcp::{MutableTcpPacket, TcpFlags, TcpPacket},
    Packet,
};
use socket2::Socket;
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{pin, task::yield_now, time::sleep};

use crate::{
    tests::helpers,
    vm::network::{linux::TAP_NAME, NON_TUN_GATEWAY},
};

use super::TestContext;

/// The port number we set in the malicious packet.
const MALICIOUS_PACKET_PORT: u16 = 12345;

const TCP_LEN: usize = 20;
const IP4_LEN: usize = 20 + TCP_LEN;
const ETH_LEN: usize = 14 + IP4_LEN;

/// Test mitigation for cve-2019-14899.
///
/// The vulnerability allowed a malicious router learn the victims private mullvad tunnel IP.
/// It is performed by sending a TCP packet to the victim with SYN and ACK flags set.
///
/// If the destination_addr of the packet was the same as the private IP, the victims computer
/// would respond to the packet with the RST flag set.
///
/// This test simply gets the private tunnel IP from the test runner and sends the SYN/ACK packet
/// targeted to that address. If the guest does not respond, the test passes.
#[test_function(target_os = "linux")]
pub async fn test_cve_2019_14899_mitigation(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    helpers::connect_and_wait(&mut mullvad_client).await?;

    // The vulnerability required local network sharing to be enabled.
    mullvad_client
        .set_allow_lan(true)
        .await
        .context("Failed to allow local network sharing")?;

    let host_interface = TAP_NAME;
    let victim_tunnel_if = "wg0-mullvad";
    let gateway_ip = NON_TUN_GATEWAY;

    // Create a raw socket which let's us send custom ethernet packets
    log::info!("Creating raw socket.");
    let socket = Socket::new(
        socket2::Domain::PACKET,
        socket2::Type::RAW,
        Some((SockProtocol::EthAll as c_int).into()),
    )
    .with_context(|| "Failed to create raw socket")?;

    log::info!("Binding raw socket to tap interface.");
    socket
        .bind_device(Some(host_interface.as_bytes()))
        .with_context(|| anyhow!("Failed to bind the socket to {host_interface:?}"))?;

    // Get the private IP address of the victims VPN tunnel
    let victim_tunnel_ip = rpc
        .get_interface_ip(victim_tunnel_if.to_string())
        .await
        .with_context(|| {
            anyhow!("Failed to get ip of guest tunnel interface {victim_tunnel_if:?}")
        })?;

    let IpAddr::V4(victim_tunnel_ip) = victim_tunnel_ip else {
        bail!("I didn't ask for IPv6!");
    };

    let victim_default_if = rpc
        .get_default_interface()
        .await
        .context("failed to get guest default interface")?;

    let victim_default_if_mac = rpc
        .get_interface_mac(victim_default_if.clone())
        .await
        .with_context(|| {
            anyhow!("Failed to get ip of guest default interface {victim_default_if:?}")
        })?
        .ok_or(anyhow!(
            "No mac address for guest default interface {victim_default_if:?}"
        ))?;

    // Get the MAC address and "index" of the tap interface.
    let mut host_interface_mac = [0u8; 6];
    let host_interface_index: c_int;
    {
        // Set up ioctl request structs.
        let mut if_mac_request: libc::ifreq = unsafe { mem::zeroed() };
        let host_interface: &[c_char] = u8_slice_to_c_char(host_interface.as_bytes());
        if_mac_request.ifr_name[..host_interface.len()].copy_from_slice(host_interface);

        let mut if_index_request: libc::ifreq = if_mac_request;

        // call the netdev ioctl that gets the interface index
        ioctl_readwrite_bad!(get_interface_index, libc::SIOCGIFINDEX, libc::ifreq);
        unsafe { get_interface_index(socket.as_raw_fd(), &mut if_index_request) }
            .context("Failed to get index of host interface")?;
        host_interface_index = unsafe { if_index_request.ifr_ifru.ifru_ifindex };

        // call the netdev ioctl that gets the interface mac address
        ioctl_readwrite_bad!(get_interface_mac, libc::SIOCGIFHWADDR, libc::ifreq);
        unsafe { get_interface_mac(socket.as_raw_fd(), &mut if_mac_request) }
            .context("Failed to get MAC address of host interface")?;

        host_interface_mac.copy_from_slice(c_char_slice_to_u8(unsafe {
            &if_mac_request.ifr_ifru.ifru_hwaddr.sa_data[..6]
        }));
    }

    // craft a malicious packet.
    let mut tcp_packet =
        MutableTcpPacket::owned(vec![0u8; TCP_LEN]).expect("TCP_LEN bytes is enough");
    tcp_packet.set_source(MALICIOUS_PACKET_PORT);
    tcp_packet.set_destination(MALICIOUS_PACKET_PORT);
    tcp_packet.set_data_offset(5); // 5 is smallest possible value
    tcp_packet.set_window(0xff);
    tcp_packet.set_flags(TcpFlags::SYN | TcpFlags::ACK);

    let mut ip4_packet =
        MutableIpv4Packet::owned(vec![0u8; IP4_LEN]).expect("IP4_LEN bytes is enough");
    ip4_packet.set_version(4);
    ip4_packet.set_header_length(5);
    ip4_packet.set_total_length(IP4_LEN as u16);
    ip4_packet.set_identification(0x77);
    ip4_packet.set_ttl(0xff);
    ip4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ip4_packet.set_source(gateway_ip);
    ip4_packet.set_destination(victim_tunnel_ip);
    tcp_packet.set_checksum(pnet_packet::tcp::ipv4_checksum(
        &tcp_packet.to_immutable(),
        &ip4_packet.get_source(),
        &ip4_packet.get_destination(),
    ));
    ip4_packet.set_payload(tcp_packet.packet());
    ip4_packet.set_checksum(pnet_packet::ipv4::checksum(&ip4_packet.to_immutable()));

    let mut eth_packet =
        MutableEthernetPacket::owned(vec![0u8; ETH_LEN]).expect("ETH_LEN bytes is enough");
    eth_packet.set_destination(victim_default_if_mac.into());
    eth_packet.set_source(host_interface_mac.into());
    eth_packet.set_ethertype(EtherType::new(ETH_P_IP as u16));
    eth_packet.set_payload(ip4_packet.packet());

    let eth_packet = eth_packet.consume_to_immutable();

    let filter = |tcp: &TcpPacket<'_>| {
        let reset_flag_set = (tcp.get_flags() & TcpFlags::RST) != 0;
        let correct_source_port = tcp.get_source() == MALICIOUS_PACKET_PORT;
        let correct_destination_port = tcp.get_destination() == MALICIOUS_PACKET_PORT;

        reset_flag_set && correct_source_port && correct_destination_port
    };

    let saw_rst_packet = select! {
        result = spam_packet(&socket, host_interface_index, &eth_packet).fuse() => return result,
        result = listen_for_packet(&socket, filter, Duration::from_secs(6000)).fuse() => result?,
    };

    if saw_rst_packet {
        bail!("Managed to leak private tunnel IP.");
    }

    Ok(())
}

/// Read from the socket and return true if we see a packet that passes the filter.
/// Returns false if we don't see such a packet within the timeout.
async fn listen_for_packet(
    socket: &Socket,
    filter: impl Fn(&TcpPacket<'_>) -> bool,
    timeout: Duration,
) -> anyhow::Result<bool> {
    let mut buf = vec![0u8; 0xffff];

    let wait_for_packet = async {
        loop {
            // yield so we don't end up hogging the runtime while polling the socket
            yield_now().await;

            match poll_for_packet(socket, &mut buf)? {
                Some(eth_packet) => {
                    let Some(ip4_packet) = Ipv4Packet::new(eth_packet.payload()) else {
                        continue;
                    };

                    let Some(tcp_packet) = TcpPacket::new(ip4_packet.payload()) else {
                        continue;
                    };

                    if filter(&tcp_packet) {
                        break;
                    }
                }
                None => sleep(Duration::from_millis(10)).await,
            }
        }
        anyhow::Ok(())
    };
    pin!(wait_for_packet);

    select! {
        result = wait_for_packet.fuse() => result.map(|_| true),
        _ = sleep(timeout).fuse() => Ok(false),
    }
}

/// Send `packet` on the socket in a loop.
async fn spam_packet(
    socket: &Socket,
    interface_index: c_int,
    packet: &EthernetPacket<'_>,
) -> anyhow::Result<()> {
    loop {
        send_packet(socket, interface_index, packet)?;
        sleep(Duration::from_millis(50)).await;
    }
}

/// Send an ethernet packet on the socket.
fn send_packet(
    socket: &Socket,
    interface_index: c_int,
    packet: &EthernetPacket<'_>,
) -> anyhow::Result<()> {
    let result = {
        let mut destination = libc::sockaddr_ll {
            sll_family: 0,
            sll_protocol: 0,
            sll_ifindex: interface_index,
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: ETH_ALEN as u8,
            sll_addr: [0; 8],
        };
        destination.sll_addr[..6].copy_from_slice(&packet.get_destination().octets());

        unsafe {
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

fn poll_for_packet<'a>(
    socket: &Socket,
    buf: &'a mut [u8],
) -> anyhow::Result<Option<EthernetPacket<'a>>> {
    let n = match nix::sys::socket::recv(socket.as_raw_fd(), &mut buf[..], MsgFlags::MSG_DONTWAIT) {
        Ok(0) | Err(Errno::EWOULDBLOCK) => return Ok(None),
        Err(e) => return Err(e).context("failed to read from socket"),
        Ok(n) => n,
    };

    let packet = &buf[..n];
    if packet.len() >= ETH_LEN {
        let eth_packet = EthernetPacket::new(packet).expect("packet is big enough");
        let Some(ip4_packet) = Ipv4Packet::new(eth_packet.payload()) else {
            return Ok(None);
        };

        let valid_ip_version = ip4_packet.get_version() == 4;
        let protocol_is_tcp = ip4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp;

        if valid_ip_version && protocol_is_tcp {
            return Ok(Some(eth_packet));
        }
    }

    Ok(None)
}

fn u8_slice_to_c_char(slice: &[u8]) -> &[c_char] {
    unsafe { std::mem::transmute(slice) }
}

fn c_char_slice_to_u8(slice: &[c_char]) -> &[u8] {
    unsafe { std::mem::transmute(slice) }
}
