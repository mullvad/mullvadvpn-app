use byteorder::{ByteOrder, NativeEndian};
use netlink_packet_core::DecodeError;
use nix::sys::socket::{SockaddrIn, SockaddrIn6};
use std::{
    ffi::{CStr, CString},
    mem::{self, transmute},
    net::{IpAddr, SocketAddr},
};

use super::timespec::KernelTimespec;

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

pub fn parse_inet_sockaddr(buffer: &[u8]) -> Result<SocketAddr, DecodeError> {
    let wrong_len = || {
        format!(
            "Unexpected length for sockaddr_in: {}, expected {} or {}",
            buffer.len(),
            mem::size_of::<libc::sockaddr_in6>(),
            mem::size_of::<libc::sockaddr_in>()
        )
    };

    const AF_INET: u16 = libc::AF_INET as u16;
    const AF_INET6: u16 = libc::AF_INET6 as u16;

    if buffer.len() < size_of::<u16>() {
        return Err(wrong_len().into());
    }

    match NativeEndian::read_u16(buffer) {
        AF_INET => {
            let buffer: &[u8; size_of::<libc::sockaddr_in>()] =
                buffer.try_into().map_err(|_| wrong_len())?;

            // SAFETY: sockaddr_in has a defined repr(C) layout and is valid for all bit patterns
            let sockaddr: libc::sockaddr_in = unsafe { transmute(*buffer) };
            let sockaddr = SockaddrIn::from(sockaddr);

            Ok(SocketAddr::from(sockaddr))
        }
        AF_INET6 => {
            let buffer: &[u8; size_of::<libc::sockaddr_in6>()] =
                buffer.try_into().map_err(|_| wrong_len())?;

            // SAFETY: sockaddr_in6 has a defined repr(C) layout and is valid for all bit patterns
            let sockaddr: libc::sockaddr_in6 = unsafe { transmute(*buffer) };
            let sockaddr = SockaddrIn6::from(sockaddr);

            Ok(SocketAddr::from(sockaddr))
        }
        unexpected_addr_family => {
            Err(format!("Unexpected address family: {unexpected_addr_family}").into())
        }
    }
}

/// Parse the last WireGuard handshake timestamp.
/// The resulting [SystemTime] is a timestamp relative to [SystemTime::UNIX_EPOCH].
pub fn parse_last_handshake_time(buffer: &[u8]) -> Result<KernelTimespec, DecodeError> {
    use zerocopy::FromBytes;
    if buffer.len() != size_of::<KernelTimespec>() {
        return Err(format!("Unexpected size for timespec: {}", buffer.len()).into());
    }
    KernelTimespec::read_from_bytes(buffer).map_err(|err| {
        format!("Failed to decode netlink message into KernelTimespec: {err}").into()
    })
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
