//! iOS TUN device implementing gotatun's `IpSend` and `IpRecv` traits.
//!
//! Reads and writes IP packets directly via the TUN file descriptor provided
//! by the iOS packet tunnel extension.
//!
//! On Darwin, the utun device prepends a 4-byte protocol family header to each
//! packet. We strip it on read and prepend it on write.

use bytes::Buf;
use gotatun::{
    packet::{
        Decoder, Ip, Ipv4Decoder, Ipv4Header, Ipv4PayloadDecoder, Ipv6Decoder, Ipv6PayloadDecoder,
        Packet, PacketBufPool, TcpDecoder,
    },
    tun::{IpRecv, IpSend, MtuWatcher},
};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use std::{
    collections::BTreeSet,
    io::{self, IoSlice},
    iter,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use talpid_netstack::{
    ip_mux::{IpMuxRecv, IpMuxSend},
    smoltcp_network::{SmoltcpIpRecv, SmoltcpIpSend},
};
use tokio::io::{Interest, unix::AsyncFd};
use zerocopy::IntoBytes;

/// Type aliases for the muxed IP pair used by GotaTun devices on iOS.
pub type IosTunIpSend = IpMuxSend<IosTunDevice, SmoltcpIpSend>;
pub type IosTunIpRecv = IpMuxRecv<IosTunDevice, SmoltcpIpRecv>;

/// 4-byte utun header: protocol family as uint32 in host byte order.
const UTUN_HEADER_LEN: usize = size_of::<u32>();

/// The original fd (owned by iOS) is never modified or closed.
/// We `dup()` it and own the copy as an [`OwnedFd`], which is closed when the
/// last clone of the wrapping `Arc<AsyncFd<_>>` is dropped.
pub struct IosTunDevice {
    async_fd: Arc<AsyncFd<OwnedFd>>,
    mtu: MtuWatcher,
    /// Packets read from the TUN device: user traffic entering the tunnel, excluding smoltcp's own.
    tx_packets: Arc<AtomicU64>,
    blocked_addrs: BTreeSet<SocketAddr>,
}

impl IosTunDevice {
    pub const DEFAULT_BLOCKED: &[SocketAddr] = &[
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from_octets([10, 64, 0, 1])), 1337),
        SocketAddr::new(
            IpAddr::V6(Ipv6Addr::from_segments([
                0xfc00, 0xbbbb, 0xbbbb, 0xbb01, 0, 0, 0, 1,
            ])),
            1337,
        ),
    ];

    /// Create a new TUN device from a raw file descriptor and fixed MTU.
    ///
    /// The fd is `dup()`d so we have our own copy (matching WireGuard-Go behavior).
    pub fn new(fd: RawFd, mtu: u16, blocked_addrs: &[SocketAddr]) -> io::Result<Self> {
        // The fd is owned by iOS; we only borrow it to make our own `dup`'d copy.
        // SAFETY: `fd` is a live TUN descriptor handed to us by the iOS packet
        // tunnel extension and stays open for the duration of this call. The
        // resulting `BorrowedFd` is non-owning and used solely for the `dup` below.
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let owned_fd = nix::unistd::dup(borrowed_fd)?;

        // Put our copy in non-blocking mode so tokio's `AsyncFd` drives readiness.
        let flags = OFlag::from_bits_retain(fcntl(&owned_fd, FcntlArg::F_GETFL)?);
        fcntl(&owned_fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;

        log::trace!(
            "IosTunDevice: dup({fd}) = {}, registering with tokio (mtu={mtu})",
            owned_fd.as_raw_fd(),
        );

        // SAFETY:
        // - The underlying socket was successfully created by `dup`, and therefore is valid.
        // - IosTunDevice is cleaned up if iOS closes the underlying fd. AsyncFd is therefore valid
        // for the lifetime of the fd.
        // - Since the dup'd fd' is owned by AyncFd, no one else will close it during the lifetime of
        // AsyncFd. Every call to inner.as_raw_fd will return the same file descriptor.
        let async_fd = unsafe { AsyncFd::register(owned_fd) }?;

        Ok(Self {
            async_fd: Arc::new(async_fd),
            mtu: MtuWatcher::new(mtu),
            tx_packets: Arc::default(),
            blocked_addrs: blocked_addrs.iter().copied().collect(),
        })
    }

    pub fn tx_packets(&self) -> Arc<AtomicU64> {
        self.tx_packets.clone()
    }

    fn should_block_packet(&self, packet: &Ip) -> bool {
        let ipv6 = Ipv6Decoder::UNCHECKED
            .decode_ref(packet)
            .and_then(|rest| Ipv6PayloadDecoder::<TcpDecoder>::UNCHECKED.decode_ref(rest))
            .map(|tcp| {
                self.blocked_addrs.contains(&SocketAddr::new(
                    tcp.header.source().into(),
                    tcp.payload.header.source_port.into(),
                ))
            });

        let ipv4 = Ipv4Decoder::UNCHECKED
            .decode_ref(packet)
            .and_then(|rest| Ipv4PayloadDecoder::<TcpDecoder>::UNCHECKED.decode_ref(rest))
            .map(|tcp| {
                self.blocked_addrs.contains(&SocketAddr::new(
                    tcp.header.source().into(),
                    tcp.payload.header.source_port.into(),
                ))
            });

        // due to us using unchecked decoding here, sometimes packages get wrongly
        // parsed as the wrong ip, so we check both
        ipv6.unwrap_or_default() || ipv4.unwrap_or_default()
    }
}

