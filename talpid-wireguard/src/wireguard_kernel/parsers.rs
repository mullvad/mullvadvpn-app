use byteorder::{ByteOrder, NativeEndian};
use nix::sys::{socket::InetAddr, time::TimeSpec};
use std::{
    ffi::{CStr, CString},
    mem,
    net::IpAddr,
};

pub use netlink_packet_utils::parsers::*;
use netlink_packet_utils::DecodeError;

pub fn parse_ip_addr(bytes: &[u8]) -> Result<IpAddr, DecodeError> {
    if bytes.len() == 4 {
        let mut ipv4_bytes = [0u8; 4];
        ipv4_bytes.copy_from_slice(bytes);
        Ok(IpAddr::from(ipv4_bytes))
    } else if bytes.len() == 16 {
        let mut ipv6_bytes = [0u8; 16];
        ipv6_bytes.copy_from_slice(bytes);
        Ok(IpAddr::from(ipv6_bytes))
    } else {
        log::error!("Expected either 4 or 16 bytes, got {} bytes", bytes.len());
        Err(format!("Invalid bytes for IP address: {bytes:?}").into())
    }
}

pub fn parse_wg_key(buffer: &[u8]) -> Result<[u8; 32], DecodeError> {
    match buffer.len() {
        32 => {
            let mut key = [0u8; 32];
            key.clone_from_slice(buffer);
            Ok(key)
        }
        anything_else => Err(format!("Unexpected length of key: {anything_else}").into()),
    }
}

pub fn parse_inet_sockaddr(buffer: &[u8]) -> Result<InetAddr, DecodeError> {
    if buffer.len() != mem::size_of::<libc::sockaddr_in6>()
        && buffer.len() != mem::size_of::<libc::sockaddr_in>()
    {
        return Err(format!(
            "Unexpected length for sockaddr_in: {}, expected {} or {}",
            buffer.len(),
            mem::size_of::<libc::sockaddr_in6>(),
            mem::size_of::<libc::sockaddr_in>()
        )
        .into());
    }
    let ptr = buffer.as_ptr();
    const AF_INET: u16 = libc::AF_INET as u16;
    const AF_INET6: u16 = libc::AF_INET6 as u16;

    match NativeEndian::read_u16(buffer) {
        AF_INET => unsafe {
            let sockaddr: *const libc::sockaddr_in = ptr as *const _;
            Ok(InetAddr::V4(*sockaddr))
        },
        AF_INET6 => unsafe {
            let sockaddr: *const libc::sockaddr_in6 = ptr as *const _;
            Ok(InetAddr::V6(*sockaddr))
        },
        unexpected_addr_family => {
            Err(format!("Unexpected address family: {unexpected_addr_family}").into())
        }
    }
}

pub fn parse_timespec(buffer: &[u8]) -> Result<TimeSpec, DecodeError> {
    if buffer.len() != mem::size_of::<libc::timespec>() {
        return Err(format!("Unexpected size for timespec: {}", buffer.len()).into());
    }

    Ok(TimeSpec::from(libc::timespec {
        tv_sec: NativeEndian::read_i64(buffer),
        // TODO: become compatible with 32-bit systems maybe?
        tv_nsec: NativeEndian::read_i64(buffer),
    }))
}

pub fn parse_cstring(buffer: &[u8]) -> Result<CString, DecodeError> {
    Ok(CStr::from_bytes_with_nul(buffer)
        .map_err(|err| format!("{err}"))?
        .into())
}

pub fn parse_genlmsghdr(buffer: &[u8]) -> Result<(u8, u8), DecodeError> {
    const GENLMSGHDR_SIZE: usize = mem::size_of::<libc::genlmsghdr>();
    if buffer.len() < GENLMSGHDR_SIZE {
        return Err(format!(
            "Expected at least {}, got {}",
            GENLMSGHDR_SIZE,
            buffer.len()
        )
        .into());
    }

    Ok((buffer[0], buffer[1]))
}
