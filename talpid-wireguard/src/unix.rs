use std::{io, os::fd::AsRawFd};

use socket2::Domain;
use talpid_types::ErrorExt;

pub fn set_mtu(interface_name: &str, mtu: u16) -> Result<(), io::Error> {
    debug_assert_ne!(
        interface_name, "eth0",
        "Should be name of mullvad tunnel interface, e.g. 'wg0-mullvad'"
    );

    let sock = socket2::Socket::new(
        Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )?;

    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    if interface_name.len() >= ifr.ifr_name.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Interface name too long",
        ));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            interface_name.as_ptr() as *const i8,
            &mut ifr.ifr_name as *mut _,
            interface_name.len(),
        )
    };
    ifr.ifr_ifru.ifru_mtu = mtu as i32;

    if unsafe { libc::ioctl(sock.as_raw_fd(), libc::SIOCSIFMTU, &ifr) } < 0 {
        let e = std::io::Error::last_os_error();
        log::error!("{}", e.display_chain_with_msg("SIOCSIFMTU failed"));
        return Err(e);
    }
    Ok(())
}