impl Clone for IosTunDevice {
    fn clone(&self) -> Self {
        Self {
            async_fd: self.async_fd.clone(),
            mtu: self.mtu.clone(),
            tx_packets: self.tx_packets.clone(),
            blocked_addrs: self.blocked_addrs.clone(),
        }
    }
}

impl IpSend for IosTunDevice {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        if self.should_block_packet(&packet) {
            return Ok(());
        }

        let utun_header = match packet.header.version() {
            4 => libc::AF_INET.to_ne_bytes(),
            6 => libc::AF_INET6.to_ne_bytes(),
            _ => return Err(io::ErrorKind::InvalidInput.into()),
        };

        // Prepend the 4-byte utun header (address family).
        let iov = [&utun_header, packet.as_bytes()].map(IoSlice::new);

        let n = self
            .async_fd
            .async_io(Interest::WRITABLE, |tun_fd| {
                nix::sys::uio::writev(tun_fd, &iov).map_err(Into::into)
            })
            .await?;

        let len = UTUN_HEADER_LEN + packet.as_bytes().len();
        debug_assert_eq!(n, len, "the entire packet must be written");

        Ok(())
    }
}

impl IpRecv for IosTunDevice {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let mut buf = pool.get();

        debug_assert!(buf.len() >= usize::from(self.mtu.get()));

        let n = self
            .async_fd
            .async_io(Interest::READABLE, |tun_fd| {
                nix::unistd::read(tun_fd, &mut buf[..]).map_err(Into::into)
            })
            .await?;

        if n == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        const MIN_LEN: usize = Ipv4Header::LEN + UTUN_HEADER_LEN;
        if n < MIN_LEN {
            return Err(io::Error::other("TUN read: Too few bytes"));
        }

        if n == buf.len() {
            log::warn!("Buffer capacify reached ({n}). Excess bytes may have been dropped.");
        }

        // Truncate buffer and strip the 4-byte utun header
        buf.buf_mut().truncate(n);
        buf.buf_mut().advance(UTUN_HEADER_LEN);

        match buf.try_into_ip() {
            Ok(packet) => {
                self.tx_packets.fetch_add(1, Ordering::Relaxed);
                Ok(iter::once(packet))
            }
            Err(e) => Err(io::Error::other(e.to_string())),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::tunnel_adapter::tun_device::IosTunDevice;
    use gotatun::{
        packet::{
            Ip, IpNextProtocol, Ipv4, Ipv4Header, Ipv6, Ipv6Header, Ipv6VersionTrafficFlow, Packet,
            PacketBufPool, Tcp, TcpFlags, TcpHeader, Udp, UdpHeader,
        },
        tun::{IpRecv, IpSend},
    };
    use nix::sys::socket::{AddressFamily, SockFlag, SockType, socketpair};
    use std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
        os::fd::AsRawFd,
        sync::Arc,
        time::Duration,
    };
    use tokio::{sync::Notify, time::timeout};
    use zerocopy::IntoBytes;

