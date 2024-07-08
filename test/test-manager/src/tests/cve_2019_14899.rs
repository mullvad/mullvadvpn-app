use std::{
    ffi::{c_char, c_int, c_void, OsString},
    mem::{self, size_of, size_of_val},
    net::IpAddr,
    os::fd::RawFd,
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use bytemuck::{bytes_of, cast_slice, from_bytes, Pod, Zeroable};
use futures::{select, FutureExt};
use libc::{ETH_ALEN, ETH_P_IP};
use mullvad_management_interface::MullvadProxyClient;
use nix::{
    errno::{errno, Errno},
    ioctl_readwrite_bad,
    sys::socket::{
        setsockopt,
        sockopt::{self},
        AddressFamily, MsgFlags, SockFlag, SockProtocol, SockType,
    },
};
use pnet_packet::{ip::IpNextHeaderProtocols, tcp::TcpFlags};
use rend::{u16_be, u32_be};
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{pin, task::yield_now, time::sleep};

use crate::{
    tests::helpers,
    vm::network::{linux::TAP_NAME, NON_TUN_GATEWAY},
};

use super::TestContext;

const IPV4_VERSION_IHL: u8 = 0x45;

/// The port number we set in the malicious packet.
const MALICIOUS_PACKET_PORT: u16 = 12345;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct EthernetPacket<T> {
    destination_mac: [u8; 6],
    source_mac: [u8; 6],
    ether_type: u16_be,
    data: T,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Ipv4Header {
    /// Must be set to [IPV4_VERSION_IHL].
    version_ihl: u8,
    dscp_ecn: u8,
    total_len: u16_be,
    identification: u16_be,
    flags_fragment_offset: u16_be,
    ttl: u8,
    protocol: u8,
    header_checksum: u16_be,
    source_addr: u32_be,
    destination_addr: u32_be,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Ipv4Packet<T> {
    header: Ipv4Header,
    data: T,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct TcpHeader {
    source_port: u16_be,
    destination_port: u16_be,
    seq_num: u32_be,
    ack_num: u32_be,
    data_offset: u8,
    tcp_flags: u8,
    window_size: u16_be,
    tcp_checksum: u16_be,
    urgent_pointer: u16_be,
}

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

    // Create a raw socket which let's us send custom ethernet packets
    log::info!("Creating raw socket.");
    let socket: RawFd = nix::sys::socket::socket(
        AddressFamily::Packet,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::EthAll,
    )
    .with_context(|| "Failed to create raw socket")?;

    let host_interface = TAP_NAME;
    let victim_tunnel_if = "wg0-mullvad";
    let gateway_ip = NON_TUN_GATEWAY;

    log::info!("Binding raw socket to tap interface.");
    setsockopt(
        socket,
        sockopt::BindToDevice,
        &OsString::from(host_interface),
    )
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
        let host_interface: &[c_char] = cast_slice(host_interface.as_bytes());
        if_mac_request.ifr_name[..host_interface.len()].copy_from_slice(host_interface);

        let mut if_index_request: libc::ifreq = if_mac_request;

        // call the netdev ioctl that gets the interface index
        ioctl_readwrite_bad!(get_interface_index, libc::SIOCGIFINDEX, libc::ifreq);
        unsafe { get_interface_index(socket, &mut if_index_request) }
            .context("Failed to get index of host interface")?;
        host_interface_index = unsafe { if_index_request.ifr_ifru.ifru_ifindex };

        // call the netdev ioctl that gets the interface mac address
        ioctl_readwrite_bad!(get_interface_mac, libc::SIOCGIFHWADDR, libc::ifreq);
        let result = unsafe { get_interface_mac(socket, &mut if_mac_request) }
            .context("Failed to get MAC address of host interface")?;
        log::warn!("SIOCGIFHWADDR result code: {result}");

        host_interface_mac.copy_from_slice(cast_slice(unsafe {
            &if_mac_request.ifr_ifru.ifru_hwaddr.sa_data[..6]
        }));
    }

    // craft a malicious packet.
    let mut ip_packet = Ipv4Packet {
        header: Ipv4Header {
            version_ihl: IPV4_VERSION_IHL,
            total_len: (size_of::<Ipv4Packet<TcpHeader>>() as u16).into(),
            identification: 0x77.into(),
            ttl: 0xff,
            protocol: IpNextHeaderProtocols::Tcp.0,
            source_addr: u32::from(gateway_ip).into(),

            // set the destination_addr to the victims private tunnel IP
            destination_addr: u32::from(victim_tunnel_ip).into(),

            ..Ipv4Header::zeroed()
        },
        data: TcpHeader {
            // We use the port value to help us identify the response packet.
            source_port: MALICIOUS_PACKET_PORT.into(),
            destination_port: MALICIOUS_PACKET_PORT.into(),

            data_offset: 0x50.into(), // 5
            window_size: 0xff.into(),

            // Important: Set the SYN and ACK flags
            tcp_flags: ((TcpFlags::SYN | TcpFlags::ACK) as u8).into(),

            ..TcpHeader::zeroed()
        },
    };
    ip_packet.calculate_checksum();

    let eth_packet = EthernetPacket {
        destination_mac: victim_default_if_mac,
        source_mac: host_interface_mac,
        ether_type: (ETH_P_IP as u16).into(),
        data: ip_packet,
    };

    let filter = |packet: &Ipv4Packet<TcpHeader>| {
        let source_port = packet.data.source_port;
        let destination_port = packet.data.destination_port;
        let reset_flag_set = (packet.data.tcp_flags & TcpFlags::RST as u8) != 0;

        reset_flag_set
            && source_port == MALICIOUS_PACKET_PORT
            && destination_port == MALICIOUS_PACKET_PORT
    };

    let saw_packet = select! {
        result = spam_packet(socket, host_interface_index, &eth_packet).fuse() => return result,
        result = listen_for_packet(socket, filter, Duration::from_secs(3)).fuse() => result?,
    };

    if saw_packet {
        bail!("Managed to leak private tunnel IP.");
    }

    Ok(())
}

/// Read from the socket and return true if we see a packet that passes the filter.
/// Returns false if we don't see such a packet within the timeout.
async fn listen_for_packet(
    socket: RawFd,
    filter: impl Fn(&Ipv4Packet<TcpHeader>) -> bool,
    timeout: Duration,
) -> anyhow::Result<bool> {
    let mut buf = vec![0u8; 0xffff];

    let listener = async {
        loop {
            // yield so we don't end up hogging the runtime polling the socket
            yield_now().await;

            match poll_for_packet(socket, &mut buf)? {
                Some(packet) if filter(&packet.data) => break,
                Some(_packet) => yield_now().await,
                None => sleep(Duration::from_millis(100)).await,
            }
        }
        anyhow::Ok(())
    };
    pin!(listener);

    select! {
        result = listener.fuse() => result.map(|_| true),
        _ = sleep(timeout).fuse() => Ok(false),
    }
}

/// Send `packet` on the socket in a loop.
async fn spam_packet(
    socket: RawFd,
    interface_index: c_int,
    packet: &EthernetPacket<Ipv4Packet<TcpHeader>>,
) -> anyhow::Result<()> {
    loop {
        send_packet(socket, interface_index, packet)?;
        sleep(Duration::from_millis(50)).await;
    }
}

/// Send an ethernet packet on the socket.
fn send_packet(
    socket: RawFd,
    interface_index: c_int,
    packet: &EthernetPacket<Ipv4Packet<TcpHeader>>,
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
        destination.sll_addr[..6].copy_from_slice(&packet.destination_mac);

        unsafe {
            libc::sendto(
                socket,
                bytes_of(packet).as_ptr() as *const c_void,
                size_of_val(packet),
                0,
                (&destination as *const libc::sockaddr_ll).cast(),
                size_of::<libc::sockaddr_ll>() as u32,
            )
        }
    };

    if result < 0 {
        let err = errno();
        bail!("Failed to send ethernet packet. code={err}");
    }

    Ok(())
}

fn poll_for_packet(
    socket: RawFd,
    buf: &mut [u8],
) -> anyhow::Result<Option<EthernetPacket<Ipv4Packet<TcpHeader>>>> {
    let n = match nix::sys::socket::recv(socket, &mut buf[..], MsgFlags::MSG_DONTWAIT) {
        Ok(0) | Err(Errno::EWOULDBLOCK) => return Ok(None),
        Err(e) => return Err(e).context("failed to read from socket"),
        Ok(n) => n,
    };

    let packet = &buf[..n];
    const LEN: usize = size_of::<EthernetPacket<Ipv4Packet<TcpHeader>>>();
    if packet.len() >= LEN {
        let packet: &[u8; LEN] = packet[..LEN].try_into().unwrap();
        let eth_packet: EthernetPacket<Ipv4Packet<TcpHeader>> = *from_bytes(packet);

        let valid_ip_version = eth_packet.data.header.version_ihl == IPV4_VERSION_IHL;
        let protocol_is_tcp = eth_packet.data.header.protocol == IpNextHeaderProtocols::Tcp.0;

        if valid_ip_version && protocol_is_tcp {
            return Ok(Some(eth_packet));
        }
    }

    Ok(None)
}

impl Ipv4Header {
    pub fn calculate_checksum(&mut self) {
        self.header_checksum = 0.into();
        self.header_checksum = checksum(bytes_of(self)).into();
    }
}

impl Ipv4Packet<TcpHeader> {
    pub fn calculate_checksum(&mut self) {
        // calculate TCP checksum by constructing the pseudo header that TCP expects
        // it's weird, I know...
        #[repr(C, packed)]
        #[derive(Clone, Copy, Debug, Pod, Zeroable)]
        struct PseudoHeader {
            source_addr: u32_be,
            destination_addr: u32_be,
            zeros: u8,
            protocol: u8,
            tcp_length: u16_be,
            tcp_header: TcpHeader,
        }

        let pseudo_header = PseudoHeader {
            source_addr: self.header.source_addr,
            destination_addr: self.header.destination_addr,
            zeros: 0,
            protocol: 6, // TCP
            tcp_length: (size_of::<TcpHeader>() as u16).into(),
            tcp_header: TcpHeader {
                tcp_checksum: 0.into(),
                ..self.data
            },
        };

        self.data.tcp_checksum = checksum(bytes_of(&pseudo_header)).into();

        self.header.calculate_checksum();
    }
}

/// Checksum algorithm used by TCP and IPv4
fn checksum(data: &[u8]) -> u16 {
    let mut sum: u64 = 0;

    // iterate over the data as big-endian u16s and sum them
    for short in data.chunks(2) {
        let short: &[u8; 2] = short.try_into().unwrap();
        let short = u16::from_be_bytes(*short);
        sum += u64::from(short);
    }

    // account for the last byte if length isn't divisible by 2
    if data.len() & 1 != 0 {
        let last_byte = data[data.len() - 1];
        sum += u64::from(u16::from_be_bytes([last_byte, 0]));
    }

    // do checksum magic
    let sum = (sum >> 16) + (sum & 0xffff);
    let sum = sum + (sum >> 16);
    let sum = !sum;
    sum as u16
}
