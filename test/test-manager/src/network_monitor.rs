use std::{
    future::poll_fn,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use futures::{channel::oneshot, pin_mut, StreamExt};
pub use pcap::Direction;
use pcap::PacketCodec;
use pnet_packet::{
    ethernet::EtherTypes, ip::IpNextHeaderProtocol, ipv4::Ipv4Packet, ipv6::Ipv6Packet,
    tcp::TcpPacket, udp::UdpPacket, Packet,
};

pub use pnet_packet::ip::IpNextHeaderProtocols as IpHeaderProtocols;

use crate::tests::config::TEST_CONFIG;
use crate::vm::network::CUSTOM_TUN_INTERFACE_NAME;

struct Codec {
    no_frame: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPacket {
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub protocol: IpNextHeaderProtocol,
}

impl PacketCodec for Codec {
    type Item = Option<ParsedPacket>;

    fn decode(&mut self, packet: pcap::Packet) -> Self::Item {
        if self.no_frame {
            // skip utun header specifying an address family
            #[cfg(target_os = "macos")]
            let data = &packet.data[4..];
            #[cfg(not(target_os = "macos"))]
            let data = packet.data;
            let ip_version = (data[0] & 0xf0) >> 4;

            return match ip_version {
                4 => Self::parse_ipv4(data),
                6 => Self::parse_ipv6(data),
                version => {
                    log::debug!("Ignoring unknown IP version: {version}");
                    None
                }
            };
        }

        let frame = pnet_packet::ethernet::EthernetPacket::new(packet.data).or_else(|| {
            log::error!("Received invalid ethernet frame");
            None
        })?;

        match frame.get_ethertype() {
            EtherTypes::Ipv4 => Self::parse_ipv4(frame.payload()),
            EtherTypes::Ipv6 => Self::parse_ipv6(frame.payload()),
            ethertype => {
                log::debug!("Ignoring unknown ethertype: {ethertype}");
                None
            }
        }
    }
}

impl Codec {
    fn parse_ipv4(payload: &[u8]) -> Option<ParsedPacket> {
        let packet = Ipv4Packet::new(payload).or_else(|| {
            log::error!("invalid v4 packet");
            None
        })?;

        let mut source = SocketAddr::new(IpAddr::V4(packet.get_source()), 0);
        let mut destination = SocketAddr::new(IpAddr::V4(packet.get_destination()), 0);

        let protocol = packet.get_next_level_protocol();

        match protocol {
            IpHeaderProtocols::Tcp => {
                let seg = TcpPacket::new(packet.payload()).or_else(|| {
                    log::error!("invalid TCP segment");
                    None
                })?;
                source.set_port(seg.get_source());
                destination.set_port(seg.get_destination());
            }
            IpHeaderProtocols::Udp => {
                let seg = UdpPacket::new(packet.payload()).or_else(|| {
                    log::error!("invalid UDP fragment");
                    None
                })?;
                source.set_port(seg.get_source());
                destination.set_port(seg.get_destination());
            }
            IpHeaderProtocols::Icmp => {}
            proto => log::debug!("ignoring v4 packet, transport/protocol type {proto}"),
        }

        Some(ParsedPacket {
            source,
            destination,
            protocol,
        })
    }

    fn parse_ipv6(payload: &[u8]) -> Option<ParsedPacket> {
        let packet = Ipv6Packet::new(payload).or_else(|| {
            log::error!("invalid v6 packet");
            None
        })?;

        let mut source = SocketAddr::new(IpAddr::V6(packet.get_source()), 0);
        let mut destination = SocketAddr::new(IpAddr::V6(packet.get_destination()), 0);

        let protocol = packet.get_next_header();
        match protocol {
            IpHeaderProtocols::Tcp => {
                let seg = TcpPacket::new(packet.payload()).or_else(|| {
                    log::error!("invalid TCP segment");
                    None
                })?;
                source.set_port(seg.get_source());
                destination.set_port(seg.get_destination());
            }
            IpHeaderProtocols::Udp => {
                let seg = UdpPacket::new(packet.payload()).or_else(|| {
                    log::error!("invalid UDP fragment");
                    None
                })?;
                source.set_port(seg.get_source());
                destination.set_port(seg.get_destination());
            }
            IpHeaderProtocols::Icmpv6 => {}
            proto => log::debug!("ignoring v6 packet, transport/protocol type {proto}"),
        }

        Some(ParsedPacket {
            source,
            destination,
            protocol,
        })
    }
}

#[derive(Debug)]
pub struct MonitorUnexpectedlyStopped(());

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
    pub no_frame: bool,
}

pub async fn start_packet_monitor(
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> PacketMonitor {
    start_packet_monitor_until(filter_fn, |_| true, monitor_options).await
}

pub async fn start_packet_monitor_until(
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> PacketMonitor {
    start_packet_monitor_for_interface(
        &TEST_CONFIG.host_bridge_name,
        filter_fn,
        should_continue_fn,
        monitor_options,
    )
    .await
}

pub async fn start_tunnel_packet_monitor_until(
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    mut monitor_options: MonitorOptions,
) -> PacketMonitor {
    monitor_options.no_frame = true;
    start_packet_monitor_for_interface(
        CUSTOM_TUN_INTERFACE_NAME,
        filter_fn,
        should_continue_fn,
        monitor_options,
    )
    .await
}

async fn start_packet_monitor_for_interface(
    interface: &str,
    filter_fn: impl Fn(&ParsedPacket) -> bool + Send + 'static,
    mut should_continue_fn: impl FnMut(&ParsedPacket) -> bool + Send + 'static,
    monitor_options: MonitorOptions,
) -> PacketMonitor {
    let dev = pcap::Capture::from_device(interface)
        .expect("Failed to open capture handle")
        .immediate_mode(true)
        .open()
        .expect("Failed to activate capture");

    if let Some(direction) = monitor_options.direction {
        dev.direction(direction).unwrap();
    }

    let dev = dev.setnonblock().unwrap();

    let (is_receiving_tx, is_receiving_rx) = oneshot::channel();

    let packet_stream = dev
        .stream(Codec {
            no_frame: monitor_options.no_frame,
        })
        .unwrap();
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let interface = interface.to_owned();

    let handle = tokio::spawn(async move {
        let mut monitor_result = MonitorResult {
            packets: vec![],
            discarded_packets: 0,
        };
        let mut packet_stream = packet_stream.fuse();

        let timeout = async move {
            if let Some(timeout) = monitor_options.timeout {
                tokio::time::sleep(timeout).await
            } else {
                futures::future::pending().await
            }
        };
        pin_mut!(timeout);

        let mut is_receiving_tx = Some(is_receiving_tx);

        loop {
            let mut next_packet_fut = packet_stream.next();
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
                    match maybe_next_packet {
                        Some(Ok(packet))=> {
                            if let Some(packet) = packet {
                                if !filter_fn(&packet) {
                                    log::debug!("{interface} \"{packet:?}\" does not match closure conditions");
                                    monitor_result.discarded_packets =
                                        monitor_result.discarded_packets.saturating_add(1);
                                } else {
                                    log::debug!("{interface} \"{packet:?}\" matches closure conditions");

                                    let should_continue = should_continue_fn(&packet);

                                    monitor_result.packets.push(packet);

                                    if !should_continue {
                                        break Ok(monitor_result);
                                    }
                                }
                            }
                        }
                        _ => {
                            log::error!("lost packet stream");
                            break Err(MonitorUnexpectedlyStopped(()));
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
