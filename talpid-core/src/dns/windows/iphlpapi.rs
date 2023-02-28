use crate::dns::DnsMonitorT;
use std::{
    ffi::OsString,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::ffi::OsStrExt,
    ptr,
};
use talpid_windows_net::{guid_from_luid, luid_from_alias};
use windows_sys::{
    core::GUID,
    Win32::{
        Foundation::NO_ERROR,
        NetworkManagement::IpHelper::{
            SetInterfaceDnsSettings, DNS_INTERFACE_SETTINGS, DNS_INTERFACE_SETTINGS_VERSION1,
            DNS_SETTING_IPV6, DNS_SETTING_NAMESERVER,
        },
    },
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

    /// Failed to set DNS settings on interface.
    #[error(display = "Failed to set DNS settings on interface: {}", _0)]
    SetInterfaceDnsSettings(i32),
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


        let mut v4_servers = vec![];
        let mut v6_servers = vec![];

        for server in servers {
            match server {
                IpAddr::V4(addr) => v4_servers.push(addr),
                IpAddr::V6(addr) => v6_servers.push(addr),
            }
        }

        self.current_guid = Some(guid);

        if !v4_servers.is_empty() {
            set_interface_dns_servers_v4(&guid, &v4_servers)?;
        }
        if !v6_servers.is_empty() {
            set_interface_dns_servers_v6(&guid, &v6_servers)?;
        }

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        if let Some(guid) = self.current_guid.take() {
            let _ = set_interface_dns_servers_v4(&guid, &[]);
            let _ = set_interface_dns_servers_v6(&guid, &[]);
        }
        Ok(())
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        // do nothing since the tunnel interface goes away
        let _ = self.current_guid.take();
        Ok(())
    }
}

fn set_interface_dns_servers_v4(guid: &GUID, servers: &[&Ipv4Addr]) -> Result<(), Error> {
    set_interface_dns_servers(guid, servers, DNS_SETTING_NAMESERVER)
}

fn set_interface_dns_servers_v6(guid: &GUID, servers: &[&Ipv6Addr]) -> Result<(), Error> {
    set_interface_dns_servers(guid, servers, DNS_SETTING_NAMESERVER | DNS_SETTING_IPV6)
}

fn set_interface_dns_servers<T: ToString>(
    guid: &GUID,
    servers: &[T],
    flags: u32,
) -> Result<(), Error> {
    // Create comma-separated nameserver list
    let nameservers = servers
        .iter()
        .map(|addr| addr.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let mut nameservers: Vec<u16> = OsString::from(nameservers)
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    let dns_interface_settings = DNS_INTERFACE_SETTINGS {
        Version: DNS_INTERFACE_SETTINGS_VERSION1,
        Flags: u64::from(flags),
        Domain: ptr::null_mut(),
        NameServer: nameservers.as_mut_ptr(),
        SearchList: ptr::null_mut(),
        RegistrationEnabled: 0,
        RegisterAdapterName: 0,
        EnableLLMNR: 0,
        QueryAdapterName: 0,
        ProfileNameServer: ptr::null_mut(),
    };

    let result = unsafe { SetInterfaceDnsSettings(guid.to_owned(), &dns_interface_settings) };
    if result != (NO_ERROR as i32) {
        return Err(Error::SetInterfaceDnsSettings(result));
    }
    Ok(())
}
