use crate::dns::DnsMonitorT;
use std::{io, net::IpAddr};
use talpid_types::ErrorExt;
use talpid_windows_net::{guid_from_luid, luid_from_alias};
use windows_sys::{core::GUID, Win32::System::Com::StringFromGUID2};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE},
    transaction::Transaction,
    RegKey,
};

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

    /// Failure to flush DNS cache.
    #[error(display = "Failed to flush DNS resolver cache")]
    FlushResolverCacheError(#[error(source)] super::dnsapi::Error),

    /// Failed to update DNS servers for interface.
    #[error(display = "Failed to update interface DNS servers")]
    SetResolversError(#[error(source)] io::Error),
}

pub struct DnsMonitor {
    current_guid: Option<GUID>,
}

impl DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        Ok(DnsMonitor { current_guid: None })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        let guid = guid_from_luid(&luid_from_alias(interface).map_err(Error::InterfaceLuidError)?)
            .map_err(Error::InterfaceGuidError)?;
        set_dns(&guid, servers)?;
        self.current_guid = Some(guid);
        flush_dns_cache()?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        if let Some(guid) = self.current_guid.take() {
            return set_dns(&guid, &[]).and(flush_dns_cache());
        }
        Ok(())
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
    super::dnsapi::flush_resolver_cache().map_err(Error::FlushResolverCacheError)
}

/// Obtain a string representation for a GUID object.
fn string_from_guid(guid: &GUID) -> String {
    let mut buffer = [0u16; 40];
    let length = unsafe { StringFromGUID2(guid, &mut buffer[0] as *mut _, buffer.len() as i32 - 1) }
        as usize;
    // cannot fail because `buffer` is large enough
    assert!(length > 0);
    let length = length - 1;
    String::from_utf16(&buffer[0..length]).unwrap()
}
