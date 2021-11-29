use crate::logging::windows::{log_sink, LogSink};

use lazy_static::lazy_static;
use log::{error, trace, warn};
use std::{env, io, net::IpAddr, path::Path};
use talpid_types::ErrorExt;
use widestring::WideCString;
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, REG_MULTI_SZ},
    transaction::Transaction,
    RegKey, RegValue,
};

const DNS_CACHE_POLICY_GUID: &str = "{d57d2750-f971-408e-8e55-cfddb37e60ae}";

lazy_static! {
    /// Specifies whether to override per-interface DNS resolvers with a global DNS policy.
    static ref GLOBAL_DNS_CACHE_POLICY: bool = env::var("TALPID_DNS_CACHE_POLICY")
        .map(|v| v != "0")
        .unwrap_or(true);
}

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to initialize WinDns.
    #[error(display = "Failed to initialize WinDns")]
    Initialization,

    /// Failure to deinitialize WinDns.
    #[error(display = "Failed to deinitialize WinDns")]
    Deinitialization,

    /// Failure to set new DNS servers on the interface.
    #[error(display = "Failed to set new DNS servers on interface")]
    Setting,

    /// Failure to set new DNS servers.
    #[error(display = "Failed to update dnscache policy config")]
    UpdateDnsCachePolicy(#[error(source)] io::Error),
}

pub struct DnsMonitor {}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        unsafe { WinDns_Initialize(Some(log_sink), b"WinDns\0".as_ptr()).into_result()? };

        let mut monitor = DnsMonitor {};
        monitor.reset()?;

        Ok(monitor)
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
        }?;

        if *GLOBAL_DNS_CACHE_POLICY && is_minimum_windows10() {
            if let Err(error) = set_dns_cache_policy(servers) {
                error!("{}", error.display_chain());
                warn!("DNS resolution may be slowed down");
            }
        }

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        if *GLOBAL_DNS_CACHE_POLICY && is_minimum_windows10() {
            reset_dns_cache_policy()
        } else {
            Ok(())
        }
    }
}

fn ip_to_widestring(ip: &IpAddr) -> WideCString {
    WideCString::new(ip.to_string().encode_utf16().collect::<Vec<_>>()).unwrap()
}

impl Drop for DnsMonitor {
    fn drop(&mut self) {
        if *GLOBAL_DNS_CACHE_POLICY && is_minimum_windows10() {
            if let Err(error) = reset_dns_cache_policy() {
                warn!(
                    "{}",
                    error.display_chain_with_msg("Failed to reset DNS cache policy")
                );
            }
        }

        if unsafe { WinDns_Deinitialize().into_result().is_ok() } {
            trace!("Successfully deinitialized WinDns");
        } else {
            error!("Failed to deinitialize WinDns");
        }
    }
}

fn set_dns_cache_policy(servers: &[IpAddr]) -> Result<(), Error> {
    let transaction = Transaction::new()?;
    match set_dns_cache_policy_inner(&transaction, servers) {
        Ok(()) => {
            transaction.commit()?;
            Ok(())
        }
        Err(error) => {
            transaction.rollback()?;
            Err(error)
        }
    }
}

fn set_dns_cache_policy_inner(transaction: &Transaction, servers: &[IpAddr]) -> Result<(), Error> {
    let (dns_cache_parameters, _) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey_transacted(
        r#"SYSTEM\CurrentControlSet\Services\DnsCache\Parameters"#,
        transaction,
    )?;

    // Fall back on LLMNR and NetBIOS if DNS resolution fails
    dns_cache_parameters.set_value("DnsSecureNameQueryFallback", &1u32)?;

    let policy_path = Path::new("DnsPolicyConfig").join(DNS_CACHE_POLICY_GUID);
    let (policy_config, _) =
        dns_cache_parameters.create_subkey_transacted(policy_path, transaction)?;

    // Enable only the "Generic DNS server" option
    policy_config.set_value("ConfigOptions", &0x08u32)?;
    let server_list: Vec<String> = servers.iter().map(|server| server.to_string()).collect();
    policy_config.set_value("GenericDNSServers", &server_list.join(";"))?;
    policy_config.set_value("IPSECCARestriction", &"")?;
    policy_config.set_raw_value(
        "Name",
        &RegValue {
            // utf16 string: ".\0\0"
            bytes: [0x2e, 0, 0, 0, 0, 0].to_vec(),
            vtype: REG_MULTI_SZ,
        },
    )?;
    policy_config.set_value("Version", &2u32)?;

    Ok(())
}

fn reset_dns_cache_policy() -> Result<(), Error> {
    let (dns_cache_parameters, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .create_subkey(r#"SYSTEM\CurrentControlSet\Services\DnsCache\Parameters"#)?;
    match dns_cache_parameters.delete_value("DnsSecureNameQueryFallback") {
        Ok(()) => Ok(()),
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(Error::UpdateDnsCachePolicy(error))
            }
        }
    }?;
    let policy_path = Path::new("DnsPolicyConfig").join(DNS_CACHE_POLICY_GUID);
    match dns_cache_parameters.delete_subkey_all(policy_path) {
        Ok(()) => Ok(()),
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(Error::UpdateDnsCachePolicy(error))
            }
        }
    }
}

fn is_minimum_windows10() -> bool {
    match talpid_platform_metadata::WindowsVersion::new() {
        Ok(version_info) => version_info.major_version() >= 10,
        Err(error) => {
            error!(
                "{}",
                error.display_chain_with_msg("OS version check failed")
            );
            false
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
