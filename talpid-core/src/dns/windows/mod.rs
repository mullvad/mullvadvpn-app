use crate::windows::{get_system_dir, guid_from_luid, luid_from_alias, string_from_guid};
use lazy_static::lazy_static;
use std::{env, io, net::IpAddr, path::Path, process::Command};
use talpid_types::ErrorExt;
use winapi::shared::guiddef::GUID;
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_MULTI_SZ},
    transaction::Transaction,
    RegKey, RegValue,
};

const DNS_CACHE_POLICY_GUID: &str = "{d57d2750-f971-408e-8e55-cfddb37e60ae}";

lazy_static! {
    /// Specifies whether to override per-interface DNS resolvers with a global DNS policy.
    static ref GLOBAL_DNS_CACHE_POLICY: bool = env::var("TALPID_DNS_CACHE_POLICY")
        .map(|v| v != "0")
        .unwrap_or(false);
}

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failure to obtain an interface LUID given an alias.
    #[error(display = "Failed to obtain LUID for the interface alias")]
    InterfaceLuidError(#[error(source)] io::Error),

    /// Failure to obtain an interface GUID.
    #[error(display = "Failed to obtain GUID for the interface")]
    InterfaceGuidError(#[error(source)] io::Error),

    /// Failure to set new DNS servers.
    #[error(display = "Failed to update dnscache policy config")]
    UpdateDnsCachePolicy(#[error(source)] io::Error),

    /// Failure to flush DNS cache.
    #[error(display = "Failed to execute ipconfig")]
    ExecuteIpconfigError(#[error(source)] io::Error),

    /// Failure to flush DNS cache.
    #[error(display = "Failed to flush DNS resolver cache")]
    FlushResolverCacheError,

    /// Failed to update DNS servers for interface.
    #[error(display = "Failed to update interface DNS servers")]
    SetResolversError(#[error(source)] io::Error),

    /// Failed to locate system dir.
    #[error(display = "Failed to locate the system directory")]
    SystemDirError(#[error(source)] io::Error),
}

pub struct DnsMonitor {
    current_guid: Option<GUID>,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        let mut monitor = DnsMonitor { current_guid: None };
        monitor.reset()?;

        Ok(monitor)
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        let guid = guid_from_luid(&luid_from_alias(interface).map_err(Error::InterfaceLuidError)?)
            .map_err(Error::InterfaceGuidError)?;
        set_dns(&guid, servers)?;
        self.current_guid = Some(guid);
        flush_dns_cache()?;

        if *GLOBAL_DNS_CACHE_POLICY {
            if let Err(error) = set_dns_cache_policy(servers) {
                log::error!("{}", error.display_chain());
            }
        }

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        let mut result = Ok(());

        if let Some(guid) = self.current_guid.take() {
            result = result.and(set_dns(&guid, &[])).and(flush_dns_cache());
        }

        if *GLOBAL_DNS_CACHE_POLICY {
            result = result.and(reset_dns_cache_policy());
        }

        result
    }
}

impl Drop for DnsMonitor {
    fn drop(&mut self) {
        if *GLOBAL_DNS_CACHE_POLICY {
            if let Err(error) = reset_dns_cache_policy() {
                log::warn!(
                    "{}",
                    error.display_chain_with_msg("Failed to reset DNS cache policy")
                );
            }
        }
    }
}

fn set_dns(interface: &GUID, servers: &[IpAddr]) -> Result<(), Error> {
    let transaction = Transaction::new().map_err(Error::SetResolversError)?;
    let result = match set_dns_inner(&transaction, interface, servers) {
        Ok(()) => transaction.commit(),
        Err(error) => transaction.rollback().and(Err(error)),
    };
    result.map_err(Error::SetResolversError)
}

fn set_dns_inner(
    transaction: &Transaction,
    interface: &GUID,
    servers: &[IpAddr],
) -> io::Result<()> {
    let guid_str = string_from_guid(interface);

    config_interface(
        transaction,
        &guid_str,
        "Tcpip",
        servers.iter().filter(|addr| addr.is_ipv4()),
    )?;

    config_interface(
        transaction,
        &guid_str,
        "Tcpip6",
        servers.iter().filter(|addr| addr.is_ipv6()),
    )?;

    Ok(())
}

fn config_interface<'a>(
    transaction: &Transaction,
    guid: &str,
    service: &str,
    nameservers: impl Iterator<Item = &'a IpAddr>,
) -> io::Result<()> {
    let nameservers = nameservers
        .map(|addr| addr.to_string())
        .collect::<Vec<String>>();

    let reg_path =
        format!(r#"SYSTEM\CurrentControlSet\Services\{service}\Parameters\Interfaces\{guid}"#,);
    let adapter_key = match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_transacted_with_flags(
        reg_path,
        transaction,
        KEY_SET_VALUE,
    ) {
        Ok(adapter_key) => Ok(adapter_key),
        Err(error) => {
            if nameservers.is_empty() && error.kind() == io::ErrorKind::NotFound {
                return Ok(());
            }
            Err(error)
        }
    }?;

    if !nameservers.is_empty() {
        adapter_key.set_value("NameServer", &nameservers.join(","))?;
    } else {
        adapter_key.delete_value("NameServer").or_else(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(error)
            }
        })?;
    }

    // Try to disable LLMNR on the interface
    if let Err(error) = adapter_key.set_value("EnableMulticast", &0u32) {
        log::error!(
            "{}\nService: {service}",
            error.display_chain_with_msg("Failed to disable LLMNR on the tunnel interface")
        );
    }

    Ok(())
}

fn flush_dns_cache() -> Result<(), Error> {
    let sysdir = get_system_dir().map_err(Error::SystemDirError)?;
    let mut ipconfig = Command::new(sysdir.join("ipconfig.exe"));
    ipconfig.arg("/flushdns");
    let output = ipconfig.output().map_err(Error::ExecuteIpconfigError)?;
    let output = String::from_utf8_lossy(&output.stdout);
    // The exit code cannot be trusted
    if !output.contains("Successfully flushed") {
        log::error!("Failed to flush DNS cache: {}", output);
        return Err(Error::FlushResolverCacheError);
    }
    Ok(())
}

fn set_dns_cache_policy(servers: &[IpAddr]) -> Result<(), Error> {
    let transaction = Transaction::new().map_err(Error::UpdateDnsCachePolicy)?;
    let result = match set_dns_cache_policy_inner(&transaction, servers) {
        Ok(()) => transaction.commit(),
        Err(error) => transaction.rollback().and_then(|_| Err(error)),
    };
    result.map_err(Error::UpdateDnsCachePolicy)
}

fn set_dns_cache_policy_inner(transaction: &Transaction, servers: &[IpAddr]) -> io::Result<()> {
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
        .create_subkey(r#"SYSTEM\CurrentControlSet\Services\DnsCache\Parameters"#)
        .map_err(Error::UpdateDnsCachePolicy)?;
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
