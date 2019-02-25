use log::{debug, error, info, trace, warn};
use std::{
    borrow::Borrow,
    net::IpAddr,
    os::raw::{c_char, c_void},
    path::Path,
    ptr, slice,
};

mod system_state;
use self::system_state::SystemStateWriter;

use error_chain::ChainedError;
use widestring::WideCString;


const DNS_STATE_FILENAME: &'static str = "dns-state-backup";

error_chain! {
    errors{
        /// Failure to initialize WinDns
        Initialization{
            description("Failed to initialize WinDns")
        }

        /// Failure to deinitialize WinDns
        Deinitialization{
            description("Failed to deinitialize WinDns")
        }

        /// Failure to set new DNS servers
        Setting{
            description("Failed to set new DNS servers")
        }

        /// Failure to reset DNS settings
        Resetting{
            description("Failed to reset DNS")
        }

        /// Failure to reset DNS settings from backup
        Recovery{
            description("Failed to recover to backed up system state")
        }
    }
}

pub struct DnsMonitor {
    backup_writer: SystemStateWriter,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        unsafe { WinDns_Initialize(Some(log_sink), ptr::null_mut()).into_result()? };

        let backup_writer = SystemStateWriter::new(
            cache_dir
                .as_ref()
                .join(DNS_STATE_FILENAME)
                .into_boxed_path(),
        );
        let mut dns = DnsMonitor { backup_writer };
        if let Err(error) = dns
            .restore_system_backup()
            .chain_err(|| "Failed to restore DNS backup")
        {
            error!("{}", error.display_chain());
        }
        Ok(dns)
    }

    fn set(&mut self, _interface: &str, servers: &[IpAddr]) -> Result<()> {
        let ipv4 = servers
            .iter()
            .filter(|ip| ip.is_ipv4())
            .map(ip_to_widestring)
            .collect::<Vec<_>>();
        let ipv6 = servers
            .iter()
            .filter(|ip| ip.is_ipv6())
            .map(ip_to_widestring)
            .collect::<Vec<_>>();


        let mut ipv4_address_ptrs = ipv4
            .iter()
            .map(|ip_cstr| ip_cstr.as_ptr())
            .collect::<Vec<_>>();
        let mut ipv6_address_ptrs = ipv6
            .iter()
            .map(|ip_cstr| ip_cstr.as_ptr())
            .collect::<Vec<_>>();

        trace!("ipv4 ips - {:?} - {}", ipv4, ipv4.len());
        trace!("ipv6 ips - {:?} - {}", ipv6, ipv6.len());

        unsafe {
            WinDns_Set(
                ipv4_address_ptrs.as_mut_ptr(),
                ipv4_address_ptrs.len() as u32,
                ipv6_address_ptrs.as_mut_ptr(),
                ipv6_address_ptrs.len() as u32,
                Some(write_system_state_backup_cb),
                &self.backup_writer as *const _ as *const c_void,
            )
            .into_result()
        }
    }

    fn reset(&mut self) -> Result<()> {
        unsafe { WinDns_Reset().into_result()? };

        if let Err(e) = self.backup_writer.remove_backup() {
            warn!("Failed to remove DNS state backup file: {}", e);
        }
        Ok(())
    }
}

impl DnsMonitor {
    fn restore_dns_settings(&mut self, data: &[u8]) -> Result<()> {
        unsafe { WinDns_Recover(data.as_ptr(), data.len() as u32) }.into_result()
    }

    fn restore_system_backup(&mut self) -> Result<()> {
        if let Some(previous_state) = self
            .backup_writer
            .read_backup()
            .chain_err(|| "Failed to read backed up DNS state")?
        {
            info!("Restoring DNS state from backup");
            if let Err(e) = self.restore_dns_settings(&previous_state) {
                error!("Failed to restore DNS settings - {}", e);
            } else {
                trace!("Successfully restored DNS state");
            };
            self.backup_writer
                .remove_backup()
                .chain_err(|| "Failed to remove backed up DNS state after restoring it")?;
            debug!("DNS recovery file removed!");
        } else {
            trace!("No DNS state to restore");
        }
        Ok(())
    }
}

fn ip_to_widestring(ip: &IpAddr) -> WideCString {
    WideCString::new(ip.to_string().encode_utf16().collect::<Vec<_>>()).unwrap()
}

