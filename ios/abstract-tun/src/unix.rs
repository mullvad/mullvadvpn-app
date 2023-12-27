use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    os::fd::{IntoRawFd, RawFd},
    ptr,
    sync::Arc,
};
use tun::{platform, Configuration, Device};

use crate::TunnelTransport;

pub struct TunWriteHandle {
    dev: Arc<TunHandle>,
}

#[derive(Clone)]
pub struct TunReadHandle {
    dev: Arc<TunHandle>,
}

impl TunWriteHandle {
    pub fn read_handle(&self) -> TunReadHandle {
        TunReadHandle {
            dev: self.dev.clone(),
        }
    }

    pub fn new(addr: Ipv4Addr) -> Result<TunWriteHandle, Box<dyn std::error::Error>> {
        let config = Configuration::default();

        let mut tun = platform::create(&config)?;
        tun.set_address(addr)?;
        let dev = TunHandle {
            fd: tun.into_raw_fd(),
        };

        Ok(TunWriteHandle { dev: Arc::new(dev) })
    }

    pub fn send_packet(&self, buffer: &[u8], af: u8) -> io::Result<()> {
        let _ = self.dev.write(buffer, af)?;
        Ok(())
    }
}

impl TunReadHandle {
    pub fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.dev.read(buffer)
    }
}

impl TunnelTransport for TunWriteHandle {
    fn send_v4_packet(&self, buffer: &[u8]) -> io::Result<()> {
        self.send_packet(buffer, libc::AF_INET.try_into().unwrap())
    }

    fn send_v6_packet(&self, buffer: &[u8]) -> io::Result<()> {
        self.send_packet(buffer, libc::AF_INET6.try_into().unwrap())
    }
}

struct TunHandle {
    fd: RawFd,
}

impl TunHandle {
    fn read<'a>(&self, dst: &'a mut [u8]) -> Result<usize, io::Error> {
        let mut hdr = [0u8; 4];

        let mut iov = [
            libc::iovec {
                iov_base: hdr.as_mut_ptr() as _,
                iov_len: hdr.len(),
            },
            libc::iovec {
                iov_base: dst.as_mut_ptr() as _,
                iov_len: dst.len(),
            },
        ];

        let mut msg_hdr = libc::msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov[0],
            msg_iovlen: iov.len() as _,
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        match unsafe { libc::recvmsg(self.fd, &mut msg_hdr, 0) } {
            -1 => Err(io::Error::last_os_error()),
            0..=4 => Ok(0),
            n => Ok((n - 4) as usize),
        }
    }

    fn write(&self, src: &[u8], af: u8) -> io::Result<usize> {
        let mut hdr = [0u8, 0u8, 0u8, af as u8];
        let mut iov = [
            libc::iovec {
                iov_base: hdr.as_mut_ptr() as _,
                iov_len: hdr.len(),
            },
            libc::iovec {
                iov_base: src.as_ptr() as _,
                iov_len: src.len(),
            },
        ];

        let msg_hdr = libc::msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov[0],
            msg_iovlen: iov.len() as _,
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        match unsafe { libc::sendmsg(self.fd, &msg_hdr, 0) } {
            -1 => Err(io::Error::last_os_error()),
            n => Ok(n as usize),
        }
    }
}

impl Drop for TunHandle {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd);
    }
}

#[derive(Clone)]
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
}

impl UdpTransport {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            socket: Arc::new(UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?),
        })
    }

    pub fn with_listen_addr(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            socket: Arc::new(UdpSocket::bind(addr)?),
        })
    }

    pub fn receive_packet(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buffer)
    }
}

impl crate::UdpTransport for UdpTransport {
    fn send_packet(&self, addr: std::net::SocketAddr, buffer: &[u8]) -> io::Result<()> {
        let _ = self.socket.send_to(buffer, addr)?;
        Ok(())
    }
}
