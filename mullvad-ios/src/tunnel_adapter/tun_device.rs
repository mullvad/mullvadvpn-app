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
use std::{io, iter, os::fd::RawFd};
use tokio::io::unix::AsyncFd;

/// Type aliases for the muxed IP pair used by GotaTun devices on iOS.
pub type IosTunIpSend = IpMuxSend<IosTunDevice, SmoltcpIpSend>;
pub type IosTunIpRecv = IpMuxRecv<IosTunDevice, SmoltcpIpRecv>;

/// 4-byte utun header: protocol family as uint32 in host byte order.
const UTUN_HEADER_LEN: usize = 4;
const AF_INET: u32 = 2;
const AF_INET6: u32 = 30;

/// The original fd (owned by iOS) is never modified or closed.
/// We `dup()` it and own the copy, which we close on drop.
#[derive(Clone)]
pub struct IosTunDevice {
    fd: RawFd,
    async_fd: std::sync::Arc<AsyncFd<RawFd>>,
    _close_guard: std::sync::Arc<FdCloseGuard>,
    mtu: MtuWatcher,
}

struct FdCloseGuard(RawFd);

impl Drop for FdCloseGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` is the fd we obtained from `dup()` in `IosTunDevice::new`
        // and have owned exclusively ever since. `Drop` runs at most once, so the fd
        // is still open and is not closed anywhere else.
        unsafe { libc::close(self.0) };
        log::debug!("IosTunDevice: closed dup'd fd {}", self.0);
    }
}

impl IosTunDevice {
    /// Create a new TUN device from a raw file descriptor and fixed MTU.
    ///
    /// The fd is `dup()`d so we have our own copy (matching WireGuard-Go behavior).
    pub fn new(fd: RawFd, mtu: u16) -> io::Result<Self> {
        // SAFETY: `dup` has no preconditions on its argument — an invalid `fd` just
        // returns an error, which we handle. On success it returns a fresh fd we own.
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: `dup_fd` is a valid open fd we just obtained from `dup`; `F_GETFL`
        // reads the descriptor flags and takes no further argument.
        let flags = unsafe { libc::fcntl(dup_fd, libc::F_GETFL) };
        if flags < 0 {
            let err = io::Error::last_os_error();
            // SAFETY: `dup_fd` is the valid fd from `dup` above and is still open; we
            // close it exactly once on this error path before returning.
            unsafe { libc::close(dup_fd) };
            return Err(err);
        }
        // SAFETY: `dup_fd` is valid; `F_SETFL` consumes the `flags` int argument.
        let ret = unsafe { libc::fcntl(dup_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            // SAFETY: as above — `dup_fd` is still the only owner and still open.
            unsafe { libc::close(dup_fd) };
            return Err(err);
        }

        log::debug!("IosTunDevice: dup({fd}) = {dup_fd}, registering with tokio (mtu={mtu})");

        let async_fd = match AsyncFd::new(dup_fd) {
            Ok(fd) => fd,
            Err(e) => {
                // SAFETY: as above — `dup_fd` is still open and not yet handed to the
                // `FdCloseGuard`, so we close it exactly once here.
                unsafe { libc::close(dup_fd) };
                return Err(e);
            }
        };

        Ok(Self {
            fd: dup_fd,
            async_fd: std::sync::Arc::new(async_fd),
            _close_guard: std::sync::Arc::new(FdCloseGuard(dup_fd)),
            mtu: MtuWatcher::new(mtu),
        })
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

        // Prepend 4-byte utun header
        let mut buf = Vec::with_capacity(UTUN_HEADER_LEN + ip_bytes.len());
        buf.extend_from_slice(&af.to_ne_bytes());
        buf.extend_from_slice(ip_bytes);

        loop {
            let mut guard = self.async_fd.writable().await?;
            match guard.try_io(|_| {
                // SAFETY: `self.fd` stays open for the lifetime of `self` (closed only
                // when the last clone drops, via `FdCloseGuard`). `buf` is a live slice
                // of `buf.len()` bytes, so the pointer/length pair is valid to read.
                let ret = unsafe { libc::write(self.fd, buf.as_ptr().cast(), buf.len()) };
                if ret < 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(())
                }
            }) {
                Ok(result) => return result,
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
        let mut raw_buf = vec![0u8; UTUN_HEADER_LEN + self.mtu.clone().get() as usize];

        loop {
            let mut guard = self.async_fd.readable().await?;
            match guard.try_io(|_| {
                // SAFETY: `self.fd` stays open for the lifetime of `self` (see above).
                // `raw_buf` is a live, mutable allocation of `raw_buf.len()` bytes, so
                // the pointer/length pair is valid for `read` to write into.
                let ret =
                    unsafe { libc::read(self.fd, raw_buf.as_mut_ptr().cast(), raw_buf.len()) };
                if ret < 0 {
                    Err(io::Error::last_os_error())
                } else if ret == 0 {
                    Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "TUN read returned 0",
                    ))
                } else {
                    Ok(ret as usize)
                }
            }) {
                Ok(Ok(n)) => {
                    if n <= UTUN_HEADER_LEN {
                        continue;
                    }
                    // Strip the 4-byte utun header
                    let ip_data = &raw_buf[UTUN_HEADER_LEN..n];
                    let mut packet = pool.get();
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
