use std::{
    io, mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::windows::io::RawHandle,
    ptr,
    sync::Mutex,
};
use winapi::shared::{
    ifdef::NET_LUID,
    in6addr::IN6_ADDR,
    inaddr::IN_ADDR,
    netioapi::{
        CancelMibChangeNotify2, GetIpInterfaceEntry, MibAddInstance, NotifyIpInterfaceChange,
        SetIpInterfaceEntry, MIB_IPINTERFACE_ROW,
    },
    ntdef::FALSE,
    winerror::{ERROR_NOT_FOUND, NO_ERROR},
    ws2def::{AF_INET, AF_INET6, AF_UNSPEC},
    ws2ipdef::SOCKADDR_INET,
};


#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Unknown address family
    #[error(display = "Unknown address family: {}", _0)]
    UnknownAddressFamily(i32),
}

/// Context for [`notify_ip_interface_change`]. When it is dropped,
/// the callback is unregistered.
pub struct IpNotifierHandle<'a> {
    callback: Mutex<Box<dyn FnMut(&MIB_IPINTERFACE_ROW, u32) + Send + 'a>>,
    handle: RawHandle,
}

unsafe impl Send for IpNotifierHandle<'_> {}

impl<'a> Drop for IpNotifierHandle<'a> {
    fn drop(&mut self) {
        unsafe { CancelMibChangeNotify2(self.handle as *mut _) };
    }
}

unsafe extern "system" fn inner_callback(
    context: *mut winapi::ctypes::c_void,
    row: *mut MIB_IPINTERFACE_ROW,
    notify_type: u32,
) {
    let context = &mut *(context as *mut IpNotifierHandle<'_>);
    context
        .callback
        .lock()
        .expect("NotifyIpInterfaceChange mutex poisoned")(&*row, notify_type);
}

/// Registers a callback function that is invoked when an interface is added, removed,
/// or changed.
pub fn notify_ip_interface_change<'a, T: FnMut(&MIB_IPINTERFACE_ROW, u32) + Send + 'a>(
    callback: T,
    family: u16,
) -> io::Result<Box<IpNotifierHandle<'a>>> {
    let mut context = Box::new(IpNotifierHandle {
        callback: Mutex::new(Box::new(callback)),
        handle: std::ptr::null_mut(),
    });

    let status = unsafe {
        NotifyIpInterfaceChange(
            family,
            Some(inner_callback),
            &mut *context as *mut _ as *mut _,
            FALSE,
            (&mut context.handle) as *mut _,
        )
    };

    if status == NO_ERROR {
        Ok(context)
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
}

/// Returns information about a network IP interface.
pub fn get_ip_interface_entry(family: u16, luid: &NET_LUID) -> io::Result<MIB_IPINTERFACE_ROW> {
    let mut row: MIB_IPINTERFACE_ROW = unsafe { mem::zeroed() };
    row.Family = family;
    row.InterfaceLuid = *luid;

    let result = unsafe { GetIpInterfaceEntry(&mut row) };
    if result == NO_ERROR {
        Ok(row)
    } else {
        Err(io::Error::from_raw_os_error(result as i32))
    }
}

/// Set the properties of an IP interface.
pub fn set_ip_interface_entry(row: &MIB_IPINTERFACE_ROW) -> io::Result<()> {
    let result = unsafe { SetIpInterfaceEntry(row as *const _ as *mut _) };
    if result == NO_ERROR {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(result as i32))
    }
}

fn ip_interface_entry_exists(family: u16, luid: &NET_LUID) -> io::Result<bool> {
    match get_ip_interface_entry(family, luid) {
        Ok(_) => Ok(true),
        Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => Ok(false),
        Err(error) => Err(error),
    }
}

/// Waits until the specified IP interfaces have attached to a given network interface.
pub async fn wait_for_interfaces(luid: NET_LUID, ipv4: bool, ipv6: bool) -> io::Result<()> {
    let (tx, rx) = futures::channel::oneshot::channel();

    let mut found_ipv4 = if ipv4 { false } else { true };
    let mut found_ipv6 = if ipv6 { false } else { true };

    let mut tx = Some(tx);

    let _handle = notify_ip_interface_change(
        move |row, notification_type| {
            if found_ipv4 && found_ipv6 {
                return;
            }
            if notification_type != MibAddInstance {
                return;
            }
            if row.InterfaceLuid.Value != luid.Value {
                return;
            }
            match row.Family as i32 {
                AF_INET => found_ipv4 = true,
                AF_INET6 => found_ipv6 = true,
                _ => (),
            }
            if found_ipv4 && found_ipv6 {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(());
                }
            }
        },
        AF_UNSPEC as u16,
    )?;

    // Make sure they don't already exist
    if (!ipv4 || ip_interface_entry_exists(AF_INET as u16, &luid)?)
        && (!ipv6 || ip_interface_entry_exists(AF_INET6 as u16, &luid)?)
    {
        return Ok(());
    }

    let _ = rx.await;
    Ok(())
}


