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
    io::{self, IoSlice},
    iter,
    os::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd},
    sync::Arc,
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
        })
    }
}

impl Clone for IosTunDevice {
    fn clone(&self) -> Self {
        Self {
            async_fd: self.async_fd.clone(),
            mtu: self.mtu.clone(),
        }
    }
}

impl IpSend for IosTunDevice {
    async fn send(&mut self, packet: Packet<Ip>) -> io::Result<()> {
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
            Ok(packet) => Ok(iter::once(packet)),
            Err(e) => Err(io::Error::other(e.to_string())),
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}
