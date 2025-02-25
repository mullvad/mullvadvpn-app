//! This module provides a thin wrapper for BPF devices on macOS. BPF is used for packet
//! filtering/capture and is exposed as several devices `/dev/bpfN` (where `N` is some integer).
//!
//! BPF devices can be attached to network interface and used for reading and writing packets
//! directly on them, usually whole frames.
//!
//! Certain features may be macOS-specific, but much of the documentation for FreeBSD still holds
//! true. Read more here: https://man.freebsd.org/cgi/man.cgi?bpf
use futures::ready;
use libc::{
    bpf_hdr, ifreq, BIOCGBLEN, BIOCGDLT, BIOCIMMEDIATE, BIOCSBLEN, BIOCSETIF, BIOCSHDRCMPLT,
    BIOCSSEESENT, BPF_ALIGNMENT, EBUSY, F_GETFL, F_SETFL, O_NONBLOCK,
};
use std::{
    ffi::{c_int, c_uint},
    fs::File,
    io::{self, Read, Write},
    mem,
    os::fd::AsRawFd,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{unix::AsyncFd, AsyncRead, Interest, ReadBuf};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to open BPF device
    #[error("Failed to open BPF device")]
    OpenBpfDevice(#[source] io::Error),
    /// Failed to duplicate BPF fd
    #[error("Failed to duplicate BPF device")]
    Duplicate(#[source] io::Error),
    /// No free BPF device found
    #[error("No free BPF device found")]
    NoFreeBpfDeviceFound,
    /// Interface name too long
    #[error("Interface name too long")]
    InterfaceNameTooLong,
    /// IOCTL failed
    #[error("IOCTL failed")]
    IoctlFailed(#[source] io::Error),
    /// Failed to get flags for BPF device
    #[error("Failed to get flags for BPF device")]
    GetFileFlags(#[source] io::Error),
    /// Failed to set flags for BPF device
    #[error("Failed to set flags for BPF device")]
    SetFileFlags(#[source] io::Error),
    /// Failed to create AsyncFd
    #[error("Failed to create AsyncFd")]
    AsyncFd(#[source] io::Error),
}

macro_rules! ioctl {
    ($fd:expr, $request:expr, $($arg:expr),+) => {
        if libc::ioctl($fd, $request, $($arg),+) >= 0 {
            Ok(())
        } else {
            Err(Error::IoctlFailed(io::Error::last_os_error()))
        }
    };
}

pub struct Bpf {
    file: File,
}

pub struct ReadHalf(File);

pub struct WriteHalf(File);

impl Bpf {
    pub fn open() -> Result<Self, Error> {
        Ok(Self {
            file: Self::open_device()?,
        })
    }

    pub fn split(self) -> Result<(ReadHalf, WriteHalf), Error> {
        let dup = self.file.try_clone().map_err(Error::Duplicate)?;
        Ok((ReadHalf(dup), WriteHalf(self.file)))
    }

    fn open_device() -> Result<File, Error> {
        const MAX_BPF_COUNT: usize = 1000;

        // Find a free bpf device
        for dev_num in 0..MAX_BPF_COUNT {
            // Open as O_RDWR
            match File::options()
                .read(true)
                .write(true)
                .open(format!("/dev/bpf{dev_num}"))
            {
                Ok(file) => {
                    log::trace!("Opened BPF device: /dev/bpf{dev_num}");
                    return Ok(file);
                }
                Err(_e) if _e.raw_os_error() == Some(EBUSY) => continue,
                Err(error) => return Err(Error::OpenBpfDevice(error)),
            }
        }
        Err(Error::NoFreeBpfDeviceFound)
    }

    pub fn set_nonblocking(&self, enabled: bool) -> Result<(), Error> {
        // SAFETY: The fd is valid for the lifetime of `self`
        let mut flags = unsafe { libc::fcntl(self.as_raw_fd(), F_GETFL) };
        if flags == -1 {
            return Err(Error::GetFileFlags(io::Error::last_os_error()));
        }
        if enabled {
            flags |= O_NONBLOCK;
        } else {
            flags &= !O_NONBLOCK;
        }

        // SAFETY: The fd is valid for the lifetime of `self`
        let result = unsafe { libc::fcntl(self.as_raw_fd(), F_SETFL, flags) };
        if result == -1 {
            return Err(Error::SetFileFlags(io::Error::last_os_error()));
        }
        Ok(())
    }

    /// Set BIOCSETIF
    pub fn set_interface(&self, name: &str) -> Result<(), Error> {
        // SAFETY: It is valid for this C struct to be zeroed. We fill in the details later
        let mut ifr: ifreq = unsafe { std::mem::zeroed() };

        let name_bytes = name.as_bytes();
        if name_bytes.len() >= std::mem::size_of_val(&ifr.ifr_name) {
            return Err(Error::InterfaceNameTooLong);
        }

        // SAFETY: `name_bytes` cannot exceed the size of `ifr_name`
        unsafe {
            std::ptr::copy_nonoverlapping(
                name_bytes.as_ptr(),
                &mut ifr.ifr_name as *mut _ as *mut _,
                name_bytes.len(),
            )
        };

        // SAFETY: The fd is valid for the lifetime of `self`, and `ifr` has a valid interface
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSETIF, &ifr) }
    }

    /// Enable or disable immediate mode (BIOCIMMEDIATE)
    pub fn set_immediate(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCIMMEDIATE, &enable) }
    }

    // See locally sent packets (BIOCSSEESENT)
    pub fn set_see_sent(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSSEESENT, &enable) }
    }

    /// Enable or disable locally sent messages (BIOCSHDRCMPLT)
    pub fn set_header_complete(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSHDRCMPLT, &enable) }
    }

    pub fn set_buffer_size(&self, mut buffer_size: c_uint) -> Result<usize, Error> {
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe {
            ioctl!(self.file.as_raw_fd(), BIOCSBLEN, &mut buffer_size)?;
        }
        Ok(buffer_size as usize)
    }

    pub fn required_buffer_size(&self) -> Result<usize, Error> {
        let mut buf_size = 0i32;
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe {
            ioctl!(self.file.as_raw_fd(), BIOCGBLEN, &mut buf_size)?;
        }
        Ok(buf_size as usize)
    }

    pub fn dlt(&self) -> Result<u32, Error> {
        let mut dlt = 0;
        // SAFETY: The fd is valid for the lifetime of `self`
        unsafe {
            ioctl!(self.file.as_raw_fd(), BIOCGDLT, &mut dlt)?;
        }
        Ok(dlt)
    }
}

