use futures::ready;
use libc::{
    bpf_hdr, ifreq, BIOCGBLEN, BIOCGDLT, BIOCGSTATS, BIOCIMMEDIATE, BIOCSBLEN, BIOCSETIF,
    BIOCSHDRCMPLT, BIOCSSEESENT, BPF_ALIGNMENT, EBUSY, F_GETFL, F_SETFL, O_NONBLOCK,
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

use super::bindings::{bpf_stat, BIOCSWANTPKTAP};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to open BPF device
    #[error("Failed to open BPF device")]
    OpenBpfDevice(#[source] io::Error),
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

pub struct Stats {
    pub dropped: u32,
    pub recv: u32,
}

pub struct ReadHalf(File);

pub struct WriteHalf(File);

impl Bpf {
    pub fn open() -> Result<Self, Error> {
        Ok(Self {
            file: Self::open_device()?,
        })
    }

    pub fn split(self) -> (ReadHalf, WriteHalf) {
        let dup = self.file.try_clone().unwrap();
        (ReadHalf(dup), WriteHalf(self.file))
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
                Err(error) => {
                    if error.raw_os_error() == Some(EBUSY) {
                        // This BPF device is in use
                        continue;
                    }
                    return Err(Error::OpenBpfDevice(error));
                }
            }
        }
        Err(Error::NoFreeBpfDeviceFound)
    }

    pub fn set_nonblocking(&self, enabled: bool) -> Result<(), Error> {
        let mut flags = unsafe { libc::fcntl(self.as_raw_fd(), F_GETFL) };
        if flags == -1 {
            return Err(Error::GetFileFlags(io::Error::last_os_error()));
        }
        if enabled {
            flags |= O_NONBLOCK;
        } else {
            flags &= !O_NONBLOCK;
        }

        let result = unsafe { libc::fcntl(self.as_raw_fd(), F_SETFL, flags) };
        if result == -1 {
            return Err(Error::SetFileFlags(io::Error::last_os_error()));
        }
        Ok(())
    }

    pub fn get_stats(&self) -> Result<Stats, Error> {
        let mut raw_stats: bpf_stat = unsafe { mem::zeroed() };
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCGSTATS, &mut raw_stats) }?;

        Ok(Stats {
            dropped: raw_stats.bs_drop,
            recv: raw_stats.bs_recv,
        })
    }

    /// Set BIOCSETIF
    pub fn set_interface(&self, name: &str) -> Result<(), Error> {
        let mut ifr: ifreq = unsafe { std::mem::zeroed() };

        let name_bytes = name.as_bytes();
        if name_bytes.len() >= std::mem::size_of_val(&ifr.ifr_name) {
            return Err(Error::InterfaceNameTooLong);
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                name_bytes.as_ptr(),
                &mut ifr.ifr_name as *mut _ as *mut _,
                name_bytes.len(),
            );
            ioctl!(self.file.as_raw_fd(), BIOCSETIF, &ifr)
        }
    }

    /// Enable or disable immediate mode (BIOCIMMEDIATE)
    pub fn set_immediate(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCIMMEDIATE, &enable) }
    }

    // See locally sent packets (BIOCSSEESENT)
    pub fn set_see_sent(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSSEESENT, &enable) }
    }

    /// Enable or disable locally sent messages (BIOCSHDRCMPLT)
    pub fn set_header_complete(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSHDRCMPLT, &enable) }
    }

    pub fn set_want_pktap(&self, enable: bool) -> Result<(), Error> {
        let enable: c_int = if enable { 1 } else { 0 };
        unsafe { ioctl!(self.file.as_raw_fd(), BIOCSWANTPKTAP, &enable) }
    }

    pub fn set_buffer_size(&self, mut buffer_size: c_uint) -> Result<usize, Error> {
        unsafe {
            ioctl!(self.file.as_raw_fd(), BIOCSBLEN, &mut buffer_size)?;
        }
        Ok(buffer_size as usize)
    }

    pub fn required_buffer_size(&self) -> Result<usize, Error> {
        let mut buf_size = 0i32;
        unsafe {
            ioctl!(self.file.as_raw_fd(), BIOCGBLEN, &mut buf_size)?;
        }
        Ok(buf_size as usize)
    }

    pub fn dlt(&self) -> Result<u32, Error> {
        let mut dlt = 0;
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
        // TODO: verify
        Ok(())
    }
}

impl Write for &Bpf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.file).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // no-op
        // TODO: verify
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
        // TODO: verify
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

pub fn parse_bpf_header<'a>(data: &'a [u8]) -> Option<(&'a bpf_hdr, usize)> {
    if data.len() < mem::size_of::<bpf_hdr>() {
        return None;
    }
    let bpf_header: &bpf_hdr = unsafe { &*(data.as_ptr() as *const u8 as *const bpf_hdr) };
    Some((
        bpf_header,
        bpf_wordalign(bpf_header.bh_hdrlen as u32 + bpf_header.bh_caplen as u32) as usize,
    ))
}

const fn bpf_wordalign(n: u32) -> u32 {
    const ALIGNMENT: u32 = BPF_ALIGNMENT as u32;
    return (n + (ALIGNMENT - 1)) & (!(ALIGNMENT - 1));
}

#[test]
fn test_alignment() {
    assert_eq!(bpf_wordalign(0), 0);
    assert_eq!(bpf_wordalign(1), 4);
    assert_eq!(bpf_wordalign(4), 4);
    assert_eq!(bpf_wordalign(5), 8);
}