    fn create_tcp_packet(destination: SocketAddr) -> Packet<[u8]> {
        let tcp = TcpHeader {
            source_port: destination.port().into(),
            destination_port: 0.into(),
            seq_num: 10.into(),
            ack_num: 2.into(),
            data_offset: 0.into(),
            flags: TcpFlags::new(),
            window: 0.into(),
            checksum: 0.into(),
            urgent_pointer: 0.into(),
        };
        let payload = Tcp {
            header: tcp,
            options_and_payload: [0u8; 0],
        };

        match destination.ip() {
            IpAddr::V4(ipv4_addr) => {
                let header = Ipv4Header::new(
                    ipv4_addr,
                    Ipv4Addr::LOCALHOST,
                    IpNextProtocol::Tcp,
                    tcp.as_bytes(),
                );
                let packet = Ipv4 { header, payload };
                Packet::<[u8]>::copy_from(packet.as_bytes())
            }
            IpAddr::V6(ipv6_addr) => {
                let header = Ipv6Header {
                    version_traffic_flow: Ipv6VersionTrafficFlow::new().with_version(6),
                    payload_length: (size_of::<Ipv6Header>() as u16 + tcp.as_bytes().len() as u16)
                        .into(),
                    next_header: IpNextProtocol::Tcp,
                    hop_limit: 10,
                    source_address: ipv6_addr.to_bits().into(),
                    destination_address: Ipv6Addr::LOCALHOST.to_bits().into(),
                };
                let packet = Ipv6 { header, payload };
                Packet::<[u8]>::copy_from(packet.as_bytes())
            }
        }
    }

    fn create_udp_packet(destination: SocketAddr) -> Packet<[u8]> {
        let udp = UdpHeader {
            source_port: destination.port().into(),
            destination_port: 0.into(),
            length: 0.into(),
            checksum: 0.into(),
        };
        let payload = Udp {
            header: udp,
            payload: [0u8; 0],
        };
        match destination.ip() {
            IpAddr::V4(ipv4_addr) => {
                let header = Ipv4Header::new(
                    ipv4_addr,
                    Ipv4Addr::LOCALHOST,
                    IpNextProtocol::Udp,
                    udp.as_bytes(),
                );
                let packet = Ipv4 { header, payload };

                Packet::<[u8]>::copy_from(packet.as_bytes())
            }
            IpAddr::V6(ipv6_addr) => {
                let header = Ipv6Header {
                    version_traffic_flow: Ipv6VersionTrafficFlow::new().with_version(6),
                    payload_length: (size_of::<Ipv6Header>() as u16 + udp.as_bytes().len() as u16)
                        .into(),
                    next_header: IpNextProtocol::Tcp,
                    hop_limit: 10,
                    source_address: ipv6_addr.to_bits().into(),
                    destination_address: Ipv6Addr::LOCALHOST.to_bits().into(),
                };
                let packet = Ipv6 { header, payload };
                Packet::<[u8]>::copy_from(packet.as_bytes())
            }
        }
    }

    fn create_packet(pck: PacketTest) -> Packet<Ip> {
        let destination = pck
            .destination
            .parse::<SocketAddr>()
            .expect("invalid socket address passed to test");
        let bytes = match pck.protocol {
            Proto::Tcp => create_tcp_packet(destination),
            Proto::Udp => create_udp_packet(destination),
        };
        bytes.try_into_ip().unwrap()
    }

    #[derive(Debug, Clone, Copy)]
    enum Proto {
        Tcp,
        Udp,
    }
    #[derive(Clone, Copy)]
    struct PacketTest {
        should_arrive: bool,
        protocol: Proto,
        destination: &'static str,
    }
    fn pt(should_arrive: bool, protocol: Proto, destination: &'static str) -> PacketTest {
        PacketTest {
            should_arrive,
            protocol,
            destination,
        }
    }