impl Read for Bpf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Read for &Bpf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.file).read(buf)
    }
}

impl Write for Bpf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl Write for &Bpf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.file).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl Read for ReadHalf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for WriteHalf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl AsRawFd for Bpf {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.file.as_raw_fd()
    }
}

pub struct BpfStream {
    inner: AsyncFd<File>,
}

impl BpfStream {
    pub fn from_read_half(reader: ReadHalf) -> Result<Self, Error> {
        Self::from_file(reader.0)
    }

    fn from_file(file: File) -> Result<Self, Error> {
        Ok(BpfStream {
            inner: AsyncFd::with_interest(file, Interest::READABLE).map_err(Error::AsyncFd)?,
        })
    }
}

impl AsyncRead for BpfStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|inner| inner.get_ref().read(unfilled)) {
                Ok(Ok(len)) => {
                    buf.advance(len);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

/// Parse one or more BPF headers and payloads from an arbitrarily sized buffer
pub struct BpfIterMut<'a> {
    data: &'a mut [u8],
    current_packet_offset: usize,
}

impl<'a> BpfIterMut<'a> {
    /// Return a new iterator over BPF packets
    pub fn new(data: &'a mut [u8]) -> Self {
        Self {
            data,
            current_packet_offset: 0,
        }
    }

    /// Return the next BPF payload, or None
    pub fn next(&mut self) -> Option<&mut [u8]> {
        let offset = self.current_packet_offset;
        if self.data.len() <= offset || self.data.len() - offset < mem::size_of::<bpf_hdr>() {
            return None;
        }

        // SAFETY: The buffer is large enough to contain a BPF header
        let hdr = unsafe {
            &*(self.data[offset..offset + mem::size_of::<bpf_hdr>()].as_ptr() as *const bpf_hdr)
        };

        if offset + hdr.bh_hdrlen as usize + hdr.bh_caplen as usize > self.data.len() {
            return None;
        }

        // SAFETY: This is within the bounds of 'data'
        let payload = &mut self.data[offset + hdr.bh_hdrlen as usize
            ..offset + (hdr.bh_hdrlen as usize + hdr.bh_caplen as usize)];

        // Each packet starts on a word boundary after the previous header and capture
        self.current_packet_offset =
            offset + usize::try_from(bpf_wordalign(hdr.bh_hdrlen as u32 + hdr.bh_caplen)).unwrap();

        Some(payload)
    }
}

/// Compute the next word boundary given `n`. `n` will be rounded up to a multiple of
/// "word" (defined by `BPF_ALIGNMENT`). Assuming `BPF_ALIGNMENT == 4`:
///
/// ```text
/// n=0: bpf_wordalign(0) == 0
/// n=1: bpf_wordalign(1) == 4
/// n=2: bpf_wordalign(2) == 4
/// n=3: bpf_wordalign(3) == 4
/// n=4: bpf_wordalign(4) == 4
/// n=5: bpf_wordalign(5) == 8
/// n=6: bpf_wordalign(6) == 8
/// ...
/// n=9: bpf_wordalign(9) == 12
/// ```
const fn bpf_wordalign(n: u32) -> u32 {
    const ALIGNMENT: u32 = BPF_ALIGNMENT as u32;
    (n + (ALIGNMENT - 1)) & (!(ALIGNMENT - 1))
}

#[test]
fn test_alignment() {
    assert_eq!(bpf_wordalign(0), 0);
    assert_eq!(bpf_wordalign(1), 4);
    assert_eq!(bpf_wordalign(4), 4);
    assert_eq!(bpf_wordalign(5), 8);
}
