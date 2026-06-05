#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use socket2::SockAddr;
use std::{
    ffi::{OsStr, OsString},
    fmt, io,
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::windows::ffi::{OsStrExt, OsStringExt},
    ptr::{self, NonNull},
    sync::Mutex,
    time::{Duration, Instant},
};
use talpid_types::win32_err;
use windows_sys::{
    Win32::{
        Foundation::{ERROR_NOT_FOUND, HANDLE},
        NetworkManagement::{
            IpHelper::{
                ConvertInterfaceAliasToLuid, ConvertInterfaceLuidToAlias,
                ConvertInterfaceLuidToGuid, ConvertInterfaceLuidToIndex,
                CreateUnicastIpAddressEntry, FreeMibTable, GetUnicastIpAddressEntry,
                GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, MIB_IPINTERFACE_ROW,
                MIB_UNICASTIPADDRESS_ROW, MIB_UNICASTIPADDRESS_TABLE, MibAddInstance,
                SetIpInterfaceEntry,
            },
            Ndis::{IF_MAX_STRING_SIZE, NET_LUID_LH},
        },
        Networking::WinSock::{
            AF_INET, AF_INET6, AF_UNSPEC, IN_ADDR, IN6_ADDR, IpDadStateDeprecated,
            IpDadStateDuplicate, IpDadStateInvalid, IpDadStatePreferred, IpDadStateTentative,
            NL_DAD_STATE, SOCKADDR_IN as sockaddr_in, SOCKADDR_IN6 as sockaddr_in6, SOCKADDR_INET,
        },
    },
    core::GUID,
};

/// Result type for this module.
pub type Result<T> = std::result::Result<T, Error>;

