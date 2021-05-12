use byteorder::{NetworkEndian, WriteBytesExt};
use rand::Rng;
use socket2::{Domain, Protocol, Socket, Type};
#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::{
    io::{self, Write},
    net::{Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

const SEND_RETRY_ATTEMPTS: u32 = 10;

/// Pinger errors
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to open raw socket
    #[error(display = "Failed to open ICMP socket")]
    OpenError(#[error(source)] io::Error),

    /// Failed to read from raw socket
    #[error(display = "Failed to read ICMP socket")]
    ReadError(#[error(source)] io::Error),

    /// Failed to set socket options
    #[error(display = "Failed to set socket options")]
    SocketOptError(#[error(source)] io::Error),

    /// Failed to write to raw socket
    #[error(display = "Failed to write to socket")]
    WriteError(#[error(source)] io::Error),

    /// ICMP buffer too small
    #[error(display = "ICMP message buffer too small")]
    BufferTooSmall,

    /// Interface name contains null bytes
    #[error(display = "Interface name contains a null byte")]
    InterfaceNameContainsNull,
}

type Result<T> = std::result::Result<T, Error>;

pub struct Pinger {
    sock: Socket,
    addr: SocketAddr,
    id: u16,
    seq: u16,
}

impl Pinger {
    #[cfg(target_os = "windows")]
    pub fn new(addr: Ipv4Addr, _interface_name: String) -> Result<Self> {
        let addr = SocketAddr::new(addr.into(), 0);
        let sock = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))
            .map_err(Error::OpenError)?;
        sock.set_nonblocking(true).map_err(Error::OpenError)?;

        Ok(Self {
            sock,
            id: rand::random(),
            addr,
            seq: 0,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn new(addr: Ipv4Addr, interface_name: String) -> Result<Self> {
        let addr = SocketAddr::new(addr.into(), 0);
        let sock = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))
            .map_err(Error::OpenError)?;
        sock.set_nonblocking(true).map_err(Error::OpenError)?;

        let cname = CString::new(interface_name.as_bytes().to_vec())
            .map_err(|_| Error::InterfaceNameContainsNull)?;
        sock.bind_device(Some(&cname))
            .map_err(Error::SocketOptError)?;

        Ok(Self {
            addr,
            sock,
            seq: 0,
            id: rand::random(),
        })
    }

    fn send_ping_request(&mut self, message: &[u8], destination: SocketAddr) -> Result<()> {
        let mut tries = 0;
        let mut result = Ok(());
        while tries < SEND_RETRY_ATTEMPTS {
            match self.sock.send_to(message, &destination.into()) {
                Ok(_) => {
                    return Ok(());
                }
                Err(err) => {
                    if Some(10065) != err.raw_os_error() {
                        return Err(Error::WriteError(err));
                    }
                    result = Err(Error::WriteError(err));
                }
            }
            thread::sleep(Duration::from_secs(1));
            tries += 1;
        }
        result
    }

    fn construct_icmpv4_packet(&mut self, buffer: &mut [u8]) -> Result<()> {
        if !construct_icmpv4_packet_inner(buffer, self) {
            return Err(Error::BufferTooSmall);
        }
        Ok(())
    }
}

impl super::Pinger for Pinger {
    fn send_icmp(&mut self) -> Result<()> {
        let mut message = [0u8; 50];
        self.construct_icmpv4_packet(&mut message)?;
        self.send_ping_request(&message, self.addr)
    }
}

trait PayloadWriter {
    fn packet_id(&mut self) -> u16;
    fn sequence_num(&mut self) -> u16;
    fn write_payload(&mut self, buffer: &mut [u8]);
}

impl PayloadWriter for Pinger {
    fn packet_id(&mut self) -> u16 {
        self.id
    }

    fn sequence_num(&mut self) -> u16 {
        let seq = self.seq;
        self.seq += 1;
        seq
    }

    fn write_payload(&mut self, buffer: &mut [u8]) {
        rand::thread_rng().fill(buffer);
    }
}

fn construct_icmpv4_packet_inner(
    buffer: &mut [u8],
    packet_writer: &mut impl PayloadWriter,
) -> bool {
    const ICMP_CHECKSUM_OFFSET: usize = 2;
    if buffer.len() < 14 {
        return false;
    }

    let mut writer = &mut buffer[..];
    // ICMP type - Echo (ping) request
    writer.write_u8(0x08).unwrap();
    // Code - 0
    writer.write_u8(0x00).unwrap();
    // Checksum -filled in later
    writer.write_u16::<NetworkEndian>(0x000).unwrap();
    // packet ID
    writer
        .write_u16::<NetworkEndian>(packet_writer.packet_id())
        .unwrap();
    // packet sequence number
    writer
        .write_u16::<NetworkEndian>(packet_writer.sequence_num())
        .unwrap();
    // payload
    packet_writer.write_payload(writer);

    let checksum = internet_checksum::checksum(buffer);
    (&mut buffer[ICMP_CHECKSUM_OFFSET..])
        .write(&checksum)
        .unwrap();

    true
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestPayload {}

    impl PayloadWriter for TestPayload {
        fn packet_id(&mut self) -> u16 {
            0x1dcd
        }

        fn sequence_num(&mut self) -> u16 {
            0x0001
        }

        fn write_payload(&mut self, mut buffer: &mut [u8]) {
            let _ = buffer.write(&[
                0xb6, 0xe0, 0x87, 0x60, 0x00, 0x00, 0x00, 0x00, 0x97, 0xad, 0x09, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
                0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
                0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            ]);
        }
    }

    #[test]
    fn test_icmpv4_packet() {
        // captured from a plain `ping -4 127.1`
        let expected_packet = [
            // ICMP type - echo request
            0x08, // Code 0
            0x00, // checksum
            0x3c, 0x70, // packet ID
            0x1d, 0xcd, // sequence number
            0x00, 0x01, // payload
            0xb6, 0xe0, 0x87, 0x60, 0x00, 0x00, 0x00, 0x00, 0x97, 0xad, 0x09, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
        ];

        let mut buffer = [0u8; 64];
        assert!(construct_icmpv4_packet_inner(
            &mut buffer[..],
            &mut TestPayload {}
        ));
        assert_eq!(buffer, expected_packet);
    }

    #[test]
    fn test_icmpv4_packet_too_short() {
        assert!(!construct_icmpv4_packet_inner(
            &mut [0u8; 13],
            &mut TestPayload {}
        ));
    }
}
