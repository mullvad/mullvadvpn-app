extern crate libc;
extern crate mullvad_paths;
extern crate widestring;

use super::super::system_state::SystemStateWriter;
use super::ffi;

use self::mullvad_paths::cache_dir;
use self::widestring::WideCString;
use std::net::IpAddr;
use std::os::raw::c_void;
use std::ptr;
use std::slice;

const DNS_STATE_FILENAME: &'static str = "dns_state_backup";

error_chain!{
    errors{
        #[doc = "Failure to initialize WinDNS"]
        Initialization{
            description("Failed to initialize WinDNS")
        }

        #[doc = "Failure to deinitialize WinDNS"]
        Deinitialization{
            description("Failed to deinitialize WinDNS")
        }

        #[doc = "Failure to set new DNS servers"]
        Setting{
            description("Failed to set new DNS servers")
        }

        #[doc = "Failure to reset DNS settings"]
        Resetting{
            description("Failed to reset DNS")
        }

        #[doc = "Failure to reset DNS settings from backup"]
        Recovery{
            description("Failed to recover to backed up system state")
        }
    }

    links {
        NoCacheDir(mullvad_paths::Error, mullvad_paths::ErrorKind) #[doc = "Failure to create a cache directory"];
    }

    foreign_links {
        Io(::std::io::Error) #[doc = "IO error, most probably occurs when reading system state backup"];
    }
}

pub struct WinDNS {
    backup_writer: SystemStateWriter,
}

impl WinDNS {
    pub fn new() -> Result<Self> {
        unsafe { WinDns_Initialize(Some(ffi::error_sink), ptr::null_mut()).into_result()? };

        let backup_writer = SystemStateWriter::new(cache_dir()?.join(DNS_STATE_FILENAME));
        let mut dns = WinDNS { backup_writer };
        dns.restore_system_backup()?;
        Ok(dns)
    }

    pub fn set_dns(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        debug!("Setting DNS servers - {:?}", servers);
        let widestring_ips = servers
            .iter()
            .map(|ip| ip.to_string().encode_utf16().collect::<Vec<_>>())
            .map(|ip| WideCString::new(ip).unwrap())
            .collect::<Vec<_>>();

        let mut ip_ptrs = widestring_ips
            .iter()
            .map(|ip_cstr| ip_cstr.as_ptr())
            .collect::<Vec<_>>();

        unsafe {
            WinDns_Set(
                ip_ptrs.as_mut_ptr(),
                widestring_ips.len() as u32,
                Some(write_system_state_backup_cb),
                &self.backup_writer as *const _ as *const c_void,
            ).into_result()
        }
    }

    pub fn reset_dns(&mut self) -> Result<()> {
        trace!("Resetting DNS");
        unsafe { WinDns_Reset().into_result()? };

        if let Err(e) = self.backup_writer.remove_state_file() {
            warn!("Failed to remove DNS state backup file: {}", e);
        }
        Ok(())
    }

    fn restore_dns_settings(&mut self, data: &[u8]) -> Result<()> {
        unsafe { WinDns_Recover(data.as_ptr(), data.len() as u32) }.into_result()
    }

    fn restore_system_backup(&mut self) -> Result<()> {
        if let Some(previous_state) = self.backup_writer.consume_state_backup()? {
            trace!("Restoring system backed up DNS state");
            if let Err(e) = self.restore_dns_settings(&previous_state) {
                self.backup_writer.write_backup(&previous_state)?;
                return Err(e.into());
            }
            trace!("Successfully restored DNS state");
            return Ok(());
        }
        trace!("No dns state to restore");
        Ok(())
    }
}

impl Drop for WinDNS {
    fn drop(&mut self) {
        if unsafe { WinDns_Deinitialize().into_result().is_ok() } {
            trace!("Successfully deinitialized WinDNS");
        } else {
            error!("Failed to deinitialize WinDNS");
        }
    }
}


ffi_error!(init, ErrorKind::Initialization.into());
ffi_error!(deinit, ErrorKind::Deinitialization.into());
ffi_error!(setting, ErrorKind::Setting.into());
ffi_error!(resetting, ErrorKind::Resetting.into());
ffi_error!(recovering, ErrorKind::Recovery.into());


/// A callback for writing system state data
pub extern "system" fn write_system_state_backup_cb(
    blob: *const u8,
    length: u32,
    state_writer: *mut SystemStateWriter,
) -> i32 {
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


#[allow(improper_ctypes)]
type DNSConfigSink =
    extern "system" fn(data: *const u8, length: u32, state_writer: *mut SystemStateWriter) -> i32;

#[allow(non_snake_case, improper_ctypes)]
extern "system" {

    #[link_name(WinDns_Initialize)]
    pub fn WinDns_Initialize(
        sink: Option<ffi::ErrorSink>,
        sink_context: *mut libc::c_void,
    ) -> init::FFIResult;

    // WinDns_Deinitialize:
    //
    // Call this function once before unloading WINDNS or exiting the process.
    #[link_name(WinDns_Deinitialize)]
    pub fn WinDns_Deinitialize() -> deinit::FFIResult;

    // Configure which DNS servers should be used and start enforcing these settings.
    #[link_name(WinDns_Set)]
    pub fn WinDns_Set(
        ips: *mut *const u16,
        n_ips: u32,
        callback: Option<DNSConfigSink>,
        backup_writer: *const c_void,
    ) -> setting::FFIResult;

    // Revert server settings to what they were before calling WinDns_Set.
    //
    // (Also taking into account external changes to DNS settings that have ocurred
    // during the period of enforcing specific settings.)
    #[link_name(WinDns_Reset)]
    pub fn WinDns_Reset() -> resetting::FFIResult;

    #[link_name(WinDns_Recover)]
    pub fn WinDns_Recover(data: *const u8, length: u32) -> recovering::FFIResult;
}
