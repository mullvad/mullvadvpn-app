//! ICMP pinger backed by a smoltcp socket.
//!
//! Used on iOS where the tunnel process cannot bind raw ICMP sockets directly.
//! ICMP echo requests are constructed and sent through the smoltcp stack, which
//! produces IP packets that flow through IpMux into GotaTun for encryption.

use crate::gotatun::smoltcp_network::SmoltcpIcmpSocket;
use rand::Rng;
use smoltcp::{
    phy::ChecksumCapabilities,
    wire::{Icmpv4Packet, Icmpv4Repr},
};
use std::{io, net::Ipv4Addr};

/// Random payload carried in each echo request, matching common `ping` implementations.
const PAYLOAD_LEN: usize = 42;
const PACKET_LEN: usize = 8 + PAYLOAD_LEN; // ICMPv4 echo header + payload

pub struct SmoltcpPinger {
    socket: SmoltcpIcmpSocket,
    dest: Ipv4Addr,
    id: u16,
    seq: u16,
}

impl SmoltcpPinger {
    /// `id` must be the identifier the underlying ICMP socket is bound to,
    /// or echo replies will not be delivered to it.
    pub fn new(socket: SmoltcpIcmpSocket, dest: Ipv4Addr, id: u16) -> Self {
        Self {
            socket,
            dest,
            id,
            seq: 0,
        }
    }

    pub async fn send_icmp(&mut self) -> Result<(), io::Error> {
        let mut data = [0u8; PAYLOAD_LEN];
        rand::rng().fill(&mut data);

        let repr = Icmpv4Repr::EchoRequest {
            ident: self.id,
            seq_no: self.seq,
            data: &data,
        };
        self.seq = self.seq.wrapping_add(1);

        let mut buffer = [0u8; PACKET_LEN];
        let mut packet = Icmpv4Packet::new_unchecked(&mut buffer[..]);
        repr.emit(&mut packet, &ChecksumCapabilities::default());

        self.socket.send_to_v4(&buffer, self.dest).await
    }
}
