#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use socket2::SockAddr;
use std::{
    ffi::{OsStr, OsString},
    fmt, io,
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::windows::ffi::{OsStrExt, OsStringExt},
    panic::RefUnwindSafe,
    ptr::NonNull,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use talpid_types::win32_err;
use windows_sys::{
    Win32::{
        Foundation::{ERROR_NOT_FOUND, HANDLE, WIN32_ERROR},
        NetworkManagement::{
            IpHelper::{
                ConvertInterfaceAliasToLuid, ConvertInterfaceLuidToAlias,
                ConvertInterfaceLuidToGuid, ConvertInterfaceLuidToIndex,
                CreateUnicastIpAddressEntry, FreeMibTable, GetUnicastIpAddressTable,
                InitializeUnicastIpAddressEntry, MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW,
                MIB_UNICASTIPADDRESS_TABLE, MibAddInstance, SetIpInterfaceEntry,
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

    /// Failed to start unicast address check.
    #[error("Error waiting on unicast IP addresses")]
    StartUnicastAddressNotify(#[source] io::Error),

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

type InnerCallback<T> = Box<Mutex<dyn FnMut(&T, i32) + Send + 'static>>;

/// The callback function passed to the `Notify*Change` functions. `T` is the row type that the
/// notification carries.
type ChangeCallback<T> = unsafe extern "system" fn(*const core::ffi::c_void, *const T, i32);

/// Context for [`notify_ip_interface_change`] and [`notify_unicast_ip_address_change`].
/// When it is dropped, the callback is unregistered.
pub struct IpNotifierHandle<T: 'static> {
    callback: Option<NonNull<InnerCallback<T>>>,
    handle: Option<NonNull<core::ffi::c_void>>,
}

unsafe impl<T> Send for IpNotifierHandle<T> {}

impl<T> Drop for IpNotifierHandle<T> {
    fn drop(&mut self) {
        #[cfg(not(test))]
        use windows_sys::Win32::NetworkManagement::IpHelper::CancelMibChangeNotify2;

        #[cfg(test)]
        use tests::fake_cancel_mib_change_notify2 as CancelMibChangeNotify2;

        if let Some(handle) = self.handle.take() {
            // SAFETY: pointer is a valid notify handle that we own
            unsafe { CancelMibChangeNotify2(handle.as_ptr()) };
        }

        let callback = self
            .callback
            .take()
            .expect("callback is Some until drop is called");
        let callback = callback.as_ptr();
        // SAFETY:
        // - Callback was constructed in `register_change_notifier` using `Box::into_raw`.
        // - `CancelMibChangeNotify2` ensures that the callback is removed, so we can safely take ownership.
        let _inner_callback: Box<InnerCallback<T>> = unsafe { Box::from_raw(callback) };
    }
}

unsafe extern "system" fn outer_callback<T: RefUnwindSafe>(
    context: *const std::ffi::c_void,
    row: *const T,
    notify_type: i32,
) {
    // SAFETY: `context` is a valid pointer to an `InnerCallback` constructed in `register_change_notifier`.
    // `outer_callback` is never called after `CancelMibChangeNotify2` has completed, and `CancelMibChangeNotify2`
    // blocks until the function returns if it is currently being called.
    let cb = unsafe { &*context.cast::<InnerCallback<T>>() };
    // SAFETY: `row` is set when type is not `MibInitialNotification`, which we do not use.
    let row = unsafe { &*row };
    _ = std::panic::catch_unwind(|| {
        cb.lock().expect("notification mutex poisoned")(row, notify_type)
    });
}

/// Registers `callback` to be invoked for every notification delivered by the `Notify*Change`
/// function that `register` calls. `register` is handed the callback function and context pointer
/// to register, along with the out pointer for the notification handle.
fn register_change_notifier<T: RefUnwindSafe + 'static>(
    callback: impl FnMut(&T, i32) + Send + 'static,
    register: impl FnOnce(ChangeCallback<T>, *const core::ffi::c_void, *mut HANDLE) -> WIN32_ERROR,
) -> io::Result<IpNotifierHandle<T>> {
    // Box mutex because fat pointer
    let callback = Box::new(Mutex::new(callback)) as Box<Mutex<_>>;
    let callback: Box<InnerCallback<T>> = Box::new(callback);
    let callback = NonNull::new(Box::into_raw(callback)).unwrap();

    let mut handle = HANDLE::default();

    if let Err(error) = win32_err!(register(
        outer_callback::<T>,
        callback.as_ptr().cast(),
        &raw mut handle,
    )) {
        // SAFETY: The callback was never registered, so we still have sole ownership of it.
        drop(unsafe { Box::from_raw(callback.as_ptr()) });
        return Err(error);
    }

    Ok(IpNotifierHandle {
        callback: Some(callback),
        handle: Some(NonNull::new(handle).expect("non-null because registration succeeded")),
    })
}

/// Registers a callback function that is invoked when an interface is added, removed,
/// or changed.
pub fn notify_ip_interface_change<T: FnMut(&MIB_IPINTERFACE_ROW, i32) + Send + 'static>(
    callback: T,
    family: Option<AddressFamily>,
) -> io::Result<IpNotifierHandle<MIB_IPINTERFACE_ROW>> {
    #[cfg(not(test))]
    use windows_sys::Win32::NetworkManagement::IpHelper::NotifyIpInterfaceChange;

    #[cfg(test)]
    use tests::fake_notify_ip_interface_change as NotifyIpInterfaceChange;

    register_change_notifier(callback, |callback, context, handle| {
        // SAFETY: `context` points at the boxed callback owned by the returned handle, which does
        // not free it until `CancelMibChangeNotify2` has returned, so it outlives every callback.
        unsafe {
            NotifyIpInterfaceChange(
                af_family_from_family(family),
                Some(callback),
                context,
                false,
                handle,
            )
        }
    })
}

/// Registers a callback function that is invoked when a unicast IP address is added, removed,
/// or changed.
///
/// Note that the row passed to `callback` contains incomplete data. Use
/// [`get_unicast_ip_address_entry`] to obtain the current state of the address.
pub fn notify_unicast_ip_address_change<
    T: FnMut(&MIB_UNICASTIPADDRESS_ROW, i32) + Send + 'static,
>(
    callback: T,
    family: Option<AddressFamily>,
) -> io::Result<IpNotifierHandle<MIB_UNICASTIPADDRESS_ROW>> {
    #[cfg(not(test))]
    use windows_sys::Win32::NetworkManagement::IpHelper::NotifyUnicastIpAddressChange;

    #[cfg(test)]
    use tests::fake_notify_unicast_ip_address_change as NotifyUnicastIpAddressChange;

    register_change_notifier(callback, |callback, context, handle| {
        // SAFETY: `context` points at the boxed callback owned by the returned handle, which does
        // not free it until `CancelMibChangeNotify2` has returned, so it outlives every callback.
        unsafe {
            NotifyUnicastIpAddressChange(
                af_family_from_family(family),
                Some(callback),
                context,
                false,
                handle,
            )
        }
    })
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
    use tests::fake_get_ip_interface_entry as GetIpInterfaceEntry;

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

/// Waits until the specified IP interfaces have appeared for a given network device.
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

enum StartNotifyResult<T: 'static> {
    AlreadyExist,
    Waiting(IpNotifierHandle<T>),
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
) -> io::Result<StartNotifyResult<MIB_IPINTERFACE_ROW>> {
    struct FoundInterfaces {
        ipv4: bool,
        ipv6: bool,
    }

    let found_interfaces = Arc::new(Mutex::new(FoundInterfaces {
        ipv4: !ipv4,
        ipv6: !ipv6,
    }));

    let mut on_found = Some(on_found);
    let found_interfaces2 = found_interfaces.clone();

    let handle = notify_ip_interface_change(
        move |row, notification_type| {
            if notification_type != MibAddInstance {
                return;
            }
            // SAFETY: This is always valid as a `u64`.
            if unsafe { row.InterfaceLuid.Value != luid.Value } {
                return;
            }
            let mut found_interfaces = found_interfaces2.lock().unwrap();
            match row.Family {
                AF_INET => found_interfaces.ipv4 = true,
                AF_INET6 => found_interfaces.ipv6 = true,
                _ => (),
            }
            if found_interfaces.ipv4
                && found_interfaces.ipv6
                && let Some(on_found) = on_found.take()
            {
                on_found();
            }
        },
        None,
    )?;

    // Succeed if the interfaces were already up
    let mut found_interfaces = found_interfaces.lock().unwrap();

    if !found_interfaces.ipv4 {
        found_interfaces.ipv4 |= ip_interface_entry_exists(AddressFamily::Ipv4, &luid)?;
    }
    if !found_interfaces.ipv6 {
        found_interfaces.ipv6 |= ip_interface_entry_exists(AddressFamily::Ipv6, &luid)?;
    }

    if found_interfaces.ipv4 && found_interfaces.ipv6 {
        return Ok(StartNotifyResult::AlreadyExist);
    }

    Ok(StartNotifyResult::Waiting(handle))
}

/// Returns the current state of the unicast address `address` on the interface `luid`.
pub fn get_unicast_ip_address_entry(
    luid: NET_LUID_LH,
    address: IpAddr,
) -> io::Result<MIB_UNICASTIPADDRESS_ROW> {
    let mut row = MIB_UNICASTIPADDRESS_ROW {
        InterfaceLuid: luid,
        Address: inet_sockaddr_from_socketaddr(SocketAddr::new(address, 0)),
        ..Default::default()
    };

    #[cfg(not(test))]
    use windows_sys::Win32::NetworkManagement::IpHelper::GetUnicastIpAddressEntry;

    #[cfg(test)]
    use tests::fake_get_unicast_ip_address_entry as GetUnicastIpAddressEntry;

    win32_err!(unsafe { GetUnicastIpAddressEntry(&raw mut row) })?;
    Ok(row)
}

/// Returns whether an address in the given DAD state is ready to be used, or an error if
/// duplicate address detection has failed for it.
#[expect(non_upper_case_globals)]
fn address_is_ready(state: NL_DAD_STATE) -> Result<bool> {
    match state {
        IpDadStatePreferred => Ok(true),
        IpDadStateTentative => Ok(false),
        state => Err(Error::DadStateError(DadStateError::from(state))),
    }
}

/// Waits until all of `addresses` have been added to the interface `luid` and are usable, i.e.
/// until duplicate address detection has completed for each of them.
///
/// This blocks for at most `timeout`.
pub fn wait_for_addresses_sync(
    luid: NET_LUID_LH,
    addresses: &[IpAddr],
    timeout: Duration,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    let on_ready = move |result| {
        let _ = tx.send(result);
    };
    match start_wait_for_addresses(luid, addresses, on_ready)? {
        StartNotifyResult::AlreadyExist => Ok(()),
        StartNotifyResult::Waiting(_handle) => {
            let start = Instant::now();
            let result = rx
                .recv_timeout(timeout)
                .unwrap_or(Err(Error::DeviceReadyTimeout));
            log::debug!(
                "Waited {:?} for duplicate address detection: {}",
                start.elapsed(),
                if result.is_ok() { "done" } else { "failed" },
            );
            result
        }
    }
}

/// Waits until all of `addresses` have been added to the interface `luid` and are usable, i.e.
/// until duplicate address detection has completed for each of them.
///
/// This waits for at most [`DAD_CHECK_TIMEOUT`].
pub async fn wait_for_addresses(luid: NET_LUID_LH, addresses: Vec<IpAddr>) -> Result<()> {
    let (tx, rx) = futures::channel::oneshot::channel();
    // The wait blocks on a notification, so it is moved off of the async runtime.
    std::thread::spawn(move || {
        let _ = tx.send(wait_for_addresses_sync(luid, &addresses, DAD_CHECK_TIMEOUT));
    });
    rx.await.map_err(|_| Error::UnicastSenderDropped)?
}

/// Begins to wait until all of `addresses` have been added to the interface `luid` and have
/// completed duplicate address detection.
///
/// `StartNotifyResult::AlreadyExist` is returned if all of them are already preferred.
///
/// Otherwise, on success, `on_ready` is called when the remaining addresses have been added and
/// become preferred, or as soon as duplicate address detection fails for one of them.
/// The wait is cancelled if the returned handle is dropped.
fn start_wait_for_addresses(
    luid: NET_LUID_LH,
    addresses: &[IpAddr],
    on_ready: impl FnOnce(Result<()>) + Send + 'static,
) -> Result<StartNotifyResult<MIB_UNICASTIPADDRESS_ROW>> {
    /// Why an address is not usable yet.
    #[derive(Debug)]
    enum Pending {
        /// The address has not been added to the interface yet.
        Missing(IpAddr),
        /// The address exists, but duplicate address detection has not completed.
        Tentative(IpAddr),
    }

    impl Pending {
        const fn address(&self) -> &IpAddr {
            let (Pending::Missing(address) | Pending::Tentative(address)) = self;
            address
        }
    }

    // Addresses that are not usable yet. Both missing and tentative addresses are waited for, so
    // that an address that has not been plumbed at all is not mistaken for a ready one.
    let pending: Arc<Mutex<Vec<Pending>>> = Arc::new(Mutex::new(vec![]));

    let mut on_ready = Some(on_ready);
    let pending2 = pending.clone();
    let expected = addresses.to_vec();

    let handle = notify_unicast_ip_address_change(
        move |row, notification_type| {
            // SAFETY: This is always valid as a `u64`.
            if unsafe { row.InterfaceLuid.Value != luid.Value } {
                return;
            }
            let Ok(address) = try_socketaddr_from_inet_sockaddr(row.Address) else {
                return;
            };
            let address = address.ip();

            if notification_type == MibAddInstance && !expected.contains(&address) {
                log::debug!("Unexpected address added to the tunnel interface: {address}");
                return;
            }

            let mut pending = pending2.lock().unwrap();
            if !pending.iter().any(|pending| pending.address() == &address) {
                return;
            }

            // The row handed to the callback contains incomplete data, so the current state of the
            // address has to be queried separately.
            let ready = match get_unicast_ip_address_entry(luid, address) {
                Ok(row) => address_is_ready(row.DadState),
                // The address is not there (yet). Keep waiting for it to be added.
                Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => return,
                Err(error) => Err(Error::ObtainUnicastAddress(error)),
            };

            let result = match ready {
                Ok(true) => {
                    pending.retain(|pending| pending.address() != &address);
                    if !pending.is_empty() {
                        return;
                    }
                    Ok(())
                }
                // Added, but still tentative. Keep waiting.
                Ok(false) => return,
                // Duplicate address detection failed. Give up immediately.
                Err(error) => Err(error),
            };

            if let Some(on_ready) = on_ready.take() {
                on_ready(result);
            }
        },
        None,
    )
    .map_err(Error::StartUnicastAddressNotify)?;

    // Check the current state while holding the lock, so that the callback cannot observe an empty
    // set of pending addresses and discard a change that we have not seen yet.
    //
    // NOTE: This guard must be declared after `handle`, so that it is released first on the early
    // returns below. Dropping `handle` waits for any in-flight callback, which may be blocked on
    // this very lock.
    let mut pending = pending.lock().unwrap();

    for &address in addresses {
        match get_unicast_ip_address_entry(luid, address) {
            Ok(row) => {
                if !address_is_ready(row.DadState)? {
                    pending.push(Pending::Tentative(address));
                }
            }
            Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => {
                pending.push(Pending::Missing(address));
            }
            Err(error) => return Err(Error::ObtainUnicastAddress(error)),
        }
    }

    if pending.is_empty() {
        return Ok(StartNotifyResult::AlreadyExist);
    }

    log::debug!(
        "Waiting for tunnel addresses to become usable: {:?}",
        *pending
    );

    Ok(StartNotifyResult::Waiting(handle))
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

    use windows_sys::Win32::NetworkManagement::IpHelper::{
        PIPINTERFACE_CHANGE_CALLBACK, PUNICAST_IPADDRESS_CHANGE_CALLBACK,
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
        get_ipv4_result: u32,
        get_ipv6_result: u32,
    }

    impl Default for NotifySettings {
        fn default() -> Self {
            NotifySettings {
                expected_luid: NET_LUID_LH { Value: 1 },
                send_add_event_for_families: vec![AF_INET, AF_INET6],
                sleep_duration: None,
                get_ipv4_result: ERROR_NOT_FOUND,
                get_ipv6_result: ERROR_NOT_FOUND,
            }
        }
    }

    static NOTIFY_SETTINGS: LazyLock<Mutex<NotifySettings>> =
        LazyLock::new(|| Mutex::new(NotifySettings::default()));

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

    struct AddressSettings {
        expected_luid: NET_LUID_LH,
        /// DAD state reported by [`fake_get_unicast_ip_address_entry`] for each address.
        /// Addresses that are absent here are reported as `ERROR_NOT_FOUND`.
        dad_states: Vec<(IpAddr, NL_DAD_STATE)>,
        /// Replaces `dad_states` just before notifications are delivered. This models addresses
        /// that are plumbed after the wait has already begun.
        dad_states_on_notify: Option<Vec<(IpAddr, NL_DAD_STATE)>>,
        /// Addresses to deliver `MibAddInstance` notifications for.
        send_add_event_for_addresses: Vec<IpAddr>,
        sleep_duration: Option<Duration>,
    }

    impl Default for AddressSettings {
        fn default() -> Self {
            AddressSettings {
                expected_luid: NET_LUID_LH { Value: 1 },
                dad_states: vec![],
                dad_states_on_notify: None,
                send_add_event_for_addresses: vec![],
                sleep_duration: None,
            }
        }
    }

    static ADDRESS_SETTINGS: LazyLock<Mutex<AddressSettings>> =
        LazyLock::new(|| Mutex::new(AddressSettings::default()));

    pub unsafe fn fake_notify_unicast_ip_address_change(
        family: u16,
        callback: PUNICAST_IPADDRESS_CHANGE_CALLBACK,
        callercontext: *const core::ffi::c_void,
        initialnotification: bool,
        notificationhandle: *mut HANDLE,
    ) -> WIN32_ERROR {
        assert_eq!(family, AF_UNSPEC);
        assert!(!initialnotification);

        struct Context {
            callback: PUNICAST_IPADDRESS_CHANGE_CALLBACK,
            callercontext: *const core::ffi::c_void,
        }
        unsafe impl Send for Context {}
        let ctx = Context {
            callback,
            callercontext,
        };

        let thread = std::thread::spawn(move || {
            let ctx = ctx;

            if let Some(duration) = ADDRESS_SETTINGS.lock().unwrap().sleep_duration {
                std::thread::sleep(duration);
            }

            let cb = ctx.callback.unwrap();

            // Do not hold the lock while invoking the callback: it queries the address state.
            let (luid, addresses) = {
                let mut settings = ADDRESS_SETTINGS.lock().unwrap();
                if let Some(dad_states) = settings.dad_states_on_notify.take() {
                    settings.dad_states = dad_states;
                }
                (
                    settings.expected_luid,
                    settings.send_add_event_for_addresses.clone(),
                )
            };

            for address in addresses {
                // Only the address and LUID are populated, mirroring the incomplete row that
                // Windows hands to the callback.
                let row = MIB_UNICASTIPADDRESS_ROW {
                    InterfaceLuid: luid,
                    Address: inet_sockaddr_from_socketaddr(SocketAddr::new(address, 0)),
                    ..MIB_UNICASTIPADDRESS_ROW::default()
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

    pub unsafe fn fake_get_unicast_ip_address_entry(
        row: *mut MIB_UNICASTIPADDRESS_ROW,
    ) -> WIN32_ERROR {
        // SAFETY: The caller passes an initialized row containing an address and LUID.
        let address = try_socketaddr_from_inet_sockaddr(unsafe { (*row).Address })
            .unwrap()
            .ip();

        let settings = ADDRESS_SETTINGS.lock().unwrap();
        match settings
            .dad_states
            .iter()
            .find(|(candidate, _)| candidate == &address)
        {
            Some(&(_, dad_state)) => {
                // SAFETY: See above.
                unsafe { (*row).DadState = dad_state };
                0
            }
            None => ERROR_NOT_FOUND,
        }
    }

    pub unsafe fn fake_cancel_mib_change_notify2(notificationhandle: HANDLE) -> WIN32_ERROR {
        // Block until thread exits.
        // SAFETY: Constructed once using `Box::into_raw` above.
        let h: Box<NotifyHandle> =
            unsafe { Box::from_raw(notificationhandle as *mut NotifyHandle) };
        h.handle.join().unwrap();
        0
    }

    pub unsafe fn fake_get_ip_interface_entry(row: *mut MIB_IPINTERFACE_ROW) -> WIN32_ERROR {
        let settings = &*NOTIFY_SETTINGS.lock().unwrap();
        match unsafe { (*row).Family } {
            AF_INET => settings.get_ipv4_result,
            AF_INET6 => settings.get_ipv6_result,
            _ => unreachable!(),
        }
    }

    // Serialize and reset `NOTIFY_SETTINGS` since it is globally shared between tests.
    static NOTIFY_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

        // IPv4 interface already exists
        {
            let mut settings = NOTIFY_SETTINGS.lock().unwrap();
            *settings = NotifySettings::default();
            settings.send_add_event_for_families = vec![AF_INET6];
            settings.get_ipv4_result = 0;
        }

        let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;
        wait_for_interfaces_sync(luid, true, true, Duration::from_secs(1)).unwrap();

        // IPv6 interface already exists
        {
            let mut settings = NOTIFY_SETTINGS.lock().unwrap();
            *settings = NotifySettings::default();
            settings.send_add_event_for_families = vec![AF_INET];
            settings.get_ipv6_result = 0;
        }

        let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;
        wait_for_interfaces_sync(luid, true, true, Duration::from_secs(1)).unwrap();

        // Both interfaces exist
        {
            let mut settings = NOTIFY_SETTINGS.lock().unwrap();
            *settings = NotifySettings::default();
            settings.send_add_event_for_families = vec![];
            settings.get_ipv4_result = 0;
            settings.get_ipv6_result = 0;
        }

        let luid = NOTIFY_SETTINGS.lock().unwrap().expected_luid;
        wait_for_interfaces_sync(luid, true, true, Duration::from_secs(1)).unwrap();
    }

    const ADDR_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 64, 0, 2));
    const ADDR_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 2));

    /// Test [`wait_for_addresses_sync`] using mocked notifications.
    // This can be tested with miri:
    //
    // ```rust
    // cargo +nightly miri test -p talpid-windows -- test_wait_for_addresses_sync
    // ```
    #[test]
    fn test_wait_for_addresses_sync() {
        let _guard = NOTIFY_LOCK.blocking_lock();

        let luid = ADDRESS_SETTINGS.lock().unwrap().expected_luid;

        // The addresses are already preferred
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
            settings.dad_states = vec![
                (ADDR_V4, IpDadStatePreferred),
                (ADDR_V6, IpDadStatePreferred),
            ];
        }
        wait_for_addresses_sync(luid, &[ADDR_V4, ADDR_V6], Duration::from_secs(1)).unwrap();

        // The addresses have not been added yet, and show up preferred later on
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
            settings.sleep_duration = Some(Duration::from_millis(10));
            settings.dad_states_on_notify = Some(vec![
                (ADDR_V4, IpDadStatePreferred),
                (ADDR_V6, IpDadStatePreferred),
            ]);
            settings.send_add_event_for_addresses = vec![ADDR_V4, ADDR_V6];
        }
        wait_for_addresses_sync(luid, &[ADDR_V4, ADDR_V6], Duration::from_secs(1)).unwrap();

        // The address is added, but remains tentative
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
            settings.dad_states_on_notify = Some(vec![(ADDR_V4, IpDadStateTentative)]);
            settings.send_add_event_for_addresses = vec![ADDR_V4];
        }
        wait_for_addresses_sync(luid, &[ADDR_V4], Duration::from_millis(50)).unwrap_err();

        // Only one of the two addresses is ever added
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
            settings.dad_states_on_notify = Some(vec![(ADDR_V4, IpDadStatePreferred)]);
            settings.send_add_event_for_addresses = vec![ADDR_V4];
        }
        wait_for_addresses_sync(luid, &[ADDR_V4, ADDR_V6], Duration::from_millis(50)).unwrap_err();

        // Duplicate address detection fails
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
            settings.dad_states_on_notify = Some(vec![(ADDR_V4, IpDadStateDuplicate)]);
            settings.send_add_event_for_addresses = vec![ADDR_V4];
        }
        wait_for_addresses_sync(luid, &[ADDR_V4], Duration::from_secs(1)).unwrap_err();

        // The address is never added at all
        {
            let mut settings = ADDRESS_SETTINGS.lock().unwrap();
            *settings = AddressSettings::default();
        }
        wait_for_addresses_sync(luid, &[ADDR_V4], Duration::from_millis(10)).unwrap_err();

        // Waiting for no addresses at all succeeds immediately
        wait_for_addresses_sync(luid, &[], Duration::from_millis(1)).unwrap();
    }
}