// typedef void (WINDNS_API *WinDnsErrorSink)(const char *errorMessage, const char **details,
// uint32_t numDetails, void *context);
extern "system" fn log_sink(
    log_level: u8,
    msg: *const c_char,
    detail_ptr: *const *const c_char,
    n_details: u32,
    _ctx: *mut c_void,
) {
    use std::ffi::CStr;
    if msg.is_null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        if detail_ptr.is_null() || n_details == 0 {
            error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
        } else {
            let raw_details = unsafe { slice::from_raw_parts(detail_ptr, n_details as usize) };
            let mut appendix = String::new();
            for detail_ptr in raw_details {
                appendix
                    .push_str(unsafe { CStr::from_ptr(*detail_ptr).to_string_lossy().borrow() });
                appendix.push_str("\n");
            }

            let message = format!(
                "{}: {}",
                unsafe { CStr::from_ptr(msg).to_string_lossy() },
                appendix
            );

            match log_level {
                0x01 => error!("{}", message),
                0x02 => info!("{}", message),
                _ => error!("Unknwon log level - {}", message),
            }
        }
    }
}

impl Drop for DnsMonitor {
    fn drop(&mut self) {
        if unsafe { WinDns_Deinitialize().into_result().is_ok() } {
            trace!("Successfully deinitialized WinDns");
        } else {
            error!("Failed to deinitialize WinDns");
        }
    }
}


ffi_error!(InitializationResult, ErrorKind::Initialization.into());
ffi_error!(DeinitializationResult, ErrorKind::Deinitialization.into());
ffi_error!(SettingResult, ErrorKind::Setting.into());
ffi_error!(ResettingResult, ErrorKind::Resetting.into());
ffi_error!(RecoveringResult, ErrorKind::Recovery.into());


/// A callback for writing system state data
pub extern "system" fn write_system_state_backup_cb(
    blob: *const u8,
    length: u32,
    state_writer_ptr: *mut c_void,
) -> i32 {
    let state_writer = state_writer_ptr as *mut SystemStateWriter;
    if state_writer.is_null() {
        error!("State writer pointer is null, can't save system state backup");
        return -1;
    }

    unsafe {
        trace!(
            "Writing {} bytes to store system state backup to {}",
            length,
            (*state_writer).backup_path.to_string_lossy()
        );
        let data = slice::from_raw_parts(blob, length as usize);
        match (*state_writer).write_backup(data) {
            Ok(()) => 0,
            Err(e) => {
                error!(
                    "Failed to write system state backup to {} because {}",
                    (*state_writer).backup_path.to_string_lossy(),
                    e
                );
                e.raw_os_error().unwrap_or(-1)
            }
        }
    }
}


type DNSConfigSink =
    extern "system" fn(data: *const u8, length: u32, state_writer: *mut c_void) -> i32;

// This callback can be called from multiple threads concurrently, thus if there ever is a real
// context object passed around, it should probably implement Sync.
type ErrorSink = extern "system" fn(
    log_level: u8,
    msg: *const c_char,
    details: *const *const c_char,
    num_details: u32,
    ctx: *mut c_void,
);

#[allow(non_snake_case)]
extern "system" {

    #[link_name = "WinDns_Initialize"]
    pub fn WinDns_Initialize(
        sink: Option<ErrorSink>,
        sink_context: *mut c_void,
    ) -> InitializationResult;

    // WinDns_Deinitialize:
    //
    // Call this function once before unloading WINDNS or exiting the process.
    #[link_name = "WinDns_Deinitialize"]
    pub fn WinDns_Deinitialize() -> DeinitializationResult;

    // Configure which DNS servers should be used and start enforcing these settings.
    #[link_name = "WinDns_Set"]
    pub fn WinDns_Set(
        v4_ips: *mut *const u16,
        v4_n_ips: u32,
        v6_ips: *mut *const u16,
        v6_n_ips: u32,
        callback: Option<DNSConfigSink>,
        backup_writer: *const c_void,
    ) -> SettingResult;

    // Revert server settings to what they were before calling WinDns_Set.
    //
    // (Also taking into account external changes to DNS settings that have ocurred
    // during the period of enforcing specific settings.)
    #[link_name = "WinDns_Reset"]
    pub fn WinDns_Reset() -> ResettingResult;

    #[link_name = "WinDns_Recover"]
    pub fn WinDns_Recover(data: *const u8, length: u32) -> RecoveringResult;
}
