//! Routes the traffic arriving on the tunnel device out through real OS sockets.
//!
//! One task reads the device and demultiplexes by protocol; one task writes everything back. The
//! read loop never waits on an upstream socket, so a stalled connection cannot hold up the rest.

use std::{io::{stdin, BufRead}, sync::Arc};

use arc_swap::ArcSwap;
use bytes::Bytes;
use smoltcp::wire::{IpProtocol, Ipv4Packet, TcpPacket, UdpPacket};
use tokio::sync::{Notify, mpsc};
use tun_rs::AsyncDevice;
use udp::UdpRouter;

use super::{RuleMap, TunnelDevices};

mod filter;
mod tcp;
mod udp;

/// Largest packet read off the tunnel device in one go.
const MAX_PACKET_SIZE: usize = u16::MAX as usize;
/// Packets buffered for the tunnel device writer.
const EGRESS_QUEUE_DEPTH: usize = 512;

pub struct Router {
}

impl Router {
    pub fn spawn(devices: TunnelDevices, rules: Arc<ArcSwap<RuleMap>>) {
        let TunnelDevices { sink, source } = devices;
        let sink = Arc::new(sink);
        let source = Arc::new(source);
        let sink_clone = sink.clone();
        let source_clone = source.clone();
        let rules_clone = rules.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];

            while let Ok(bytes_received) = sink.recv(&mut buf).await {
                let packet = &buf[..bytes_received];
                if filter::blocked(&rules.load(), packet) {
                    continue;
                }
                if let Err(err) = source_clone.send(packet).await {
                    log::error!("Failed to write to source utun: {err}");
                }
            }

        });

        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];
            while let Ok(bytes_received) = source.recv(&mut buf).await {
                let packet = &buf[..bytes_received];
                if filter::blocked(&rules_clone.load(), packet) {
                    continue;
                }
                if let Err(err) = sink_clone.send(packet).await {
                    log::error!("Failed to write to sink utun: {err}");
                }
            }

        });
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
