//! Routes the traffic arriving on the tunnel device out through real OS sockets.
//!
//! One task reads the device and demultiplexes by protocol; one task writes everything back. The
//! read loop never waits on an upstream socket, so a stalled connection cannot hold up the rest.

use std::sync::Arc;

use bytes::Bytes;
use smoltcp::wire::{IpProtocol, Ipv4Packet, TcpPacket, UdpPacket};
use tokio::sync::{Notify, mpsc};
use tun_rs::AsyncDevice;
use udp::UdpRouter;

use super::TunnelDevices;

mod tcp;
mod udp;

/// Largest packet read off the tunnel device in one go.
const MAX_PACKET_SIZE: usize = u16::MAX as usize;
/// Packets buffered for the tunnel device writer.
const EGRESS_QUEUE_DEPTH: usize = 512;

pub struct Router {
    udp: UdpRouter,
    tcp_packet_tx: mpsc::Sender<Ipv4Packet<Bytes>>,
}

impl Router {
    pub fn spawn(devices: TunnelDevices) {
        let TunnelDevices { sink, source } = devices;
        let sink = Arc::new(sink);
        let source = Arc::new(source);
        let sink_clone = sink.clone();
        let source_clone = source.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];
            println!("wtf?");
            while let Ok(bytes_received) = sink.recv(&mut buf).await {
                println!("RECEIVING RETURN TRAFFIC FROM SINK");
                if let Err(err) = source_clone.send(&buf[..bytes_received]).await {
                    log::error!("Failed to write to source utun: {err}");
                }
            }

        });
        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];

            println!("why is this working at all?");
            while let Ok(bytes_received) = source.recv(&mut buf).await {
                println!("RECEIVING SOURCE TRAFFIC");
                if let Err(err) = sink_clone.send(&buf[..bytes_received]).await {
                    log::error!("Failed to write to sink utun: {err}");
                }
            }

        });
        // let (egress_tx, egress_rx) = mpsc::channel(EGRESS_QUEUE_DEPTH);
        // let (tcp_packet_tx, tcp_notify) = tcp::spawn(egress_tx.clone(), mtu);
        // spawn_tunnel_writer(tunnel_device.clone(), egress_rx, tcp_notify);

        // let mut router = Self {
        //     udp: UdpRouter::new(egress_tx, mtu),
        //     tcp_packet_tx,
        // };

        // let device_name = tunnel_device
        //     .name()
        //     .unwrap_or_else(|_| "the tunnel device".to_owned());

        // tokio::spawn(async move {
        //     // Reading into one scratch buffer and copying out exactly what arrived beats
        //     // re-zeroing a `BytesMut` per packet, and stops a whole 64 KiB allocation being held
        //     // alive by whichever packet off it is still queued.
        //     let mut buffer = vec![0u8; MAX_PACKET_SIZE];
        //     let mut seen_a_packet = false;

        //     loop {
        //         let packet_bytes = match tunnel_device.recv(&mut buffer).await {
        //             Ok(bytes_read) => Bytes::copy_from_slice(&buffer[..bytes_read]),
        //             Err(err) => {
        //                 log::error!("Failed to read from {device_name}, giving up: {err}");
        //                 return;
        //             }
        //         };

        //         // Per-packet logging is `trace`, not `debug`: formatting and writing a line per
        //         // packet costs more than routing it, so it must not ride along with the
        //         // connection-level logs. Kept inside the macros so it is skipped when disabled.
        //         if Ipv4Packet::new_checked(packet_bytes.as_ref()).is_err() {
        //             log::trace!("rx {}, discarded", describe(&packet_bytes));
        //             continue;
        //         }
        //         log::trace!("rx {}", describe(&packet_bytes));

        //         router.udp.sweep_idle_flows();
        //         router.process_packet(packet_bytes).await;
        //     }
        // });
    }

    async fn process_packet(&mut self, packet_bytes: Bytes) {
        let ipv4_packet = Ipv4Packet::new_unchecked(packet_bytes.as_ref());
        let source = ipv4_packet.src_addr();
        let destination = ipv4_packet.dst_addr();
        let protocol = ipv4_packet.next_header();
        // The read loop already checked this packet, so the header bounds it reports are sound.
        let payload = usize::from(ipv4_packet.header_len())..usize::from(ipv4_packet.total_len());

        // TODO: use block list to see if packet should be dropped
        // TODO: check if active packet capture is taking place - copy packet over

        match protocol {
            IpProtocol::Udp => {
                self.udp
                    .route(source, destination, packet_bytes.slice(payload))
                    .await
            }
            IpProtocol::Tcp => self.forward_tcp(packet_bytes),
            // Silence makes the client wait out its own timeout, so at least say what we dropped.
            protocol => log::trace!("No route for {protocol:?} from {source} to {destination}"),
        }
    }

    /// Hand a packet to the TCP router without waiting for it.
    ///
    /// Blocking here would let a congested TCP stack stall the UDP flows and every other TCP
    /// connection, so a full queue drops the packet and lets the client retransmit.
    fn forward_tcp(&self, packet_bytes: Bytes) {
        if let Err(err) = self
            .tcp_packet_tx
            .try_send(Ipv4Packet::new_unchecked(packet_bytes))
        {
            log::debug!("Dropping a TCP packet: {err}");
        }
    }
}