const DAD_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const DAD_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Errors returned by some functions in this module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error returned from `ConvertInterfaceAliasToLuid`
    #[cfg(windows)]
    #[error("Cannot find LUID for virtual adapter")]
    NoDeviceLuid(#[source] io::Error),

    /// Error returned from `GetUnicastIpAddressTable`/`GetUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error("Failed to obtain unicast IP address table")]
    ObtainUnicastAddress(#[source] io::Error),

    /// `GetUnicastIpAddressTable` contained no addresses for the interface
    #[cfg(windows)]
    #[error("Found no addresses for the given adapter")]
    NoUnicastAddress,

    /// Error returned from `CreateUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error("Failed to create unicast IP address")]
    CreateUnicastEntry(#[source] io::Error),

    /// Unexpected DAD state returned for a unicast address
    #[cfg(windows)]
    #[error("Unexpected DAD state")]
    DadStateError(#[source] DadStateError),

    /// Failed to start interface check.
    #[cfg(windows)]
    #[error("Error waiting on IP interfaces")]
    StartIpInterfaceNotify(#[source] io::Error),

    /// Interface check failed.
    #[cfg(windows)]
    #[error("Timed out waiting on IP interfaces")]
    IpInterfaceTimeout,

    /// DAD check failed.
    #[cfg(windows)]
    #[error("Timed out waiting on tunnel device")]
    DeviceReadyTimeout,

    /// Unicast DAD check fail.
    #[cfg(windows)]
    #[error("Unicast channel sender was unexpectedly dropped")]
    UnicastSenderDropped,

    /// Unknown address family
    #[error("Unknown address family: {0}")]
    UnknownAddressFamily(u16),
}

/// Handles cases where there DAD state is neither tentative nor preferred.
#[derive(thiserror::Error, Debug)]
pub enum DadStateError {
    /// Invalid DAD state.
    #[error("Invalid DAD state")]
    Invalid,

    /// Duplicate unicast address.
    #[error("A duplicate IP address was detected")]
    Duplicate,

    /// Deprecated unicast address.
    #[error("The IP address has been deprecated")]
    Deprecated,

    /// Unknown DAD state constant.
    #[error("Unknown DAD state: {0}")]
    Unknown(i32),
}

#[expect(non_upper_case_globals)]
impl From<NL_DAD_STATE> for DadStateError {
    fn from(state: NL_DAD_STATE) -> DadStateError {
        match state {
            IpDadStateInvalid => DadStateError::Invalid,
            IpDadStateDuplicate => DadStateError::Duplicate,
            IpDadStateDeprecated => DadStateError::Deprecated,
            other => DadStateError::Unknown(other),
        }
    }
}

impl AddressFamily {
    /// Convert one of the `AF_*` constants to an [`AddressFamily`].
    pub fn try_from_af_family(family: u16) -> Result<AddressFamily> {
        match family {
            AF_INET => Ok(AddressFamily::Ipv4),
            AF_INET6 => Ok(AddressFamily::Ipv6),
            family => Err(Error::UnknownAddressFamily(family)),
        }
    }

    /// Convert an [`AddressFamily`] to one of the `AF_*` constants.
    pub fn to_af_family(&self) -> u16 {
        match self {
            Self::Ipv4 => AF_INET,
            Self::Ipv6 => AF_INET6,
        }
    }
}

type InnerCallback = Box<Mutex<dyn FnMut(&MIB_IPINTERFACE_ROW, i32) + Send + 'static>>;

/// Context for [`notify_ip_interface_change`]. When it is dropped,
/// the callback is unregistered.
pub struct IpNotifierHandle {
    callback: Option<NonNull<InnerCallback>>,
    handle: HANDLE,
}

unsafe impl Send for IpNotifierHandle {}

impl Drop for IpNotifierHandle {
    fn drop(&mut self) {
        #[cfg(not(test))]
        use windows_sys::Win32::NetworkManagement::IpHelper::CancelMibChangeNotify2;

        #[cfg(test)]
        use tests::fake_cancel_mib_change_notify2 as CancelMibChangeNotify2;

        // SAFETY: `self.handle` is a valid notify handle that we own
        unsafe { CancelMibChangeNotify2(self.handle) };

        let callback = self
            .callback
            .take()
            .expect("callback is Some until drop is called");
        let callback = callback.as_ptr();
        // SAFETY:
        // - Callback was constructed in `notify_ip_interface_change` using `Box::into_raw`.
        // - `CancelMibChangeNotify2` ensures that the callback is removed, so we can safely take ownership.
        let _inner_callback: Box<InnerCallback> = unsafe { Box::from_raw(callback) };
    }
}

unsafe extern "system" fn outer_callback(
    context: *const std::ffi::c_void,
    row: *const MIB_IPINTERFACE_ROW,
    notify_type: i32,
) {
    // SAFETY: `context` is a valid pointer to an `InnerCallback` constructed in `notify_ip_interface_change`.
    // `outer_callback` is never called after `CancelMibChangeNotify2` has completed, and `CancelMibChangeNotify2`
    // blocks until the function returns if it is currently being called.
    let cb = unsafe { &*context.cast::<InnerCallback>() };
    // SAFETY: `row` is set when type is not `MibInitialNotification`, which we do not use.
    let row = unsafe { &*row };
    cb.lock().expect("NotifyIpInterfaceChange mutex poisoned")(row, notify_type);
}

/// Registers a callback function that is invoked when an interface is added, removed,
/// or changed.
pub fn notify_ip_interface_change<T: FnMut(&MIB_IPINTERFACE_ROW, i32) + Send + 'static>(
    callback: T,
    family: Option<AddressFamily>,
) -> io::Result<IpNotifierHandle> {
    // Box mutex because fat pointer
    let callback = Box::new(Mutex::new(callback)) as Box<Mutex<_>>;
    let callback: Box<InnerCallback> = Box::new(callback);
    let callback = NonNull::new(Box::into_raw(callback)).unwrap();

    let mut context = IpNotifierHandle {
        callback: Some(callback),
        handle: ptr::null_mut(),
    };

    #[cfg(not(test))]
    use windows_sys::Win32::NetworkManagement::IpHelper::NotifyIpInterfaceChange;

    #[cfg(test)]
    use tests::fake_notify_ip_interface_change as NotifyIpInterfaceChange;

    win32_err!(unsafe {
        NotifyIpInterfaceChange(
            af_family_from_family(family),
            Some(outer_callback),
            callback.as_ptr().cast(),
            false,
            &raw mut context.handle,
        )
    })?;
    Ok(context)
}

/// Returns information about a network IP interface.
pub fn get_ip_interface_entry(
    family: AddressFamily,
    luid: &NET_LUID_LH,
) -> io::Result<MIB_IPINTERFACE_ROW> {
    let mut row = MIB_IPINTERFACE_ROW {
        Family: family as u16,
        InterfaceLuid: *luid,
        ..Default::default()
    };

    #[cfg(not(test))]
    use windows_sys::Win32::NetworkManagement::IpHelper::GetIpInterfaceEntry;

    #[cfg(test)]
    use tests::fake_get_ip_interface_entry_fail as GetIpInterfaceEntry;

    win32_err!(unsafe { GetIpInterfaceEntry(&raw mut row) })?;
    Ok(row)
}

/// Set the properties of an IP interface.
pub fn set_ip_interface_entry(row: &mut MIB_IPINTERFACE_ROW) -> io::Result<()> {
    win32_err!(unsafe { SetIpInterfaceEntry(row as *mut _) })
}

fn ip_interface_entry_exists(family: AddressFamily, luid: &NET_LUID_LH) -> io::Result<bool> {
    match get_ip_interface_entry(family, luid) {
        Ok(_) => Ok(true),
        Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => Ok(false),
        Err(error) => Err(error),
    }
}

/// Waits until the specified IP interfaces have attached to a given network interface.
pub async fn wait_for_interfaces(luid: NET_LUID_LH, ipv4: bool, ipv6: bool) -> io::Result<()> {
    let (tx, rx) = futures::channel::oneshot::channel();

    let on_found = move || {
        let _ = tx.send(());
    };
    match start_wait_for_interfaces(luid, ipv4, ipv6, on_found)? {
        StartNotifyResult::AlreadyExist => Ok(()),
        StartNotifyResult::Waiting(_handle) => {
            let _ = rx.await;
            Ok(())
        }
    }
}

/// Waits until the specified IP interfaces have appeared for a given network device.
/// This fails if the interfaces have not appeared after the specified `timeout`.
pub fn wait_for_interfaces_sync(
    luid: NET_LUID_LH,
    ipv4: bool,
    ipv6: bool,
    timeout: Duration,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    let on_found = move || {
        let _ = tx.send(());
    };
    match start_wait_for_interfaces(luid, ipv4, ipv6, on_found)
        .map_err(Error::StartIpInterfaceNotify)?
    {
        StartNotifyResult::AlreadyExist => Ok(()),
        StartNotifyResult::Waiting(_handle) => rx
            .recv_timeout(timeout)
            .map_err(|_| Error::IpInterfaceTimeout),
    }
}

enum StartNotifyResult {
    AlreadyExist,
    Waiting(IpNotifierHandle),
}

/// Begins to wait until the specified IP interfaces have attached to a given network interface.
///
/// `StartNotifyResult::AlreadyExist` is returned if requested interfaces already exist.
///
/// Otherwise, on success, `on_found` is called when all requested interfaces have been added.
/// The wait is cancelled if the returned handle is dropped.
fn start_wait_for_interfaces(
    luid: NET_LUID_LH,
    ipv4: bool,
    ipv6: bool,
    on_found: impl FnOnce() + Send + 'static,
) -> io::Result<StartNotifyResult> {
    let mut found_ipv4 = !ipv4;
    let mut found_ipv6 = !ipv6;

    let mut on_found = Some(on_found);

    let handle = notify_ip_interface_change(
        move |row, notification_type| {
            if found_ipv4 && found_ipv6 {
                return;
            }
            if notification_type != MibAddInstance {
                return;
            }
            // SAFETY: This is always valid as a `u64`.
            if unsafe { row.InterfaceLuid.Value != luid.Value } {
                return;
            }
            match row.Family {
                AF_INET => found_ipv4 = true,
                AF_INET6 => found_ipv6 = true,
                _ => (),
            }
            if found_ipv4
                && found_ipv6
                && let Some(on_found) = on_found.take()
            {
                on_found();
            }
        },
        None,
    )?;

    // Make sure the interfaces were not already up
    if (!ipv4 || ip_interface_entry_exists(AddressFamily::Ipv4, &luid)?)
        && (!ipv6 || ip_interface_entry_exists(AddressFamily::Ipv6, &luid)?)
    {
        return Ok(StartNotifyResult::AlreadyExist);
    }

    Ok(StartNotifyResult::Waiting(handle))
}

/// Wait for addresses to be usable on an network adapter.
pub async fn wait_for_addresses(luid: NET_LUID_LH) -> Result<()> {
    // Obtain unicast IP addresses
    let mut unicast_rows: Vec<MIB_UNICASTIPADDRESS_ROW> = get_unicast_table(None)
        .map_err(Error::ObtainUnicastAddress)?
        .into_iter()
        .filter(|row| unsafe { row.InterfaceLuid.Value == luid.Value })
        .collect();
    if unicast_rows.is_empty() {
        return Err(Error::NoUnicastAddress);
    }

    let (tx, rx) = futures::channel::oneshot::channel();
    let mut addr_check_thread = move || {
        // Poll DAD status using GetUnicastIpAddressEntry
        // https://docs.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-createunicastipaddressentry

        let deadline = Instant::now() + DAD_CHECK_TIMEOUT;
        while Instant::now() < deadline {
            let mut ready = true;

            for row in &mut unicast_rows {
                win32_err!(unsafe { GetUnicastIpAddressEntry(row) })
                    .map_err(Error::ObtainUnicastAddress)?;
                if row.DadState == IpDadStateTentative {
                    ready = false;
                    break;
                }
                if row.DadState != IpDadStatePreferred {
                    return Err(Error::DadStateError(DadStateError::from(row.DadState)));
                }
            }

            if ready {
                return Ok(());
            }
            std::thread::sleep(DAD_CHECK_INTERVAL);
        }

        Err(Error::DeviceReadyTimeout)
    };
    std::thread::spawn(move || {
        let _ = tx.send(addr_check_thread());
    });
    rx.await.map_err(|_| Error::UnicastSenderDropped)?
}

/// Returns the first unicast IP address for the given interface.
pub fn get_ip_address_for_interface(
    family: AddressFamily,
    luid: NET_LUID_LH,
) -> Result<Option<IpAddr>> {
    match get_unicast_table(Some(family))
        .map_err(Error::ObtainUnicastAddress)?
        .into_iter()
        .find(|row| unsafe { row.InterfaceLuid.Value == luid.Value })
    {
        Some(row) => Ok(Some(try_socketaddr_from_inet_sockaddr(row.Address)?.ip())),
        None => Ok(None),
    }
}

/// Adds a unicast IP address for the given interface.
pub fn add_ip_address_for_interface(luid: NET_LUID_LH, address: IpAddr) -> Result<()> {
    let mut row = MIB_UNICASTIPADDRESS_ROW::default();
    unsafe { InitializeUnicastIpAddressEntry(&raw mut row) };

    row.InterfaceLuid = luid;
    row.Address = inet_sockaddr_from_socketaddr(SocketAddr::new(address, 0));
    row.DadState = IpDadStatePreferred;
    row.OnLinkPrefixLength = 255;

    win32_err!(unsafe { CreateUnicastIpAddressEntry(&raw const row) })
        .map_err(Error::CreateUnicastEntry)
}

/// Sets MTU on the specified network interface identified by `luid`.
pub fn set_mtu(mtu: u32, luid: NET_LUID_LH, ip_family: AddressFamily) -> io::Result<()> {
    let mut row = get_ip_interface_entry(ip_family, &luid)?;

    row.NlMtu = mtu;

    set_ip_interface_entry(&mut row)
}

/// Returns the unicast IP address table. If `family` is `None`, then addresses for all families are
/// returned.
pub fn get_unicast_table(
    family: Option<AddressFamily>,
) -> io::Result<Vec<MIB_UNICASTIPADDRESS_ROW>> {
    let mut unicast_rows = vec![];
    let mut unicast_table: *mut MIB_UNICASTIPADDRESS_TABLE = std::ptr::null_mut();

    win32_err!(unsafe {
        GetUnicastIpAddressTable(af_family_from_family(family), &raw mut unicast_table)
    })?;
    let first_row = unsafe { &(*unicast_table).Table[0] } as *const MIB_UNICASTIPADDRESS_ROW;
    for i in 0..unsafe { *unicast_table }.NumEntries {
        unicast_rows.push(unsafe { *(first_row.offset(i as isize)) });
    }
    unsafe { FreeMibTable(unicast_table as *const _) };

    Ok(unicast_rows)
}

/// Returns the index of a network interface given its LUID.
pub fn index_from_luid(luid: &NET_LUID_LH) -> io::Result<u32> {
    let mut index = 0u32;
    win32_err!(unsafe { ConvertInterfaceLuidToIndex(luid, &raw mut index) })?;
    Ok(index)
}

/// Returns the GUID of a network interface given its LUID.
pub fn guid_from_luid(luid: &NET_LUID_LH) -> io::Result<GUID> {
    let mut guid = MaybeUninit::zeroed();
    win32_err!(unsafe { ConvertInterfaceLuidToGuid(luid, guid.as_mut_ptr()) })?;
    Ok(unsafe { guid.assume_init() })
}

/// Returns the LUID of an interface given its alias.
pub fn luid_from_alias<T: AsRef<OsStr>>(alias: T) -> io::Result<NET_LUID_LH> {
    let alias_wide: Vec<u16> = alias
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();
    let mut luid = NET_LUID_LH::default();
    win32_err!(unsafe { ConvertInterfaceAliasToLuid(alias_wide.as_ptr(), &raw mut luid) })?;
    Ok(luid)
}

/// Returns the alias of an interface given its LUID.
pub fn alias_from_luid(luid: &NET_LUID_LH) -> io::Result<OsString> {
    let mut buffer = [0u16; IF_MAX_STRING_SIZE as usize + 1];
    win32_err!(unsafe { ConvertInterfaceLuidToAlias(luid, buffer.as_mut_ptr(), buffer.len()) })?;
    let nul = buffer.iter().position(|&c| c == 0u16).unwrap();
    Ok(OsString::from_wide(&buffer[0..nul]))
}

fn af_family_from_family(family: Option<AddressFamily>) -> u16 {
    family.map(|family| family as u16).unwrap_or(AF_UNSPEC)
}

/// Converts an `Ipv4Addr` to `IN_ADDR`
pub fn inaddr_from_ipaddr(addr: Ipv4Addr) -> IN_ADDR {
    let sockaddr = SockAddr::from(SocketAddr::V4(SocketAddrV4::new(addr, 0)));
    unsafe { *(sockaddr.as_ptr() as *const sockaddr_in) }.sin_addr
}

/// Converts an `Ipv6Addr` to `IN6_ADDR`
pub fn in6addr_from_ipaddr(addr: Ipv6Addr) -> IN6_ADDR {
    let sockaddr = SockAddr::from(SocketAddr::V6(SocketAddrV6::new(addr, 0, 0, 0)));
    unsafe { *(sockaddr.as_ptr() as *const sockaddr_in6) }.sin6_addr
}

/// Converts an `IN_ADDR` to `Ipv4Addr`
pub fn ipaddr_from_inaddr(addr: IN_ADDR) -> Ipv4Addr {
    Ipv4Addr::from(unsafe { addr.S_un.S_addr }.to_ne_bytes())
}

/// Converts an `IN6_ADDR` to `Ipv6Addr`
pub fn ipaddr_from_in6addr(addr: IN6_ADDR) -> Ipv6Addr {
    Ipv6Addr::from(unsafe { addr.u.Byte })
}

/// Converts a `SocketAddr` to `SOCKADDR_INET`
pub fn inet_sockaddr_from_socketaddr(addr: SocketAddr) -> SOCKADDR_INET {
    // SAFETY: SOCKADDR_INET is a union of C structs, these can be safely zeroed.
    let mut sockaddr = SOCKADDR_INET::default();
    match addr {
        // SAFETY: `*const sockaddr` may be treated as `*const sockaddr_in` since we know it's a v4
        // address.
        SocketAddr::V4(_) => unsafe {
            sockaddr.Ipv4 = *(SockAddr::from(addr).as_ptr() as *const _)
        },
        // SAFETY: `*const sockaddr` may be treated as `*const sockaddr_in6` since we know it's a v6
        // address.
        SocketAddr::V6(_) => unsafe {
            sockaddr.Ipv6 = *(SockAddr::from(addr).as_ptr() as *const _)
        },
    }
    sockaddr
}

/// Converts a `SOCKADDR_INET` to `SocketAddr`. Returns an error if the address family is invalid.
pub fn try_socketaddr_from_inet_sockaddr(addr: SOCKADDR_INET) -> Result<SocketAddr> {
    // SAFETY: si_family is always valid
    let family = unsafe { addr.si_family };
    match family {
        AF_INET => {
            // SAFETY: We know this is an IPv4 address based on the family
            let ipv4_addr = unsafe { addr.Ipv4 };
            // SAFETY: The IPv4 address is initialized
            let ip = Ipv4Addr::from(u32::from_be(unsafe { ipv4_addr.sin_addr.S_un.S_addr }));
            let port = u16::from_be(ipv4_addr.sin_port);
            Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        }
        AF_INET6 => {
            // SAFETY: We know this is an IPv6 address based on the family
            let ipv6_addr = unsafe { addr.Ipv6 };
            // SAFETY: The IPv6 address is initialized
            let ip = Ipv6Addr::from(unsafe { ipv6_addr.sin6_addr.u.Byte });
            let port = u16::from_be(ipv6_addr.sin6_port);
            let flowinfo = ipv6_addr.sin6_flowinfo;
            // SAFETY: The scope ID is initialized
            let scope_id = unsafe { ipv6_addr.Anonymous.sin6_scope_id };
            Ok(SocketAddr::V6(SocketAddrV6::new(
                ip, port, flowinfo, scope_id,
            )))
        }
        _ => Err(Error::UnknownAddressFamily(family)),
    }
}

/// Address family. These correspond to the `AF_*` constants.
#[derive(Debug, Clone, Copy)]
pub enum AddressFamily {
    /// IPv4 address family
    Ipv4 = AF_INET as isize,
    /// IPv6 address family
    Ipv6 = AF_INET6 as isize,
}

impl fmt::Display for AddressFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AddressFamily::Ipv4 => write!(f, "IPv4 (AF_INET)"),
            AddressFamily::Ipv6 => write!(f, "IPv6 (AF_INET6)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use windows_sys::Win32::{
        Foundation::WIN32_ERROR, NetworkManagement::IpHelper::PIPINTERFACE_CHANGE_CALLBACK,
    };

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

    struct NotifyHandle {
        handle: std::thread::JoinHandle<()>,
    }

    struct NotifySettings {
        expected_luid: NET_LUID_LH,
        send_add_event_for_families: Vec<u16>,
        sleep_duration: Option<Duration>,
    }

    impl Default for NotifySettings {
        fn default() -> Self {
            NotifySettings {
                expected_luid: NET_LUID_LH { Value: 1 },
                send_add_event_for_families: vec![AF_INET, AF_INET6],
                sleep_duration: None,
            }
        }
    }

    static NOTIFY_SETTINGS: LazyLock<Mutex<NotifySettings>> = LazyLock::new(|| {
        Mutex::new(NotifySettings::default())
    });

    pub unsafe fn fake_notify_ip_interface_change(
        family: u16,
        callback: PIPINTERFACE_CHANGE_CALLBACK,
        callercontext: *const core::ffi::c_void,
        initialnotification: bool,
        notificationhandle: *mut HANDLE,
    ) -> WIN32_ERROR {
        assert_eq!(family, AF_UNSPEC);
        assert!(!initialnotification);

        struct Context {
            callback: PIPINTERFACE_CHANGE_CALLBACK,
            callercontext: *const core::ffi::c_void,
        }
        unsafe impl Send for Context {}
        let ctx = Context {
            callback,
            callercontext,
        };

        let thread = std::thread::spawn(move || {
            let ctx = ctx;

            if let Some(duration) = NOTIFY_SETTINGS.lock().unwrap().sleep_duration {
                std::thread::sleep(duration);
            }

            let cb = ctx.callback.unwrap();
            let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;

            for &family in &NOTIFY_SETTINGS.lock().unwrap().send_add_event_for_families {
                let row = MIB_IPINTERFACE_ROW {
                    InterfaceLuid: luid,
                    Family: family,
                    ..MIB_IPINTERFACE_ROW::default()
                };
                // SAFETY: Caller provided valid cb.
                unsafe { cb(ctx.callercontext, &raw const row, MibAddInstance) };
            }
        });

        let h = Box::into_raw(Box::new(NotifyHandle { handle: thread }));
        // SAFETY: Valid receiver for a `c_void` pointer.
        unsafe { *notificationhandle = h as *mut core::ffi::c_void };

        0
    }

    pub unsafe fn fake_cancel_mib_change_notify2(notificationhandle: HANDLE) -> WIN32_ERROR {
        // Block until thread exits.
        // SAFETY: Constructed once using `Box::into_raw` above.
        let h: Box<NotifyHandle> =
            unsafe { Box::from_raw(notificationhandle as *mut NotifyHandle) };
        h.handle.join().unwrap();
        0
    }

    pub unsafe fn fake_get_ip_interface_entry_fail(_row: *mut MIB_IPINTERFACE_ROW) -> WIN32_ERROR {
        ERROR_NOT_FOUND
    }

    // Serialize and reset `NOTIFY_SETTINGS` since it is globally shared between tests.
    static NOTIFY_LOCK: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));

    /// Test [`wait_for_interfaces`] using mocked notifications.
    #[tokio::test]
    async fn test_wait_for_interfaces() {
        let _guard = NOTIFY_LOCK.lock().await;
        *NOTIFY_SETTINGS.lock().unwrap() = NotifySettings::default();

        // No delay
        NOTIFY_SETTINGS.lock().unwrap().sleep_duration = None;
        let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;
        wait_for_interfaces(luid, true, false).await.unwrap();
        wait_for_interfaces(luid, true, true).await.unwrap();

        // Some delay
        NOTIFY_SETTINGS.lock().unwrap().sleep_duration = Some(Duration::from_millis(10));
        wait_for_interfaces(luid, true, false).await.unwrap();
        wait_for_interfaces(luid, true, true).await.unwrap();
    }

    /// Test [`wait_for_interfaces_sync`] using mocked notifications.
    // This can be tested with miri:
    //
    // ```rust
    // cargo +nightly miri test -p talpid-windows -- test_wait_for_interfaces_sync
    // ```
    #[test]
    fn test_wait_for_interfaces_sync() {
        let _guard = NOTIFY_LOCK.blocking_lock();
        *NOTIFY_SETTINGS.lock().unwrap() = NotifySettings::default();

        // No delay
        NOTIFY_SETTINGS.lock().unwrap().sleep_duration = None;
        let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;
        wait_for_interfaces_sync(luid, true, false, Duration::from_secs(1)).unwrap();
        wait_for_interfaces_sync(luid, true, true, Duration::from_secs(1)).unwrap();

        // Some delay
        NOTIFY_SETTINGS.lock().unwrap().sleep_duration = Some(Duration::from_millis(10));
        wait_for_interfaces_sync(luid, true, false, Duration::from_secs(1)).unwrap();
        wait_for_interfaces_sync(luid, true, true, Duration::from_secs(1)).unwrap();

        // Missing IPv6
        NOTIFY_SETTINGS.lock().unwrap().send_add_event_for_families = vec![AF_INET];
        wait_for_interfaces_sync(luid, true, true, Duration::from_millis(15)).unwrap_err();

        // Force timeout
        NOTIFY_SETTINGS.lock().unwrap().sleep_duration = Some(Duration::from_millis(100));
        wait_for_interfaces_sync(luid, true, false, Duration::from_millis(1)).unwrap_err();
    }
}
