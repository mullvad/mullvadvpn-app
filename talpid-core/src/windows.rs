use std::{
    ffi::OsStr,
    fmt, io, mem,
    os::windows::{ffi::OsStrExt, io::RawHandle},
    sync::Mutex,
    time::{Duration, Instant},
};
use winapi::shared::{
    ifdef::NET_LUID,
    netioapi::{
        CancelMibChangeNotify2, ConvertInterfaceAliasToLuid, FreeMibTable, GetIpInterfaceEntry,
        GetUnicastIpAddressEntry, GetUnicastIpAddressTable, MibAddInstance,
        NotifyIpInterfaceChange, SetIpInterfaceEntry, MIB_IPINTERFACE_ROW,
        MIB_UNICASTIPADDRESS_ROW, MIB_UNICASTIPADDRESS_TABLE,
    },
    nldef::{IpDadStatePreferred, IpDadStateTentative, NL_DAD_STATE},
    ntdef::FALSE,
    winerror::{ERROR_NOT_FOUND, NO_ERROR},
    ws2def::{AF_INET, AF_INET6, AF_UNSPEC},
};

/// Result type for this module.
pub type Result<T> = std::result::Result<T, Error>;

const DAD_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const DAD_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Errors returned by some functions in this module.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Error returned from `ConvertInterfaceAliasToLuid`
    #[cfg(windows)]
    #[error(display = "Cannot find LUID for virtual adapter")]
    NoDeviceLuid(#[error(source)] io::Error),

    /// Error returned from `GetUnicastIpAddressTable`/`GetUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error(display = "Failed to obtain unicast IP address table")]
    ObtainUnicastAddress(#[error(source)] io::Error),

    /// `GetUnicastIpAddressTable` contained no addresses for the interface
    #[cfg(windows)]
    #[error(display = "Found no addresses for the given adapter")]
    NoUnicastAddress,

    /// Unexpected DAD state returned for a unicast address
    #[cfg(windows)]
    #[error(display = "Unexpected DAD state")]
    DadStateError(#[error(source)] DadStateError),

    /// DAD check failed.
    #[cfg(windows)]
    #[error(display = "Timed out waiting on tunnel device")]
    DeviceReadyTimeout,

    /// Unicast DAD check fail.
    #[cfg(windows)]
    #[error(display = "Unicast channel sender was unexpectedly dropped")]
    UnicastSenderDropped,
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

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Unknown address family
    #[error(display = "Unknown address family: {}", _0)]
    UnknownAddressFamily(i32),
}

impl AddressFamily {
    pub fn try_from_af_family(family: u16) -> Result<AddressFamily, Error> {
        match family as i32 {
            AF_INET => Ok(AddressFamily::Ipv4),
            AF_INET6 => Ok(AddressFamily::Ipv6),
            family => Err(Error::UnknownAddressFamily(family)),
        }
    }
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
    family: Option<AddressFamily>,
) -> io::Result<Box<IpNotifierHandle<'a>>> {
    let mut context = Box::new(IpNotifierHandle {
        callback: Mutex::new(Box::new(callback)),
        handle: std::ptr::null_mut(),
    });

    let status = unsafe {
        NotifyIpInterfaceChange(
            af_family_from_family(family),
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
pub fn get_ip_interface_entry(
    family: AddressFamily,
    luid: &NET_LUID,
) -> io::Result<MIB_IPINTERFACE_ROW> {
    let mut row: MIB_IPINTERFACE_ROW = unsafe { mem::zeroed() };
    row.Family = family as u16;
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

fn ip_interface_entry_exists(family: AddressFamily, luid: &NET_LUID) -> io::Result<bool> {
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
        None,
    )?;

    // Make sure they don't already exist
    if (!ipv4 || ip_interface_entry_exists(AddressFamily::Ipv4, &luid)?)
        && (!ipv6 || ip_interface_entry_exists(AddressFamily::Ipv6, &luid)?)
    {
        return Ok(());
    }

    let _ = rx.await;
    Ok(())
}

/// Handles cases where there DAD state is neither tentative nor preferred.
#[cfg(windows)]
#[derive(err_derive::Error, Debug)]
pub enum DadStateError {
    /// Invalid DAD state.
    #[error(display = "Invalid DAD state")]
    Invalid,

    /// Duplicate unicast address.
    #[error(display = "A duplicate IP address was detected")]
    Duplicate,

    /// Deprecated unicast address.
    #[error(display = "The IP address has been deprecated")]
    Deprecated,

    /// Unknown DAD state constant.
    #[error(display = "Unknown DAD state: {}", _0)]
    Unknown(u32),
}

#[cfg(windows)]
#[allow(non_upper_case_globals)]
impl From<NL_DAD_STATE> for DadStateError {
    fn from(state: NL_DAD_STATE) -> DadStateError {
        use winapi::shared::nldef::*;
        match state {
            IpDadStateInvalid => DadStateError::Invalid,
            IpDadStateDuplicate => DadStateError::Duplicate,
            IpDadStateDeprecated => DadStateError::Deprecated,
            other => DadStateError::Unknown(other),
        }
    }
}

/// Wait for addresses to be usable on an network adapter.
pub async fn wait_for_addresses(luid: NET_LUID) -> Result<()> {
    // Obtain unicast IP addresses
    let mut unicast_rows: Vec<MIB_UNICASTIPADDRESS_ROW> = get_unicast_table(None)
        .map_err(Error::ObtainUnicastAddress)?
        .into_iter()
        .filter(|row| row.InterfaceLuid.Value == luid.Value)
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
                let status = unsafe { GetUnicastIpAddressEntry(row) };
                if status != NO_ERROR {
                    return Err(Error::ObtainUnicastAddress(io::Error::from_raw_os_error(
                        status as i32,
                    )));
                }
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

/// Returns the unicast IP address table. If `family` is `None`, then addresses for all families are
/// returned.
pub fn get_unicast_table(
    family: Option<AddressFamily>,
) -> io::Result<Vec<MIB_UNICASTIPADDRESS_ROW>> {
    let mut unicast_rows = vec![];
    let mut unicast_table: *mut MIB_UNICASTIPADDRESS_TABLE = std::ptr::null_mut();

    let status =
        unsafe { GetUnicastIpAddressTable(af_family_from_family(family), &mut unicast_table) };
    if status != NO_ERROR {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let first_row = unsafe { &(*unicast_table).Table[0] } as *const MIB_UNICASTIPADDRESS_ROW;
    for i in 0..unsafe { *unicast_table }.NumEntries {
        unicast_rows.push(unsafe { *(first_row.offset(i as isize)) });
    }
    unsafe { FreeMibTable(unicast_table as *mut _) };

    Ok(unicast_rows)
}

/// Returns the LUID of an interface given its alias.
pub fn luid_from_alias<T: AsRef<OsStr>>(alias: T) -> io::Result<NET_LUID> {
    let alias_wide: Vec<u16> = alias
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();
    let mut luid: NET_LUID = unsafe { std::mem::zeroed() };
    let status = unsafe { ConvertInterfaceAliasToLuid(alias_wide.as_ptr(), &mut luid) };
    if status != NO_ERROR {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(luid)
}

fn af_family_from_family(family: Option<AddressFamily>) -> u16 {
    family
        .map(|family| family as u16)
        .unwrap_or(AF_UNSPEC as u16)
}
