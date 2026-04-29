//! ICMP pinger backed by a smoltcp socket.
//!
//! Used on iOS where the tunnel process cannot bind raw ICMP sockets directly.
//! ICMP echo requests are constructed and sent through the smoltcp stack, which
//! produces IP packets that flow through IpMux into GotaTun for encryption.

use crate::gotatun::smoltcp_network::SmoltcpIcmpSocket;
use byteorder::{NetworkEndian, WriteBytesExt};
use rand::Rng;
use std::{io, io::Write, net::Ipv4Addr};

pub struct SmoltcpPinger {
    socket: SmoltcpIcmpSocket,
    dest: Ipv4Addr,
    id: u16,
    seq: u16,
}

impl SmoltcpPinger {
    pub fn new(socket: SmoltcpIcmpSocket, dest: Ipv4Addr) -> Self {
        Self {
            socket,
            dest,
            id: rand::random(),
            seq: 0,
        }
    }

    pub async fn send_icmp(&mut self) -> Result<(), io::Error> {
        let mut message = [0u8; 50];
        construct_icmpv4_packet(&mut message, self.id, self.seq)
            .map_err(|()| io::Error::other("ICMP buffer too small"))?;
        self.seq = self.seq.wrapping_add(1);
        self.socket.send_to_v4(&message, self.dest).await
    }
}

fn construct_icmpv4_packet(buffer: &mut [u8], id: u16, seq: u16) -> Result<(), ()> {
    const ICMP_CHECKSUM_OFFSET: usize = 2;
    if buffer.len() < 14 {
        return Err(());
    }

    let mut writer = &mut buffer[..];
    // ICMP type: Echo Request
    writer.write_u8(0x08).unwrap();
    // Code: 0
    writer.write_u8(0x00).unwrap();
    // Checksum placeholder
    writer.write_u16::<NetworkEndian>(0x0000).unwrap();
    // Identifier
    writer.write_u16::<NetworkEndian>(id).unwrap();
    // Sequence number
    writer.write_u16::<NetworkEndian>(seq).unwrap();
    // Random payload
    rand::rng().fill(writer);

    let checksum = internet_checksum::checksum(buffer);
    (&mut buffer[ICMP_CHECKSUM_OFFSET..])
        .write_all(&checksum)
        .unwrap();

    Ok(())
}
