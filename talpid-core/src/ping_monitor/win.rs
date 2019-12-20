use pnet_packet::{
    icmp::{
        self,
        echo_request::{EchoRequestPacket, MutableEchoRequestPacket},
        IcmpCode, IcmpPacket, IcmpType,
    },
    Packet,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

const SEND_RETRY_ATTEMPTS: u32 = 10;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to open raw socket
    #[error(display = "Failed to open raw socket")]
    OpenError(#[error(source)] io::Error),

    /// Failed to read from raw socket
    #[error(display = "Failed to read from socket")]
    ReadError(#[error(source)] io::Error),

    /// Failed to write to raw socket
    #[error(display = "Failed to write to socket")]
    WriteError(#[error(source)] io::Error),

    #[error(display = "Timed out")]
    TimeoutError,
}

type Result<T> = std::result::Result<T, Error>;

pub struct Pinger {
    sock: Socket,
    addr: Ipv4Addr,
    id: u16,
    seq: u16,
}

const NUM_PINGS_TO_SEND: usize = 3;

impl Pinger {
    pub fn new(addr: Ipv4Addr, _interface_name: String) -> Result<Self> {
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

    pub fn send_icmp(&mut self) -> Result<()> {
        let dest = SocketAddr::new(IpAddr::from(self.addr), 0);
        for _ in 0..NUM_PINGS_TO_SEND {
            let request = self.next_ping_request();
            self.send_ping_request(&request, dest)?;
        }
        Ok(())
    }


    fn send_ping_request(
        &mut self,
        request: &EchoRequestPacket<'static>,
        destination: SocketAddr,
    ) -> Result<()> {
        let mut tries = 0;
        let mut result = Ok(());
        while tries < SEND_RETRY_ATTEMPTS {
            match self.sock.send_to(request.packet(), &destination.into()) {
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

    /// returns the next ping packet
    fn next_ping_request(&mut self) -> EchoRequestPacket<'static> {
        use rand::Rng;
        const ICMP_HEADER_LENGTH: usize = 8;
        const ICMP_PAYLOAD_LENGTH: usize = 150;
        const ICMP_PACKET_LENGTH: usize = ICMP_HEADER_LENGTH + ICMP_PAYLOAD_LENGTH;
        let mut payload = [0u8; ICMP_PAYLOAD_LENGTH];
        rand::thread_rng().fill(&mut payload[..]);
        let mut packet = MutableEchoRequestPacket::owned(vec![0u8; ICMP_PACKET_LENGTH])
            .expect("Failed to construct an empty packet");
        packet.set_icmp_type(IcmpType::new(8));
        packet.set_icmp_code(IcmpCode::new(0));
        packet.set_sequence_number(self.next_seq());
        packet.set_identifier(self.id);
        packet.set_payload(&payload);
        packet.set_checksum(icmp::checksum(&IcmpPacket::new(&packet.packet()).unwrap()));
        packet.consume_to_immutable()
    }

    fn next_seq(&mut self) -> u16 {
        let seq = self.seq;
        self.seq += 1;
        seq
    }
}
