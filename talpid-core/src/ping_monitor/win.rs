use pnet_packet::{
    icmp::{
        self,
        echo_reply::EchoReplyPacket,
        echo_request::{EchoRequestPacket, MutableEchoRequestPacket},
        IcmpCode, IcmpPacket, IcmpType,
    },
    Packet,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
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

pub fn monitor_ping(
    ip: Ipv4Addr,
    timeout_secs: u16,
    interface: &str,
    close_receiver: mpsc::Receiver<()>,
) -> Result<()> {
    let mut pinger = Pinger::new(ip, interface)?;
    while let Err(mpsc::TryRecvError::Empty) = close_receiver.try_recv() {
        let start = Instant::now();
        pinger.send_ping(Duration::from_secs(timeout_secs.into()))?;
        if let Some(remaining) =
            Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed())
        {
            thread::sleep(remaining);
        }
    }

    Ok(())
}

pub fn ping(ip: Ipv4Addr, timeout_secs: u16, interface: &str) -> Result<()> {
    Pinger::new(ip, interface)?.send_ping(Duration::from_secs(timeout_secs.into()))
}

type Result<T> = std::result::Result<T, Error>;

pub struct Pinger {
    sock: Socket,
    addr: Ipv4Addr,
    id: u16,
    seq: u16,
}

impl Pinger {
    pub fn new(addr: Ipv4Addr, _interface_name: &str) -> Result<Self> {
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

    /// Sends an ICMP echo request
    pub fn send_ping(&mut self, timeout: Duration) -> Result<()> {
        let dest = SocketAddr::new(IpAddr::from(self.addr), 0);
        let request = self.next_ping_request();
        self.send_ping_request(&request, dest.into())?;
        self.wait_for_response(Instant::now() + timeout, &request)
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
        const ICMP_HEADER_LENGTH: usize = 8;
        const ICMP_PAYLOAD_LENGTH: usize = 24;
        const ICMP_PACKET_LENGTH: usize = ICMP_HEADER_LENGTH + ICMP_PAYLOAD_LENGTH;
        let payload: [u8; ICMP_PAYLOAD_LENGTH] = rand::random();
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


    fn wait_for_response(&mut self, deadline: Instant, req: &EchoRequestPacket<'_>) -> Result<()> {
        let mut recv_buffer = [0u8; 4096];
        while Instant::now() < deadline {
            match self.sock.recv(&mut recv_buffer) {
                Ok(recv_len) => {
                    if recv_len > 20 {
                        // have to slice off first 20 bytes for the IP header.
                        if let Some(reply) = Self::parse_response(&recv_buffer[20..recv_len]) {
                            if reply.get_identifier() == req.get_identifier()
                                && reply.get_sequence_number() == req.get_sequence_number()
                                && req.payload() == reply.payload()
                            {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return Err(Error::ReadError(e));
                }
            }
        }
        Err(Error::TimeoutError)
    }

    fn parse_response<'a>(buffer: &'a [u8]) -> Option<EchoReplyPacket<'a>> {
        let icmp_checksum = icmp::checksum(&IcmpPacket::new(buffer)?);
        let reply = EchoReplyPacket::new(buffer)?;
        if reply.get_checksum() == icmp_checksum {
            Some(reply)
        } else {
            None
        }
    }
}
