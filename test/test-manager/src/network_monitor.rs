use std::{
    fmt::Display,
    future::poll_fn,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::{Context, Result};
use futures::{Stream, StreamExt, channel::oneshot, future, pin_mut};
pub use pcap::Direction;
use pcap::PacketCodec;
use pnet_packet::{
    Packet, ethernet::EtherTypes, ip::IpNextHeaderProtocol, ipv4::Ipv4Packet, ipv6::Ipv6Packet,
    tcp::TcpPacket, udp::UdpPacket,
};

pub use pnet_packet::ip::IpNextHeaderProtocols as IpHeaderProtocols;

use crate::tests::config::TEST_CONFIG;

struct Codec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPacket {
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub protocol: IpNextHeaderProtocol,
    pub payload: Vec<u8>,
}

impl PacketCodec for Codec {
    type Item = Option<ParsedPacket>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        let frame = pnet_packet::ethernet::EthernetPacket::new(packet.data).or_else(|| {
            log::error!("Received invalid ethernet frame");
            None
        })?;

        match frame.get_ethertype() {
            EtherTypes::Ipv4 => parse_ipv4_packet(frame.payload()),
            EtherTypes::Ipv6 => parse_ipv6_packet(frame.payload()),
            ethertype => {
                log::trace!("Ignoring unknown ethertype: {ethertype}");
                None
            }
        }
    }
}

/// Parse a raw IP packet, e.g. one decrypted from the tunnel.
pub(crate) fn parse_ip_packet(data: &[u8]) -> Option<ParsedPacket> {
    let Some(&first_byte) = data.first() else {
        log::error!("Received empty packet");
        return None;
    };

    match (first_byte & 0xf0) >> 4 {
        4 => parse_ipv4_packet(data),
        6 => parse_ipv6_packet(data),
        version => {
            log::debug!("Ignoring unknown IP version: {version}");
            None
        }
    }
}

fn parse_ipv4_packet(payload: &[u8]) -> Option<ParsedPacket> {
    let packet = Ipv4Packet::new(payload).or_else(|| {
        log::error!("invalid v4 packet");
        None
    })?;

    let mut source = SocketAddr::new(IpAddr::V4(packet.get_source()), 0);
    let mut destination = SocketAddr::new(IpAddr::V4(packet.get_destination()), 0);
    let mut payload = vec![];

    let protocol = packet.get_next_level_protocol();
    match protocol {
        IpHeaderProtocols::Tcp => {
            let seg = TcpPacket::new(packet.payload()).or_else(|| {
                log::error!("invalid TCP segment");
                None
            })?;
            source.set_port(seg.get_source());
            destination.set_port(seg.get_destination());
            payload = seg.payload().to_vec();
        }
        IpHeaderProtocols::Udp => {
            let seg = UdpPacket::new(packet.payload()).or_else(|| {
                log::error!("invalid UDP fragment");
                None
            })?;
            source.set_port(seg.get_source());
            destination.set_port(seg.get_destination());
            payload = seg.payload().to_vec();
        }
        IpHeaderProtocols::Icmp => {}
        proto => log::warn!("ignoring v4 packet, transport/protocol type {proto}"),
    }

    Some(ParsedPacket {
        source,
        destination,
        protocol,
        payload,
    })
}