/// Write egress packets to the tunnel device, in the order they were produced.
///
/// `tcp_notify` wakes the TCP stack once there is room again, since it holds back segments it
/// could not queue rather than dropping them.
fn spawn_tunnel_writer(
    tunnel_device: Arc<AsyncDevice>,
    mut egress_rx: mpsc::Receiver<Bytes>,
    tcp_notify: Arc<Notify>,
) {
    tokio::spawn(async move {
        // Taking the whole queue per wakeup, and notifying once for it, keeps the scheduler out of
        // the way when packets arrive faster than one at a time.
        let mut batch = Vec::with_capacity(EGRESS_QUEUE_DEPTH);

        while egress_rx.recv_many(&mut batch, EGRESS_QUEUE_DEPTH).await > 0 {
            for packet in batch.drain(..) {
                log::trace!("tx {}", describe(&packet));
                match tunnel_device.send(&packet).await {
                    Ok(written) if written != packet.len() => {
                        log::warn!(
                            "Only {written} of {} bytes reached the tunnel device",
                            packet.len()
                        );
                    }
                    Ok(_) => (),
                    Err(err) => log::error!("Failed to write a packet to the tunnel device: {err}"),
                }
            }
            tcp_notify.notify_one();
        }
    });
}

/// One line describing a packet, for the per-packet logs.
///
/// Deliberately parses from scratch rather than trusting the caller, so that packets we reject can
/// be described too. That is usually the interesting case.
fn describe(packet: &[u8]) -> String {
    let Ok(ip) = Ipv4Packet::new_checked(packet) else {
        return match packet.first().map(|byte| byte >> 4) {
            Some(6) => format!("IPv6 ({} bytes), which we do not route", packet.len()),
            Some(version) => format!(
                "unparseable IPv{version} ({} bytes), starting {:02x?}",
                packet.len(),
                &packet[..packet.len().min(8)]
            ),
            None => "an empty packet".to_owned(),
        };
    };

    let (source, destination) = (ip.src_addr(), ip.dst_addr());
    let payload = ip.payload();

    // A fragment past the first carries no transport header to report.
    if ip.frag_offset() > 0 {
        return format!(
            "{:?} {source} > {destination} fragment at {} ({} bytes){}",
            ip.next_header(),
            ip.frag_offset(),
            payload.len(),
            if ip.more_frags() { ", more follow" } else { "" }
        );
    }

    let more = if ip.more_frags() { ", more follow" } else { "" };
    match ip.next_header() {
        IpProtocol::Tcp => match TcpPacket::new_checked(payload) {
            Ok(tcp) => format!(
                "TCP {source}:{} > {destination}:{} [{}] {} bytes{more}",
                tcp.src_port(),
                tcp.dst_port(),
                tcp_flags(&tcp),
                tcp.payload().len(),
            ),
            Err(err) => format!("TCP {source} > {destination}, malformed: {err}"),
        },
        IpProtocol::Udp => match UdpPacket::new_checked(payload) {
            Ok(udp) => format!(
                "UDP {source}:{} > {destination}:{} {} bytes{more}",
                udp.src_port(),
                udp.dst_port(),
                udp.payload().len(),
            ),
            Err(err) => format!("UDP {source} > {destination}, malformed: {err}"),
        },
        protocol => format!(
            "{protocol:?} {source} > {destination} {} bytes{more}",
            payload.len()
        ),
    }
}

