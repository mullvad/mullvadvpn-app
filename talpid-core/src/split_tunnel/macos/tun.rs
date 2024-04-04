//! This module implements a tunnel capable of redirecting traffic through one of two interfaces,
//! either the default interface or a VPN tunnel interface.

use super::{
    bindings::{pcap_create, pcap_set_want_pktap, pktap_header, PCAP_ERRBUF_SIZE},
    bpf,
    default::DefaultInterface,
};
use futures::{Stream, StreamExt};
use libc::{AF_INET, AF_INET6};
use pcap::PacketCodec;
use pnet::{
    packet::{
        ethernet::{EtherTypes, MutableEthernetPacket},
        ip::IpNextHeaderProtocols,
        ipv4::MutableIpv4Packet,
        ipv6::MutableIpv6Packet,
        tcp::MutableTcpPacket,
        udp::MutableUdpPacket,
        MutablePacket, Packet,
    },
    util::MacAddr,
};
use std::ffi::c_uint;
use std::{
    ffi::CStr,
    io::{self, IoSlice, Write},
    net::{Ipv4Addr, Ipv6Addr},
    ptr::NonNull,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::broadcast,
};
use tun::Device;

/// IP address used by the ST utun
const ST_IFACE_IPV4: Ipv4Addr = Ipv4Addr::new(10, 123, 123, 123);
const ST_IFACE_IPV6: Ipv6Addr = Ipv6Addr::new(0xfd, 0x12, 0x12, 0x12, 0xfe, 0xfe, 0xfe, 0xfe);