/// Converts an `Ipv4Addr` to `IN_ADDR`
pub fn inaddr_from_ipaddr(addr: Ipv4Addr) -> IN_ADDR {
    let mut in_addr: IN_ADDR = unsafe { mem::zeroed() };
    let addr_octets = addr.octets();
    unsafe {
        ptr::copy_nonoverlapping(
            &addr_octets as *const _,
            in_addr.S_un.S_addr_mut() as *mut _ as *mut u8,
            addr_octets.len(),
        );
    }
    in_addr
}

/// Converts an `Ipv6Addr` to `IN6_ADDR`
pub fn in6addr_from_ipaddr(addr: Ipv6Addr) -> IN6_ADDR {
    let mut in_addr: IN6_ADDR = unsafe { mem::zeroed() };
    let addr_octets = addr.octets();
    unsafe {
        ptr::copy_nonoverlapping(
            &addr_octets as *const _,
            in_addr.u.Byte_mut() as *mut _,
            addr_octets.len(),
        );
    }
    in_addr
}

/// Converts an `IN_ADDR` to `Ipv4Addr`
pub fn ipaddr_from_inaddr(addr: IN_ADDR) -> Ipv4Addr {
    Ipv4Addr::from(unsafe { *(addr.S_un.S_addr()) }.to_be())
}

/// Converts an `IN6_ADDR` to `Ipv6Addr`
pub fn ipaddr_from_in6addr(addr: IN6_ADDR) -> Ipv6Addr {
    Ipv6Addr::from(*unsafe { addr.u.Byte() })
}

/// Converts a `SocketAddr` to `SOCKADDR_INET`
pub fn inet_sockaddr_from_socketaddr(addr: SocketAddr) -> SOCKADDR_INET {
    let mut sockaddr: SOCKADDR_INET = unsafe { mem::zeroed() };

    match addr {
        SocketAddr::V4(v4_addr) => {
            unsafe {
                *sockaddr.si_family_mut() = AF_INET as u16;
            }

            let mut v4sockaddr = unsafe { sockaddr.Ipv4_mut() };
            v4sockaddr.sin_family = AF_INET as u16;
            v4sockaddr.sin_port = v4_addr.port().to_be();
            v4sockaddr.sin_addr = inaddr_from_ipaddr(*v4_addr.ip());
        }
        SocketAddr::V6(v6_addr) => {
            unsafe {
                *sockaddr.si_family_mut() = AF_INET6 as u16;
            }

            let mut v6sockaddr = unsafe { sockaddr.Ipv6_mut() };
            v6sockaddr.sin6_family = AF_INET6 as u16;
            v6sockaddr.sin6_port = v6_addr.port().to_be();
            v6sockaddr.sin6_addr = in6addr_from_ipaddr(*v6_addr.ip());
            v6sockaddr.sin6_flowinfo = v6_addr.flowinfo();
            *unsafe { v6sockaddr.u.sin6_scope_id_mut() } = v6_addr.scope_id();
        }
    }

    sockaddr
}

/// Converts a `SOCKADDR_INET` to `SocketAddr`. Returns an error if the address family is invalid.
pub fn try_socketaddr_from_inet_sockaddr(addr: SOCKADDR_INET) -> Result<SocketAddr, Error> {
    unsafe {
        match *addr.si_family() as i32 {
            AF_INET => Ok(SocketAddr::V4(SocketAddrV4::new(
                ipaddr_from_inaddr(addr.Ipv4().sin_addr),
                u16::from_be(addr.Ipv4().sin_port),
            ))),
            AF_INET6 => Ok(SocketAddr::V6(SocketAddrV6::new(
                ipaddr_from_in6addr(addr.Ipv6().sin6_addr),
                u16::from_be(addr.Ipv6().sin6_port),
                addr.Ipv6().sin6_flowinfo,
                *addr.Ipv6().u.sin6_scope_id(),
            ))),
            family => Err(Error::UnknownAddressFamily(family)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sockaddr_v4() {
        let addr_v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 1234));
        assert_eq!(
            addr_v4,
            try_socketaddr_from_inet_sockaddr(inet_sockaddr_from_socketaddr(addr_v4)).unwrap()
        );
    }

    #[test]
    fn test_sockaddr_v6() {
        let addr_v6 = SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8),
            1234,
            0xa,
            0xb,
        ));
        assert_eq!(
            addr_v6,
            try_socketaddr_from_inet_sockaddr(inet_sockaddr_from_socketaddr(addr_v6)).unwrap()
        );
    }
}
