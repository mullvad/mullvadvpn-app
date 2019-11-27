use crate::logging::windows::{log_sink, LogSink};

use log::{error, trace};
use std::{net::IpAddr, path::Path};
use widestring::WideCString;

mod system_state;
use self::system_state::SystemStateWriter;


const DNS_STATE_FILENAME: &'static str = "dns-state-backup";

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to initialize WinDns.
    #[error(display = "Failed to initialize WinDns")]
    Initialization,

    /// Failure to deinitialize WinDns.
    #[error(display = "Failed to deinitialize WinDns")]
    Deinitialization,

    /// Failure to set new DNS servers.
    #[error(display = "Failed to set new DNS servers")]
    Setting,
}

pub struct DnsMonitor {}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Error> {
        unsafe { WinDns_Initialize(Some(log_sink), b"WinDns\0".as_ptr()).into_result()? };

        let backup_writer = SystemStateWriter::new(
            cache_dir
                .as_ref()
                .join(DNS_STATE_FILENAME)
                .into_boxed_path(),
        );
        let _ = backup_writer.remove_backup();
        Ok(DnsMonitor {})
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
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
                WideCString::from_str(interface).unwrap().as_ptr(),
                ipv4_address_ptrs.as_mut_ptr(),
                ipv4_address_ptrs.len() as u32,
                ipv6_address_ptrs.as_mut_ptr(),
                ipv6_address_ptrs.len() as u32,
            )
            .into_result()
        }
    }

    fn reset(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

fn ip_to_widestring(ip: &IpAddr) -> WideCString {
    WideCString::new(ip.to_string().encode_utf16().collect::<Vec<_>>()).unwrap()
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


ffi_error!(InitializationResult, Error::Initialization);
ffi_error!(DeinitializationResult, Error::Deinitialization);
ffi_error!(SettingResult, Error::Setting);


#[allow(non_snake_case)]
extern "stdcall" {

    #[link_name = "WinDns_Initialize"]
    pub fn WinDns_Initialize(
        sink: Option<LogSink>,
        sink_context: *const u8,
    ) -> InitializationResult;

    // WinDns_Deinitialize:
    //
    // Call this function once before unloading WINDNS or exiting the process.
    #[link_name = "WinDns_Deinitialize"]
    pub fn WinDns_Deinitialize() -> DeinitializationResult;

    // Configure which DNS servers should be used and start enforcing these settings.
    #[link_name = "WinDns_Set"]
    pub fn WinDns_Set(
        interface_alias: *const u16,
        v4_ips: *mut *const u16,
        v4_n_ips: u32,
        v6_ips: *mut *const u16,
        v6_n_ips: u32,
    ) -> SettingResult;
}
