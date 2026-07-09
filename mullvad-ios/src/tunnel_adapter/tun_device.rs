//! iOS TUN device implementing gotatun's `IpSend` and `IpRecv` traits.
//!
//! Reads and writes IP packets directly via the TUN file descriptor provided
//! by the iOS packet tunnel extension.
//!
//! On Darwin, the utun device prepends a 4-byte protocol family header to each
//! packet. We strip it on read and prepend it on write.

use crate::gotatun::{
    ip_mux::{IpMuxRecv, IpMuxSend},
    smoltcp_network::{SmoltcpIpRecv, SmoltcpIpSend},
};
use bytes::Buf;
use gotatun::{
    packet::{Ip, Ipv4Header, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use std::{
    io, iter,
    os::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd},
    sync::Arc,
};
use tokio::io::{Interest, unix::AsyncFd};

/// Type aliases for the muxed IP pair used by GotaTun devices on iOS.
pub type IosTunIpSend = IpMuxSend<IosTunDevice, SmoltcpIpSend>;
pub type IosTunIpRecv = IpMuxRecv<IosTunDevice, SmoltcpIpRecv>;

/// 4-byte utun header: protocol family as uint32 in host byte order.
const UTUN_HEADER_LEN: usize = 4;
const AF_INET: u32 = 2;
const AF_INET6: u32 = 30;

/// Size of the per-handle I/O scratch buffers - comfortably larger than the
/// utun header plus any IP packet we can be handed.
const IO_BUF_SIZE: usize = 65000;

fn io_buf() -> Box<[u8]> {
    vec![0u8; IO_BUF_SIZE].into_boxed_slice()
}

/// The original fd (owned by iOS) is never modified or closed.
/// We `dup()` it and own the copy as an [`OwnedFd`], which is closed when the
/// last clone of the wrapping `Arc<AsyncFd<_>>` is dropped.
pub struct IosTunDevice {
    async_fd: Arc<AsyncFd<OwnedFd>>,
    mtu: MtuWatcher,
    /// Reusable I/O buffer, so the send/recv hot paths don't allocate per
    /// packet.
    io_buf: Box<[u8]>,
}

impl IosTunDevice {
    /// Create a new TUN device from a raw file descriptor and fixed MTU.
    ///
    /// The fd is `dup()`d so we have our own copy (matching WireGuard-Go behavior).
    pub fn new(fd: RawFd, mtu: u16) -> io::Result<Self> {
        // The fd is owned by iOS; we only borrow it to make our own `dup`'d copy.
        // SAFETY: `fd` is a live TUN descriptor handed to us by the iOS packet
        // tunnel extension and stays open for the duration of this call. The
        // resulting `BorrowedFd` is non-owning and used solely for the `dup` below.
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let owned_fd = nix::unistd::dup(borrowed_fd)?;

        // Put our copy in non-blocking mode so tokio's `AsyncFd` drives readiness.
        let flags = OFlag::from_bits_retain(fcntl(&owned_fd, FcntlArg::F_GETFL)?);
        fcntl(&owned_fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;

        log::debug!(
            "IosTunDevice: dup({fd}) = {}, registering with tokio (mtu={mtu})",
            owned_fd.as_raw_fd(),
        );

        let async_fd = AsyncFd::new(owned_fd)?;

        Ok(Self {
            async_fd: Arc::new(async_fd),
            mtu: MtuWatcher::new(mtu),
            io_buf: io_buf(),
        })
    }
}

impl Clone for IosTunDevice {
    fn clone(&self) -> Self {
        Self {
            async_fd: self.async_fd.clone(),
            mtu: self.mtu.clone(),
            // Fresh scratch buffers; their contents are per-call anyway.
            io_buf: io_buf(),
        }
    }
}

impl IpSend for IosTunDevice {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
        let af: u32 = packet.header.version().into();
        let ip_bytes: &[u8] = &packet.into_bytes();

        // Prepend the 4-byte utun header in the scratch buffer.
        let len = UTUN_HEADER_LEN + ip_bytes.len();
        let Some(buf) = self.io_buf.get_mut(..len) else {
            return Err(io::Error::other("packet exceeds send buffer"));
        };
        buf[..UTUN_HEADER_LEN].copy_from_slice(&af.to_ne_bytes());
        buf[UTUN_HEADER_LEN..].copy_from_slice(ip_bytes);

        let n = self
            .async_fd
            .async_io(Interest::WRITABLE, |tun_fd| {
                nix::unistd::write(tun_fd, &self.io_buf[..len]).map_err(Into::into)
            })
            .await?;

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

        return match buf.try_into_ip() {
            Ok(packet) => Ok(iter::once(packet)),
            Err(e) => Err(io::Error::other(e.to_string())),
        };
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}