fn tcp_flags(tcp: &TcpPacket<&[u8]>) -> String {
    let flags: String = [
        (tcp.syn(), 'S'),
        (tcp.ack(), 'A'),
        (tcp.fin(), 'F'),
        (tcp.rst(), 'R'),
        (tcp.psh(), 'P'),
    ]
    .into_iter()
    .filter(|(set, _)| *set)
    .map(|(_, name)| name)
    .collect();

    if flags.is_empty() {
        ".".to_owned()
    } else {
        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::{
        phy::ChecksumCapabilities,
        wire::{IPV4_HEADER_LEN, Ipv4Repr, TcpRepr, TcpSeqNumber, UdpRepr},
    };
    use std::net::Ipv4Addr;

    const CLIENT: Ipv4Addr = Ipv4Addr::new(192, 168, 91, 84);
    const SERVER: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);

    fn ipv4(next_header: IpProtocol, payload_len: usize, emit: impl FnOnce(&mut [u8])) -> Vec<u8> {
        let mut packet = vec![0u8; IPV4_HEADER_LEN + payload_len];
        let mut view = Ipv4Packet::new_unchecked(&mut packet[..]);
        Ipv4Repr {
            src_addr: CLIENT,
            dst_addr: SERVER,
            next_header,
            payload_len,
            hop_limit: 64,
        }
        .emit(&mut view, &ChecksumCapabilities::default());
        emit(view.payload_mut());
        packet
    }

    #[test]
    fn a_tcp_syn_is_described_with_its_flags() {
        let repr = TcpRepr {
            src_port: 51234,
            dst_port: 443,
            control: smoltcp::wire::TcpControl::Syn,
            seq_number: TcpSeqNumber(0),
            ack_number: None,
            window_len: 65535,
            window_scale: None,
            max_seg_size: None,
            sack_permitted: false,
            sack_ranges: [None; 3],
            timestamp: None,
            payload: &[],
        };
        let packet = ipv4(IpProtocol::Tcp, repr.buffer_len(), |payload| {
            repr.emit(
                &mut TcpPacket::new_unchecked(payload),
                &CLIENT.into(),
                &SERVER.into(),
                &ChecksumCapabilities::default(),
            );
        });

        assert_eq!(
            describe(&packet),
            "TCP 192.168.91.84:51234 > 1.1.1.1:443 [S] 0 bytes"
        );
    }

    #[test]
    fn a_udp_datagram_is_described_with_its_ports() {
        let repr = UdpRepr {
            src_port: 53821,
            dst_port: 53,
        };
        let packet = ipv4(IpProtocol::Udp, repr.header_len() + 4, |payload| {
            repr.emit(
                &mut UdpPacket::new_unchecked(payload),
                &CLIENT.into(),
                &SERVER.into(),
                4,
                |buffer| buffer.copy_from_slice(b"ping"),
                &ChecksumCapabilities::default(),
            );
        });

        assert_eq!(
            describe(&packet),
            "UDP 192.168.91.84:53821 > 1.1.1.1:53 4 bytes"
        );
    }

    /// The point of describing packets we reject is to tell "nothing arrives" apart from "the
    /// wrong thing arrives", so these must not all collapse to one message.
    #[test]
    fn packets_we_cannot_route_say_why() {
        let mut ipv6 = vec![0u8; 40];
        ipv6[0] = 6 << 4;
        assert_eq!(describe(&ipv6), "IPv6 (40 bytes), which we do not route");

        assert_eq!(describe(&[]), "an empty packet");

        // A stray address family header would look like this, and is worth recognising on sight.
        let prefixed = [0, 0, 0, 2, 0x45, 0, 0, 20];
        assert_eq!(
            describe(&prefixed),
            "unparseable IPv0 (8 bytes), starting [00, 00, 00, 02, 45, 00, 00, 14]"
        );
    }
}