const DEFAULT_BUFFER_SIZE: c_uint = 16 * 1024 * 1024;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to create split tunnel utun
    #[error("Failed to create split tunnel interface")]
    CreateSplitTunnelInterface(#[source] tun::Error),
    /// Failed to set IPv6 address on tunnel interface
    #[error("Failed to set IPv6 address on tunnel interface")]
    AddIpv6Address(#[source] io::Error),
    /// Failed to begin capture on split tunnel utun
    #[error("Failed to begin capture on split tunnel utun")]
    CaptureSplitTunnelDevice(#[source] pcap::Error),
    /// Failed to set direction on capture
    #[error("Failed to set direction on pcap")]
    SetDirection(#[source] pcap::Error),
    /// Failed to enable nonblocking I/O
    #[error("Failed to enable nonblocking I/O")]
    EnableNonblock(#[source] pcap::Error),
    /// pcap_create failed
    #[error("pcap_create failed: {}", _0)]
    CreatePcap(String),
    /// Failed to create packet stream
    #[error("Failed to create packet stream")]
    CreateStream(#[source] pcap::Error),
    /// Failed to get next packet
    #[error("Failed to get next packet")]
    GetNextPacket(#[source] pcap::Error),
    /// Failed to create BPF device for default interface
    #[error("Failed to create BPF device for default interface")]
    CreateDefaultBpf(#[source] bpf::Error),
    /// Failed to configure BPF device for default interface
    #[error("Failed to configure BPF device for default interface")]
    ConfigDefaultBpf(#[source] bpf::Error),
    /// Failed to retrieve BPF buffer size
    #[error("Failed to retrieve BPF buffer size")]
    GetBpfBufferSize(#[source] bpf::Error),
    /// Failed to create BPF device for VPN tunnel
    #[error("Failed to create BPF device for VPN tunnel")]
    CreateVpnBpf(#[source] bpf::Error),
    /// Failed to configure BPF device for VPN
    #[error("Failed to configure BPF device for VPN tunnel")]
    ConfigVpnBpf(#[source] bpf::Error),
    /// Failed to stop tunnel redirection
    #[error("Failed to stop tunnel redirection")]
    StopRedirect,
    /// Failed to receive next pktap packet
    #[error("Failed to receive next pktap packet")]
    PktapStreamStopped,
}

/// Routing decision made for an outbound packet
#[derive(Debug, Clone, Copy)]
pub enum RoutingDecision {
    /// Send outgoing packets through the default interface
    DefaultInterface,
    /// Send outgoing packets through the VPN tunnel
    VpnTunnel,
    /// Drop the packet
    Drop,
}

/// VPN tunnel interface details
#[derive(Debug, Clone)]
pub struct VpnInterface {
    /// VPN tunnel interface name
    pub name: String,
    /// VPN tunnel IPv4 address
    pub v4_address: Option<Ipv4Addr>,
    /// VPN tunnel IPv6 address
    pub v6_address: Option<Ipv6Addr>,
}

pub struct SplitTunnelHandle {
    redir_handle: RedirectHandle,
    tun_name: String,
}

impl SplitTunnelHandle {
    pub async fn shutdown(self) -> Result<(), Error> {
        log::debug!("Shutting down split tunnel");
        let _ = self.redir_handle.stop().await?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.tun_name
    }

    pub async fn set_interfaces(
        mut self,
        default_interface: DefaultInterface,
        vpn_interface: Option<VpnInterface>,
    ) -> Result<Self, Error> {
        self.redir_handle = self
            .redir_handle
            .set_interfaces(default_interface, vpn_interface)
            .await?;
        Ok(self)
    }
}

/// Create split tunnel device and handle all packets using `classify`. Handle any changes to the
/// default interface or gateway.
///
/// # Note
///
/// `classify` receives an Ethernet frame. The Ethernet header is not valid at this point, however.
/// Only the IP header and payload are.
pub async fn create_split_tunnel(
    default_interface: DefaultInterface,
    vpn_interface: Option<VpnInterface>,
    classify: impl Fn(&PktapPacket) -> RoutingDecision + Send + 'static,
) -> Result<SplitTunnelHandle, Error> {
    let mut tun_config = tun::configure();
    tun_config.address(ST_IFACE_IPV4).up();
    let tun_device =
        tun::create_as_async(&tun_config).map_err(Error::CreateSplitTunnelInterface)?;
    let tun_name = tun_device.get_ref().name().to_owned();

    // Add IPv6 address
    let output = tokio::process::Command::new("ifconfig")
        .args([&tun_name, "inet6", &ST_IFACE_IPV6.to_string(), "alias"])
        .output()
        .await
        .map_err(Error::AddIpv6Address)?;
    if !output.status.success() {
        return Err(Error::AddIpv6Address(io::Error::new(
            io::ErrorKind::Other,
            "ifconfig failed",
        )));
    }

    let redir_handle =
        redirect_packets(tun_device, default_interface, vpn_interface, classify).await?;

    Ok(SplitTunnelHandle {
        redir_handle,
        tun_name,
    })
}

type PktapStream = std::pin::Pin<Box<dyn Stream<Item = Result<PktapPacket, Error>> + Send>>;

struct RedirectHandle {
    /// A sender that gracefully stops the other tasks (`ingress_task`, and `egress_task`)
    abort_tx: broadcast::Sender<()>,
    /// Task that handles incoming packets. On completion, it returns a handle for the ST utun
    ingress_task: tokio::task::JoinHandle<tun::AsyncDevice>,
    /// Task that handles outgoing packets. On completion, it returns a handle for the pktap, as
    /// well as the function used to classify packets
    egress_task: tokio::task::JoinHandle<
        Result<
            (
                PktapStream,
                Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static>,
            ),
            Error,
        >,
    >,
}

impl RedirectHandle {
    pub async fn stop(self) -> Result<(), Error> {
        let _ = self.abort_tx.send(());
        let _ = self.ingress_task.await.map_err(|_| Error::StopRedirect)?;
        let _ = self.egress_task.await.map_err(|_| Error::StopRedirect)??;
        Ok(())
    }

    pub async fn set_interfaces(
        self,
        default_interface: DefaultInterface,
        vpn_interface: Option<VpnInterface>,
    ) -> Result<Self, Error> {
        let _ = self.abort_tx.send(());

        let st_utun = self.ingress_task.await.map_err(|_| Error::StopRedirect)?;

        let (pktap_stream, classify) =
            self.egress_task.await.map_err(|_| Error::StopRedirect)??;

        let new_handle = redirect_packets_for_pktap_stream(
            st_utun,
            pktap_stream,
            default_interface,
            vpn_interface,
            classify,
        )
        .await?;

        Ok(new_handle)
    }
}

/// Monitor outgoing traffic on `st_tun_device` using a pktap. A routing decision is
/// made for each packet using `classify`. Based on this, a packet is forced out on either
/// `default_interface` or `vpn_interface`.
///
/// # Note
///
/// `classify` receives an Ethernet frame. The Ethernet header is not valid at this point, however.
/// Only the IP header and payload are.
async fn redirect_packets(
    st_tun_device: tun::AsyncDevice,
    default_interface: DefaultInterface,
    vpn_interface: Option<VpnInterface>,
    classify: impl Fn(&PktapPacket) -> RoutingDecision + Send + 'static,
) -> Result<RedirectHandle, Error> {
    let pktap_stream = capture_outbound_packets(st_tun_device.get_ref().name())?;
    redirect_packets_for_pktap_stream(
        st_tun_device,
        Box::pin(pktap_stream),
        default_interface,
        vpn_interface,
        Box::new(classify),
    )
    .await
}

/// Monitor outgoing traffic on `st_tun_device` using `pktap_stream`. A routing decision is made for
/// each packet using `classify`. Based on this, a packet is forced out on either
/// `default_interface` or `vpn_interface`.
///
/// # Note
///
/// `classify` receives an Ethernet frame. The Ethernet header is not valid at this point, however.
/// Only the IP header and payload are.
async fn redirect_packets_for_pktap_stream(
    st_tun_device: tun::AsyncDevice,
    pktap_stream: PktapStream,
    default_interface: DefaultInterface,
    vpn_interface: Option<VpnInterface>,
    classify: Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static>,
) -> Result<RedirectHandle, Error> {
    let default_dev = bpf::Bpf::open().map_err(Error::CreateDefaultBpf)?;
    let read_buffer_size = default_dev
        .set_buffer_size(DEFAULT_BUFFER_SIZE)
        .map_err(Error::ConfigDefaultBpf)?;
    default_dev
        .set_interface(&default_interface.name)
        .map_err(Error::ConfigDefaultBpf)?;
    default_dev
        .set_immediate(true)
        .map_err(Error::ConfigDefaultBpf)?;
    default_dev
        .set_see_sent(false)
        .map_err(Error::ConfigDefaultBpf)?;

    let (default_read, default_write) = default_dev.split().map_err(Error::ConfigDefaultBpf)?;
    let default_stream =
        bpf::BpfStream::from_read_half(default_read).map_err(Error::CreateDefaultBpf)?;

    let (abort_tx, abort_rx) = broadcast::channel(5);

    let ingress_task: tokio::task::JoinHandle<tun::AsyncDevice> = tokio::spawn(run_ingress_task(
        st_tun_device,
        default_stream,
        read_buffer_size,
        vpn_interface.clone(),
        abort_rx,
    ));

    let egress_abort_rx = abort_tx.subscribe();
    let egress_task = tokio::spawn(run_egress_task(
        pktap_stream,
        classify,
        default_interface,
        default_write,
        vpn_interface,
        egress_abort_rx,
    ));

    Ok(RedirectHandle {
        abort_tx,
        ingress_task,
        egress_task,
    })
}

/// Read incoming packets on the default interface and send them back to the ST utun.
async fn run_ingress_task(
    st_tun_device: tun::AsyncDevice,
    mut default_read: bpf::BpfStream,
    read_buffer_size: usize,
    vpn_interface: Option<VpnInterface>,
    mut abort_rx: broadcast::Receiver<()>,
) -> tun::AsyncDevice {
    let mut read_buffer = vec![0u8; read_buffer_size];
    log::trace!("Default BPF reader buffer size: {:?}", read_buffer.len());

    let vpn_v4 = vpn_interface.as_ref().and_then(|iface| iface.v4_address);
    let vpn_v6 = vpn_interface.and_then(|iface| iface.v6_address);

    let (mut tun_reader, mut tun_writer) = tokio::io::split(st_tun_device);

    let mut abort_read_rx = abort_rx.resubscribe();

    // Swallow all data written to the tun by reading from it
    // Do this to prevent the read buffer from filling up and preventing writes
    let mut garbage: Vec<u8> = vec![0u8; 8 * 1024 * 1024];
    let dummy_read = tokio::spawn(async move {
        loop {
            tokio::select! {
                result = tun_reader.read(&mut garbage) => {
                    if result.is_err() {
                        break;
                    }
                }
                Ok(()) | Err(_) = abort_read_rx.recv() => {
                    break;
                }
            }
        }
        tun_reader
    });

    // Write data incoming on the default interface to the ST utun
    let tun_writer = loop {
        tokio::select! {
            result = default_read.read(&mut read_buffer) => {
                let Ok(read_n) = result else {
                    break tun_writer;
                };
                let read_data = &mut read_buffer[0..read_n];

                let mut iter = bpf::BpfIterMut::new(read_data);
                while let Some(payload) = iter.next() {
                    handle_incoming_data(&mut tun_writer, payload, vpn_v4, vpn_v6).await;
                }
            }
            Ok(()) | Err(_) = abort_rx.recv() => {
                break tun_writer;
            }
        }
    };

    let tun_reader = dummy_read.await.unwrap();

    log::debug!("Stopping ST utun ingress");

    tun_reader.unsplit(tun_writer)
}

/// Read outgoing packets and send them out on either the default interface or VPN interface,
/// based on the result of `classify`.
async fn run_egress_task(
    mut pktap_stream: PktapStream,
    mut classify: Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static>,
    default_interface: DefaultInterface,
    mut default_write: bpf::WriteHalf,
    vpn_interface: Option<VpnInterface>,
    mut abort_rx: broadcast::Receiver<()>,
) -> Result<
    (
        PktapStream,
        Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static>,
    ),
    Error,
> {
    let mut vpn_dev = if let Some(ref vpn_interface) = vpn_interface {
        let vpn_dev = bpf::Bpf::open().map_err(Error::CreateVpnBpf)?;
        vpn_dev
            .set_interface(&vpn_interface.name)
            .map_err(Error::ConfigVpnBpf)?;
        vpn_dev.set_immediate(true).map_err(Error::ConfigVpnBpf)?;
        vpn_dev.set_see_sent(false).map_err(Error::ConfigVpnBpf)?;
        Some(vpn_dev)
    } else {
        None
    };

    loop {
        tokio::select! {
            packet = pktap_stream.next() => {
                let mut packet = packet.ok_or_else(|| {
                    log::debug!("packet stream closed");
                    Error::PktapStreamStopped
                })??;

                let vpn_device = match (vpn_interface.as_ref(), vpn_dev.as_mut()) {
                    (Some(interface), Some(device)) => Some((interface, device)),
                    (None, None) => None,
                    _ => unreachable!("missing tun interface or addresses"),
                };

                classify = classify_and_send(classify, &mut packet, &default_interface, &mut default_write, vpn_device).await;
            }
            Ok(()) | Err(_) = abort_rx.recv() => {
                log::debug!("stopping packet processing");
                break Ok((pktap_stream, classify));
            }
        }
    }
}

async fn classify_and_send(
    classify: Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static>,
    packet: &mut PktapPacket,
    default_interface: &DefaultInterface,
    default_write: &mut bpf::WriteHalf,
    vpn_interface: Option<(&VpnInterface, &mut bpf::Bpf)>,
) -> Box<dyn Fn(&PktapPacket) -> RoutingDecision + Send + 'static> {
    match classify(&packet) {
        RoutingDecision::DefaultInterface => match packet.frame.get_ethertype() {
            EtherTypes::Ipv4 => {
                let Some(ref addrs) = default_interface.v4_addrs else {
                    log::trace!("dropping IPv4 packet since there's no default route");
                    return classify;
                };
                let gateway_address = MacAddr::from(addrs.gateway_address.into_bytes());
                packet.frame.set_destination(gateway_address);
                let Some(mut ip) = MutableIpv4Packet::new(packet.frame.payload_mut()) else {
                    log::error!("dropping invalid IPv4 packet");
                    return classify;
                };
                fix_ipv4_checksums(&mut ip, Some(addrs.source_ip), None);
                if let Err(error) = default_write.write(packet.frame.packet()) {
                    log::error!("Failed to forward to default device: {error}");
                }
            }
            EtherTypes::Ipv6 => {
                let Some(ref addrs) = default_interface.v6_addrs else {
                    log::trace!("dropping IPv6 packet since there's no default route");
                    return classify;
                };
                let gateway_address = MacAddr::from(addrs.gateway_address.into_bytes());
                packet.frame.set_destination(gateway_address);
                let Some(mut ip) = MutableIpv6Packet::new(packet.frame.payload_mut()) else {
                    log::error!("dropping invalid IPv6 packet");
                    return classify;
                };
                fix_ipv6_checksums(&mut ip, Some(addrs.source_ip), None);
                if let Err(error) = default_write.write(packet.frame.packet()) {
                    log::error!("Failed to forward to default device: {error}");
                }
            }
            other => log::error!("unknown ethertype: {other}"),
        },
        RoutingDecision::VpnTunnel => {
            let Some((vpn_interface, vpn_write)) = vpn_interface else {
                log::trace!("dropping IP packet since there's no tun route");
                return classify;
            };

            match packet.frame.get_ethertype() {
                EtherTypes::Ipv4 => {
                    let Some(addr) = vpn_interface.v4_address else {
                        log::trace!("dropping IPv4 packet since there's no tun route");
                        return classify;
                    };
                    let Some(mut ip) = MutableIpv4Packet::new(packet.frame.payload_mut()) else {
                        log::error!("dropping invalid IPv4 packet");
                        return classify;
                    };
                    fix_ipv4_checksums(&mut ip, Some(addr), None);
                    if let Err(error) = vpn_write.write(packet.frame.payload()) {
                        log::error!("Failed to forward to tun device: {error}");
                    }
                }
                EtherTypes::Ipv6 => {
                    let Some(addr) = vpn_interface.v6_address else {
                        log::trace!("dropping IPv6 packet since there's no tun route");
                        return classify;
                    };
                    let Some(mut ip) = MutableIpv6Packet::new(packet.frame.payload_mut()) else {
                        log::error!("dropping invalid IPv6 packet");
                        return classify;
                    };
                    fix_ipv6_checksums(&mut ip, Some(addr), None);
                    if let Err(error) = vpn_write.write(packet.frame.payload()) {
                        log::error!("Failed to forward to tun device: {error}");
                    }
                }
                other => log::error!("unknown ethertype: {other}"),
            }
        }
        RoutingDecision::Drop => {
            log::trace!("Dropped packet from pid {}", packet.header.pth_pid);
        }
    }
    classify
}

async fn handle_incoming_data(
    tun_writer: &mut tokio::io::WriteHalf<tun::AsyncDevice>,
    payload: &mut [u8],
    vpn_v4: Option<Ipv4Addr>,
    vpn_v6: Option<Ipv6Addr>,
) {
    let Some(mut frame) = MutableEthernetPacket::new(payload) else {
        log::trace!("discarding non-Ethernet frame");
        return;
    };

    match frame.get_ethertype() {
        EtherTypes::Ipv4 => {
            let Some(vpn_addr) = vpn_v4 else {
                log::trace!("discarding incoming IPv4 packet: no tun V4 addr");
                return;
            };
            let Some(ip) = MutableIpv4Packet::new(frame.payload_mut()) else {
                log::trace!("discarding non-IPv4 packet");
                return;
            };
            handle_incoming_data_v4(tun_writer, ip, vpn_addr).await;
        }
        EtherTypes::Ipv6 => {
            let Some(vpn_addr) = vpn_v6 else {
                log::trace!("discarding incoming IPv6 packet: no tun V6 addr");
                return;
            };
            let Some(ip) = MutableIpv6Packet::new(frame.payload_mut()) else {
                log::trace!("discarding non-IPv6 packet");
                return;
            };
            handle_incoming_data_v6(tun_writer, ip, vpn_addr).await;
        }
        ethertype => {
            log::trace!("discarding non-IP frame: {ethertype}");
        }
    }
}

async fn handle_incoming_data_v4(
    tun_writer: &mut tokio::io::WriteHalf<tun::AsyncDevice>,
    mut ip: MutableIpv4Packet<'_>,
    vpn_addr: Ipv4Addr,
) {
    if ip.get_destination() == vpn_addr {
        // Drop attempt to send packets to tun IP on the real interface
        log::trace!("Dropping packet to VPN IP on default interface");
        return;
    }

    fix_ipv4_checksums(&mut ip, None, Some(vpn_addr));

    const BSD_LB_HEADER: &[u8] = &(AF_INET as u32).to_be_bytes();
    if let Err(error) = tun_writer
        .write_vectored(&[IoSlice::new(BSD_LB_HEADER), IoSlice::new(ip.packet())])
        .await
    {
        log::error!("Failed to redirect incoming IPv4 packet: {error}");
    }
}

async fn handle_incoming_data_v6(
    tun_writer: &mut tokio::io::WriteHalf<tun::AsyncDevice>,
    mut ip: MutableIpv6Packet<'_>,
    vpn_addr: Ipv6Addr,
) {
    if ip.get_destination() == vpn_addr {
        // Drop attempt to send packets to tun IP on the real interface
        log::trace!("Dropping packet to VPN IP on default interface");
        return;
    }

    fix_ipv6_checksums(&mut ip, None, Some(vpn_addr));

    const BSD_LB_HEADER: &[u8] = &(AF_INET6 as u32).to_be_bytes();
    if let Err(error) = tun_writer
        .write_vectored(&[IoSlice::new(BSD_LB_HEADER), IoSlice::new(ip.packet())])
        .await
    {
        log::error!("Failed to redirect incoming IPv6 packet: {error}");
    }
}

// Recalculate L3 and L4 checksums. Silently fail on error
fn fix_ipv4_checksums(
    ip: &mut MutableIpv4Packet<'_>,
    new_source: Option<Ipv4Addr>,
    new_destination: Option<Ipv4Addr>,
) {
    // Update source and update checksums
    if let Some(source_ip) = new_source {
        ip.set_source(source_ip);
    }
    if let Some(dest_ip) = new_destination {
        ip.set_destination(dest_ip);
    }

    let source_ip = ip.get_source();
    let destination_ip = ip.get_destination();

    match ip.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(mut tcp) = MutableTcpPacket::new(ip.payload_mut()) {
                use pnet::packet::tcp::ipv4_checksum;
                tcp.set_checksum(ipv4_checksum(
                    &tcp.to_immutable(),
                    &source_ip,
                    &destination_ip,
                ));
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(mut udp) = MutableUdpPacket::new(ip.payload_mut()) {
                use pnet::packet::udp::ipv4_checksum;
                udp.set_checksum(ipv4_checksum(
                    &udp.to_immutable(),
                    &source_ip,
                    &destination_ip,
                ));
            }
        }
        _ => (),
    }

    ip.set_checksum(pnet::packet::ipv4::checksum(&ip.to_immutable()));
}

// Recalculate L3 and L4 checksums. Silently fail on error
fn fix_ipv6_checksums(
    ip: &mut MutableIpv6Packet<'_>,
    new_source: Option<Ipv6Addr>,
    new_destination: Option<Ipv6Addr>,
) {
    // Update source and update checksums
    if let Some(source_ip) = new_source {
        ip.set_source(source_ip);
    }
    if let Some(dest_ip) = new_destination {
        ip.set_destination(dest_ip);
    }

    let source_ip = ip.get_source();
    let destination_ip = ip.get_destination();

    match ip.get_next_header() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(mut tcp) = MutableTcpPacket::new(ip.payload_mut()) {
                use pnet::packet::tcp::ipv6_checksum;
                tcp.set_checksum(ipv6_checksum(
                    &tcp.to_immutable(),
                    &source_ip,
                    &destination_ip,
                ));
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(mut udp) = MutableUdpPacket::new(ip.payload_mut()) {
                use pnet::packet::udp::ipv6_checksum;
                udp.set_checksum(ipv6_checksum(
                    &udp.to_immutable(),
                    &source_ip,
                    &destination_ip,
                ));
            }
        }
        _ => (),
    }
}

/// This returns a stream of outbound packets on a utun tunnel.
///
/// * `utun_iface`- name of a utun interface to capture packets on. Note that if this does not
///   exist, the function will not fail, but the stream will never return anything.
fn capture_outbound_packets(
    utun_iface: &str,
) -> Result<impl Stream<Item = Result<PktapPacket, Error>> + Send, Error> {
    let cap = pktap_capture()?
        .immediate_mode(true)
        .open()
        .map_err(Error::CaptureSplitTunnelDevice)?;

    cap.direction(pcap::Direction::Out)
        .map_err(Error::SetDirection)?;

    let cap = cap.setnonblock().map_err(Error::EnableNonblock)?;
    let stream = cap
        .stream(PktapCodec::new(utun_iface.to_owned()))
        .map_err(Error::CreateStream)?
        .filter_map(|pkt| async { pkt.map_err(Error::GetNextPacket).transpose() });

    Ok(stream)
}

struct PktapCodec {
    interface: String,
}

impl PktapCodec {
    fn new(interface: String) -> PktapCodec {
        Self { interface }
    }
}

#[derive(Debug)]
pub struct PktapPacket {
    pub header: pktap_header,
    pub frame: MutableEthernetPacket<'static>,
}

impl PacketCodec for PktapCodec {
    type Item = Option<PktapPacket>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        assert!(packet.data.len() >= std::mem::size_of::<pktap_header>());

        // SAFETY: packet is large enough to contain the header
        let header: &pktap_header = unsafe { &*(packet.data.as_ptr() as *const pktap_header) };

        let pth_length = header.pth_length as usize;
        let data = if pth_length < packet.data.len() {
            // SAFETY: The actual payload contains more than 'pth_length' bytes
            &packet.data[pth_length..]
        } else if pth_length == packet.data.len() {
            &[]
        } else {
            return None;
        };

        let iface = unsafe { CStr::from_ptr(header.pth_ifname.as_ptr() as *const _) };
        if iface.to_bytes() != self.interface.as_bytes() {
            return None;
        }

        // TODO: Wasteful. Could share single buffer if handling one frame at a time (assuming no
        // concurrency is needed). Allocating the frame here is purely done for efficiency reasons.
        let mut frame = MutableEthernetPacket::owned(vec![0u8; 14 + data.len() - 4]).unwrap();

        let raw_family = i32::from_ne_bytes(data[0..4].try_into().unwrap());
        let ethertype = match raw_family {
            AF_INET => EtherTypes::Ipv4,
            AF_INET6 => EtherTypes::Ipv6,
            _ => return None,
        };

        frame.set_ethertype(ethertype);
        frame.set_payload(&data[4..]);

        Some(PktapPacket {
            header: header.to_owned(),
            frame,
        })
    }
}

/// Create a pktap interface using `libpcap`
fn pktap_capture() -> Result<pcap::Capture<pcap::Inactive>, Error> {
    // We want to create a pktap "pseudo-device" and capture data on it using a bpf device.
    // This provides packet data plus a pktap header including process information.
    // libpcap will do the heavy lifting for us if we simply request a "pktap" device.

    let mut errbuf = [0u8; PCAP_ERRBUF_SIZE as usize];

    let pcap = unsafe { pcap_create(c"pktap".as_ptr(), errbuf.as_mut_ptr() as _) };
    if pcap.is_null() {
        let errstr = CStr::from_bytes_until_nul(&errbuf)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        return Err(Error::CreatePcap(errstr));
    }
    unsafe { pcap_set_want_pktap(pcap, 1) };

    // TODO: Upstream setting "want pktap" directly on Capture
    //       If we had that, we could have simply used pcap::Capture::from_device("pktap")
    // TODO: Also upstream exposure of a raw handle to pcap_t on Capture<Inactive>

    // just casting a pointer to a private type using _. that's fine, apparently
    Ok(pcap::Capture::from(unsafe {
        NonNull::new_unchecked(pcap as *mut _)
    }))
}
