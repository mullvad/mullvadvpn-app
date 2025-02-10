#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::{io, os::fd::AsRawFd};

use socket2::Domain;
use talpid_types::ErrorExt;

#[cfg(target_os = "macos")]
const SIOCSIFMTU: u64 = 0x80206934;
#[cfg(target_os = "macos")]
const SIOCGIFMTU: u64 = 0xc0206933;
#[cfg(target_os = "linux")]
const SIOCSIFMTU: u64 = libc::SIOCSIFMTU;
#[cfg(target_os = "linux")]
const SIOCGIFMTU: u64 = libc::SIOCSIFMTU;

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
            &mut ifr.ifr_name as *mut _,
            interface_name.len(),
        )
    };
    ifr.ifr_ifru.ifru_mtu = mtu as i32;

    // SAFETY: SIOCSIFMTU expects an ifreq with an MTU and interface set
    if unsafe { libc::ioctl(sock.as_raw_fd(), SIOCSIFMTU, &ifr) } < 0 {
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
            &mut ifr.ifr_name as *mut _,
            interface_name.len(),
        )
    };

    // SAFETY: SIOCGIFMTU expects an ifreq with an interface set
    if unsafe { libc::ioctl(sock.as_raw_fd(), SIOCGIFMTU, &ifr) } < 0 {
        let e = std::io::Error::last_os_error();
        log::error!("{}", e.display_chain_with_msg("SIOCGIFMTU failed"));
        return Err(e);
    }
    // SAFETY: ifru_mtu is initialized by SIOCGIFMTU
    Ok(u16::try_from(unsafe { ifr.ifr_ifru.ifru_mtu }).unwrap())
}