    #[test]
    fn ip_blocking() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let (ours, theirs) = socketpair(
                    AddressFamily::Unix,
                    SockType::Datagram,
                    None,
                    SockFlag::empty(),
                )
                .unwrap();

                let mut sender =
                    IosTunDevice::new(theirs.as_raw_fd(), 1500, IosTunDevice::DEFAULT_BLOCKED)
                        .unwrap();

                // we keep the sender and reciever in sync so that we
                // can check that blocked packages are not recieved
                let notify = Arc::new(Notify::new());

                let packets = vec![
                    // IPv4 TCP
                    pt(false, Proto::Tcp, "10.64.0.1:1337"),
                    pt(true, Proto::Tcp, "10.64.0.1:553"),
                    pt(true, Proto::Tcp, "23.64.3.2:1337"),
                    pt(true, Proto::Tcp, "41.64.3.2:1337"),
                    pt(true, Proto::Tcp, "10.64.0.1:53"),
                    // IPv4 UDP, these should not be blocked
                    pt(true, Proto::Udp, "10.64.0.1:1337"),
                    pt(true, Proto::Udp, "10.64.0.1:553"),
                    pt(true, Proto::Udp, "23.64.3.2:1337"),
                    pt(true, Proto::Udp, "41.64.3.2:1337"),
                    pt(true, Proto::Udp, "10.64.0.1:53"),
                    // IPv6 TCP
                    pt(false, Proto::Tcp, "[fc00:bbbb:bbbb:bb01::1]:1337"),
                    pt(true, Proto::Tcp, "[fc00:bbbb:bbbb:bb01::1]:553"),
                    pt(true, Proto::Tcp, "[fc00:bbbb:bbbb:bb01::2]:1337"),
                    pt(true, Proto::Tcp, "[fc00:bbbb:bbbb:bb01:2312::1]:1337"),
                    pt(true, Proto::Tcp, "[fc00:bbbb:bbbb:bb01::1]:53"),
                    // IPv6 UDP
                    pt(true, Proto::Udp, "[fc00:bbbb:bbbb:bb01::1]:1337"),
                    pt(true, Proto::Udp, "[fc00:bbbb:bbbb:bb01::1]:553"),
                    pt(true, Proto::Udp, "[fc00:bbbb:bbbb:bb01::2]:1337"),
                    pt(true, Proto::Udp, "[fc00:bbbb:bbbb:bb01:2312::1]:1337"),
                    pt(true, Proto::Udp, "[fc00:bbbb:bbbb:bb01::1]:53"),
                ];

                let recv_packets = packets.clone();
                let send_packets = packets.into_iter().map(create_packet).collect::<Vec<_>>();

                let send_task = tokio::spawn({
                    let notify = notify.clone();
                    async move {
                        for packet in send_packets {
                            notify.notified().await;
                            sender.send(packet).await.unwrap();
                        }
                    }
                });

                let mut recv =
                    IosTunDevice::new(ours.as_raw_fd(), 1500, IosTunDevice::DEFAULT_BLOCKED)
                        .unwrap();
                let mut pool = PacketBufPool::new(1);
                for pck in recv_packets {
                    notify.notify_one();

                    let to = timeout(Duration::from_millis(100), recv.recv(&mut pool)).await;

                    if pck.should_arrive {
                        let result = to.unwrap();
                        let mut packet_iter = result.expect("failed getting expected package");
                        assert!(
                            packet_iter.next().is_some(),
                            "did not get {:?} destined for {:?}",
                            pck.protocol,
                            pck.destination
                        )
                    } else {
                        let Ok(result) = to else {
                            // timed out here, and that is okay
                            continue;
                        };
                        let Ok(mut packet_iter) = result else {
                            // we failed to get a packet, and that is okay
                            continue;
                        };
                        // if we get this far the iterator should be empty
                        assert!(
                            packet_iter.next().is_none(),
                            "accidentally got {:?} destined for {:?}",
                            pck.protocol,
                            pck.destination
                        )
                    }
                }
                send_task.await.unwrap();
            })
    }
}
