#![cfg(any(target_os = "linux", target_os = "macos"))]

#[cfg(target_os = "linux")]
use std::ffi::c_ulong;
use std::{ffi::c_uint, io, os::fd::AsRawFd};

use nix::{errno::Errno, net::if_::if_nametoindex};
use socket2::Domain;
use talpid_types::ErrorExt;

/// Converts an interface name into the corresponding index.
pub fn iface_index(name: &str) -> Result<c_uint, IfaceIndexLookupError> {
    if_nametoindex(name).map_err(|error| IfaceIndexLookupError {
        interface_name: name.to_owned(),
        error,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get index for interface {interface_name}: {error}")]
pub struct IfaceIndexLookupError {
    pub interface_name: String,
    pub error: Errno,
}

#[cfg(target_os = "macos")]
const SIOCSIFMTU: u64 = 0x80206934;
#[cfg(target_os = "macos")]
const SIOCGIFMTU: u64 = 0xc0206933;
#[cfg(target_os = "linux")]
const SIOCSIFMTU: c_ulong = libc::SIOCSIFMTU;
#[cfg(target_os = "linux")]
const SIOCGIFMTU: c_ulong = libc::SIOCSIFMTU;

pub fn set_mtu(interface_name: &str, mtu: u16) -> Result<(), io::Error> {
    let sock = socket2::Socket::new(
        Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )?;

    // SAFETY: ifreq is a C struct, these can safely be zeroed.
    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    if interface_name.len() >= ifr.ifr_name.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Interface name too long",
        ));
    }

    // SAFETY: `interface_name.len()` is less than `ifr.ifr_name.len()`
    unsafe {
        std::ptr::copy_nonoverlapping(
            interface_name.as_ptr() as *const libc::c_char,
            ifr.ifr_name.as_mut_ptr(),
            interface_name.len(),
        )
    };
    ifr.ifr_ifru.ifru_mtu = mtu as i32;

    // For some reason, libc crate defines ioctl to take a c_int (which is defined as i32), but the c_ulong type is defined as u64:
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/fn.ioctl.html
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/type.c_ulong.html
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/constant.SIOCSIFMTU.html
    #[allow(clippy::useless_conversion)]
    let request = SIOCSIFMTU.try_into().unwrap();
    // SAFETY: SIOCSIFMTU expects an ifreq with an MTU and interface set
    if unsafe { libc::ioctl(sock.as_raw_fd(), request, &ifr) } < 0 {
        let e = std::io::Error::last_os_error();
        log::error!("{}", e.display_chain_with_msg("SIOCSIFMTU failed"));
        return Err(e);
    }
    Ok(())
}

pub fn get_mtu(interface_name: &str) -> Result<u16, io::Error> {
    let sock = socket2::Socket::new(
        Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )?;

    // SAFETY: ifreq is a C struct, these can safely be zeroed.
    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    if interface_name.len() >= ifr.ifr_name.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Interface name too long",
        ));
    }

    // SAFETY: `interface_name.len()` is less than `ifr.ifr_name.len()`
    unsafe {
        std::ptr::copy_nonoverlapping(
            interface_name.as_ptr() as *const libc::c_char,
            ifr.ifr_name.as_mut_ptr(),
            interface_name.len(),
        )
    };

    // For some reason, libc crate defines ioctl to take a c_int (which is defined as i32), but the c_ulong type is defined as u64:
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/fn.ioctl.html
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/type.c_ulong.html
    // https://docs.rs/libc/latest/x86_64-unknown-linux-musl/libc/constant.SIOCGIFMTU.html
    #[allow(clippy::useless_conversion)]
    let request = SIOCGIFMTU.try_into().unwrap();
    // SAFETY: SIOCGIFMTU expects an ifreq with an interface set
    if unsafe { libc::ioctl(sock.as_raw_fd(), request, &ifr) } < 0 {
        let e = std::io::Error::last_os_error();
        log::error!("{}", e.display_chain_with_msg("SIOCGIFMTU failed"));
        return Err(e);
    }
    // SAFETY: ifru_mtu is initialized by SIOCGIFMTU
    Ok(u16::try_from(unsafe { ifr.ifr_ifru.ifru_mtu }).unwrap())
}