fn parse_ipv6_packet(payload: &[u8]) -> Option<ParsedPacket> {
    let packet = Ipv6Packet::new(payload).or_else(|| {
        log::error!("invalid v6 packet");
        None
    })?;

    let mut source = SocketAddr::new(IpAddr::V6(packet.get_source()), 0);
    let mut destination = SocketAddr::new(IpAddr::V6(packet.get_destination()), 0);
    let mut payload = vec![];

    let protocol = packet.get_next_header();
    match protocol {
        IpHeaderProtocols::Tcp => {
            let seg = TcpPacket::new(packet.payload()).or_else(|| {
                log::error!("invalid TCP segment");
                None
            })?;
            source.set_port(seg.get_source());
            destination.set_port(seg.get_destination());
            payload = seg.payload().to_vec();
        }
        IpHeaderProtocols::Udp => {
            let seg = UdpPacket::new(packet.payload()).or_else(|| {
                log::error!("invalid UDP fragment");
                None
            })?;
            source.set_port(seg.get_source());
            destination.set_port(seg.get_destination());
            payload = seg.payload().to_vec();
        }
        IpHeaderProtocols::Icmpv6 => {}
        proto => log::warn!("ignoring v6 packet, transport/protocol type {proto}"),
    }

    Some(ParsedPacket {
        source,
        destination,
        protocol,
        payload,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("Packet monitor stopped unexpectedly")]
pub struct MonitorUnexpectedlyStopped;

pub struct PacketMonitor {
    handle: tokio::task::JoinHandle<Result<MonitorResult, MonitorUnexpectedlyStopped>>,
    stop_tx: oneshot::Sender<()>,
}

pub struct MonitorResult {
    pub packets: Vec<ParsedPacket>,
    pub discarded_packets: usize,
}

impl PacketMonitor {
    /// Stop monitoring and return the result.
    pub async fn into_result(self) -> Result<MonitorResult, MonitorUnexpectedlyStopped> {
        let _ = self.stop_tx.send(());
        self.handle.await.expect("monitor panicked")
    }

    /// Wait for monitor to stop on its own.
    pub async fn wait(self) -> Result<MonitorResult, MonitorUnexpectedlyStopped> {
        self.handle.await.expect("monitor panicked")
    }
}

#[derive(Default)]
pub struct MonitorOptions {
    pub timeout: Option<Duration>,
    pub direction: Option<Direction>,
}

pub async fn start_packet_monitor(
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> Result<PacketMonitor> {
    start_packet_monitor_until(filter_fn, |_| true, monitor_options).await
}

pub async fn start_packet_monitor_until(
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> Result<PacketMonitor> {
    start_packet_monitor_for_interface(
        &TEST_CONFIG.host_bridge_name,
        filter_fn,
        should_continue_fn,
        monitor_options,
    )
    .await
}

async fn start_packet_monitor_for_interface(
    interface: &str,
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> Result<PacketMonitor> {
    let dev = pcap::Capture::from_device(interface)
        .expect("Failed to open capture handle")
        .immediate_mode(true)
        .open()
        .with_context(|| format!("Failed to activate capture on interface {interface}"))?;

    if let Some(direction) = monitor_options.direction {
        dev.direction(direction).unwrap();
    }

    let dev = dev.setnonblock().unwrap();

    // End the stream on the first capture error, and skip frames that could not be parsed.
    let packets = dev
        .stream(Codec)
        .unwrap()
        .take_while(|packet| future::ready(packet.is_ok()))
        .filter_map(|packet| future::ready(packet.ok().flatten()));

    Ok(spawn_monitor(
        interface.to_owned(),
        packets,
        filter_fn,
        should_continue_fn,
        monitor_options.timeout,
    )
    .await)
}

/// Collect packets from `packets` in the background until `should_continue_fn` returns `false`
/// for a packet, or `timeout` elapses.
///
/// Packets that do not match `filter_fn` are only counted. `label` identifies the source of the
/// packets in logs. The monitor is considered unexpectedly stopped if `packets` ends.
///
/// This returns once the stream has been polled for the first time.
pub(crate) async fn spawn_monitor(
    label: impl Display + Send + 'static,
    packets: impl Stream<Item = ParsedPacket> + Send + 'static,
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    mut should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    timeout: Option<Duration>,
) -> PacketMonitor {
    let (is_receiving_tx, is_receiving_rx) = oneshot::channel();
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        let mut monitor_result = MonitorResult {
            packets: vec![],
            discarded_packets: 0,
        };
        let mut packets = std::pin::pin!(packets);

        let timeout = async move {
            if let Some(timeout) = timeout {
                tokio::time::sleep(timeout).await
            } else {
                futures::future::pending().await
            }
        };
        pin_mut!(timeout);

        let mut is_receiving_tx = Some(is_receiving_tx);

        loop {
            let mut next_packet_fut = packets.next();
            let next_packet =
                poll_fn(|ctx| poll_and_notify(ctx, &mut next_packet_fut, &mut is_receiving_tx));

            tokio::select! {
                _stop = &mut stop_rx => {
                     log::trace!("stopping packet monitor");
                     break Ok(monitor_result);
                }
                _timeout = &mut timeout => {
                     log::info!("monitor timed out");
                     break Ok(monitor_result);
                }
                maybe_next_packet = next_packet => {
                    let Some(packet) = maybe_next_packet else {
                        log::error!("lost packet stream");
                        break Err(MonitorUnexpectedlyStopped);
                    };

                    if !filter_fn(&packet) {
                        log::trace!("{label} \"{packet:?}\" does not match closure conditions");
                        monitor_result.discarded_packets =
                            monitor_result.discarded_packets.saturating_add(1);
                    } else {
                        log::trace!("{label} \"{packet:?}\" matches closure conditions");

                        let should_continue = should_continue_fn(&packet);

                        monitor_result.packets.push(packet);

                        if !should_continue {
                            break Ok(monitor_result);
                        }
                    }
                }
            }
        }
    });

    // Wait for the loop to start receiving its first packet
    let _ = is_receiving_rx.await;

    PacketMonitor { stop_tx, handle }
}

/// Poll the future once and notify `tx` that it has been polled. Then return
/// the result of this polling.
fn poll_and_notify<F: std::future::Future<Output = O> + Unpin, O>(
    context: &mut std::task::Context<'_>,
    fut: &mut F,
    tx: &mut Option<oneshot::Sender<()>>,
) -> std::task::Poll<O> {
    let result = std::pin::Pin::new(fut).poll(context);
    if let Some(tx) = tx.take() {
        let _ = tx.send(());
    }
    result
}
