#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::ffi::c_uint;
use std::io;
use std::mem;
use std::os::fd::AsRawFd;
use std::ptr;

use nix::errno::Errno;
use nix::libc::ifreq;
use nix::net::if_::if_nametoindex;
use socket2::{Domain, Protocol, Socket, Type};
use talpid_types::ErrorExt;

#[derive(Debug, thiserror::Error)]
#[error("Failed to get index for interface {interface_name}: {error}")]
pub struct IfaceIndexLookupError {
    pub interface_name: String,
    pub error: Errno,
}

/// Converts an interface name into the corresponding index.
pub fn iface_index(name: &str) -> Result<c_uint, IfaceIndexLookupError> {
    if_nametoindex(name).map_err(|error| IfaceIndexLookupError {
        interface_name: name.to_owned(),
        error,
    })
}

pub fn set_mtu(interface_name: &str, mtu: u16) -> Result<(), io::Error> {
    IfReq::new(interface_name)?.set_mtu(mtu).inspect_err(|e| {
        log::error!("{}", e.display_chain_with_msg("SIOCSIFMTU failed"));
    })
}

pub fn get_mtu(interface_name: &str) -> Result<u16, io::Error> {
    IfReq::new(interface_name)?.get_mtu().inspect_err(|e| {
        log::error!("{}", e.display_chain_with_msg("SIOCGIFMTU failed"));
    })
}

/// An [`ifreq`] initialized with an interface name.
struct IfReq {
    interface_request: ifreq,
    socket: Socket,
}

impl IfReq {
    /// Returns an [`ifreq`] refering to `interface`.
    ///
    /// - `interface`: Name of the interface (e.g. `eth0`).
    fn new(interface: &str) -> Result<Self, io::Error> {
        let invalid_input = |msg| io::Error::new(io::ErrorKind::InvalidInput, msg);
        if !interface.is_ascii() {
            return Err(invalid_input("Interface name contains UTF-8"));
        };
        let interface_name = interface.as_bytes();
        // `ifreq.ifr_name` may only contain max IF_NAMESIZE ASCII characters, including a trailing
        // null terminator.
        if interface_name.len() >= nix::libc::IF_NAMESIZE {
            return Err(invalid_input("Interface name too long"));
        };
        // SAFETY: ifreq is a C struct, these can safely be zeroed.
        let mut ifr: ifreq = unsafe { mem::zeroed() };
        // SAFETY: `interface_name.len()` does not exceed IF_NAMESIZE (+ a trailing null terminator) and `interface_name` only
        // contains ASCII.
        unsafe {
            ptr::copy_nonoverlapping(
                interface_name.as_ptr().cast::<libc::c_char>(),
                ifr.ifr_name.as_mut_ptr(),
                interface_name.len(),
            )
        };
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        Ok(Self {
            interface_request: ifr,
            socket,
        })
    }

    /// Set MTU for this interface.
    // SIOCSIFMTU in a trenchcoat.
    fn set_mtu(mut self, mtu: u16) -> Result<(), io::Error> {
        self.interface_request.ifr_ifru.ifru_mtu = i32::from(mtu);
        let socket = self.socket.as_raw_fd();
        #[cfg(target_os = "macos")]
        const SIOCSIFMTU: u64 = 0x80206934;
        #[cfg(target_os = "linux")]
        const SIOCSIFMTU: libc::c_ulong = libc::SIOCSIFMTU;
        // For some reason, libc crate defines ioctl to take a c_int (which is defined as i32), but the c_ulong type is defined as u64:
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/fn.ioctl.html
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/type.c_ulong.html
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/constant.SIOCSIFMTU.html
        #[allow(clippy::useless_conversion)]
        let request = SIOCSIFMTU.try_into().unwrap();
        // SAFETY: SIOCSIFMTU expects an ifreq with an MTU and interface set. The interface is set
        // by [Self::new].
        match unsafe { libc::ioctl(socket, request, &self.interface_request) } {
            n if n < 0 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    /// Get MTU of this interface.
    // SIOCGIFMTU in a trenchcoat.
    fn get_mtu(self) -> Result<u16, io::Error> {
        let socket = self.socket.as_raw_fd();
        #[cfg(target_os = "macos")]
        const SIOCGIFMTU: u64 = 0xc0206933;
        #[cfg(target_os = "linux")]
        const SIOCGIFMTU: libc::c_ulong = libc::SIOCSIFMTU;
        // For some reason, libc crate defines ioctl to take a c_int (which is defined as i32), but the c_ulong type is defined as u64:
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/fn.ioctl.html
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/type.c_ulong.html
        // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/constant.SIOCGIFMTU.html
        #[allow(clippy::useless_conversion)]
        let request = SIOCGIFMTU.try_into().unwrap();
        // SAFETY: SIOCGIFMTU expects an ifreq with an interface set, which is guaranteed by
        // [Self::new].
        match unsafe { libc::ioctl(socket, request, &self.interface_request) } {
            n if n < 0 => Err(io::Error::last_os_error()),
            _ => {
                // SAFETY: ifru_mtu is initialized by SIOCGIFMTU
                let mtu = unsafe { self.interface_request.ifr_ifru.ifru_mtu };
                let mtu = u16::try_from(mtu).expect("MTU of interface to be less than u16::MAX");
                Ok(mtu)
            }
        }
    }
}
