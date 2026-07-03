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
use gotatun::{
    packet::{Ip, Packet, PacketBufPool},
    tun::{IpRecv, IpSend, MtuWatcher},
};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use std::{
    io, iter,
    os::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd},
    sync::Arc,
};
use tokio::io::unix::AsyncFd;

/// Type aliases for the muxed IP pair used by GotaTun devices on iOS.
pub type IosTunIpSend = IpMuxSend<IosTunDevice, SmoltcpIpSend>;
pub type IosTunIpRecv = IpMuxRecv<IosTunDevice, SmoltcpIpRecv>;

/// 4-byte utun header: protocol family as uint32 in host byte order.
const UTUN_HEADER_LEN: usize = 4;
const AF_INET: u32 = 2;
const AF_INET6: u32 = 30;

/// Size of the per-handle I/O scratch buffers — comfortably larger than the
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
        let ip_bytes: &[u8] = &packet.into_bytes();

        // Determine address family from IP version (first nibble)
        let af: u32 = if !ip_bytes.is_empty() && (ip_bytes[0] >> 4) == 6 {
            AF_INET6
        } else {
            AF_INET
        };

        // Prepend the 4-byte utun header in the scratch buffer.
        let len = UTUN_HEADER_LEN + ip_bytes.len();
        let Some(buf) = self.io_buf.get_mut(..len) else {
            return Err(io::Error::other("packet exceeds send buffer"));
        };
        buf[..UTUN_HEADER_LEN].copy_from_slice(&af.to_ne_bytes());
        buf[UTUN_HEADER_LEN..].copy_from_slice(ip_bytes);

        loop {
            let mut guard = self.async_fd.writable().await?;
            match guard.try_io(|inner| {
                nix::unistd::write(inner.get_ref(), &self.io_buf[..len]).map_err(Into::into)
            }) {
                Ok(result) => return result.map(drop),
                Err(_would_block) => continue,
            }
        }
    }
}

impl IpRecv for IosTunDevice {
    async fn recv<'a>(
        &'a mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<impl Iterator<Item = Packet<Ip>> + Send + 'a> {
        let raw_buf = &mut self.io_buf;

        loop {
            let mut guard = self.async_fd.readable().await?;
            match guard.try_io(|inner| {
                let n = nix::unistd::read(inner.get_ref(), raw_buf)?;
                if n == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "TUN read returned 0",
                    ));
                }
                Ok(n)
            }) {
                Ok(Ok(n)) => {
                    if n <= UTUN_HEADER_LEN {
                        continue;
                    }
                    // Strip the 4-byte utun header
                    let ip_data = &raw_buf[UTUN_HEADER_LEN..n];
                    let mut packet = pool.get();
                    if ip_data.len() > packet.len() {
                        log::warn!("dropping oversized TUN packet ({} bytes)", ip_data.len());
                        continue;
                    }
                    let ip_len = ip_data.len();
                    packet[..ip_len].copy_from_slice(ip_data);
                    packet.truncate(ip_len);

                    return match packet.try_into_ip() {
                        Ok(packet) => Ok(iter::once(packet)),
                        Err(e) => Err(io::Error::other(e.to_string())),
                    };
                }
                Ok(Err(e)) => return Err(e),
                Err(_would_block) => continue,
            }
        }
    }

    fn mtu(&self) -> MtuWatcher {
        self.mtu.clone()
    }
}
