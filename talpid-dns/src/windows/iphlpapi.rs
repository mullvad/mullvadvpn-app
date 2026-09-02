//! DNS monitor that uses `SetInterfaceDnsSettings`.
//!
//! `SetInterfaceDnsSettings` is available on all Windows versions that Mullvad supports. See `supported-platforms.md`.

use super::{DnsMonitorT, ResolvedDnsConfig};
use std::{
    ffi::OsString,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::ffi::OsStrExt,
    ptr,
};
use talpid_types::win32_err;
use talpid_windows::net::{guid_from_luid, luid_from_alias};
use windows_sys::{
    Win32::NetworkManagement::IpHelper::{
        DNS_INTERFACE_SETTINGS, DNS_INTERFACE_SETTINGS_VERSION1, DNS_SETTING_IPV6,
        DNS_SETTING_NAMESERVER, SetInterfaceDnsSettings,
    },
    core::GUID,
};

/// Errors that can happen when configuring DNS on Windows.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to obtain an interface LUID given an alias.
    #[error("Failed to obtain LUID for the interface alias")]
    ObtainInterfaceLuid(#[source] io::Error),

    /// Failure to obtain an interface GUID.
    #[error("Failed to obtain GUID for the interface")]
    ObtainInterfaceGuid(#[source] io::Error),

    /// Failed to set DNS settings on interface.
    #[error("Failed to set DNS settings on interface")]
    SetInterfaceDnsSettings(#[source] io::Error),

    /// Failure to flush DNS cache.
    #[error("Failed to flush DNS resolver cache")]
    FlushResolverCache(#[source] super::dnsapi::Error),
}

pub struct DnsMonitor {
    current_guid: Option<GUID>,
}

impl DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        Ok(DnsMonitor { current_guid: None })
    }

    fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<(), Error> {
        let servers = config.tunnel_config();
        let guid = guid_from_luid(&luid_from_alias(interface).map_err(Error::ObtainInterfaceLuid)?)
            .map_err(Error::ObtainInterfaceGuid)?;

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

        flush_dns_cache()?;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        let Some(guid) = self.current_guid.take() else {
            return Ok(());
        };
        set_interface_dns_servers_v4(&guid, &[]).and(set_interface_dns_servers_v6(&guid, &[]))
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        // do nothing since the tunnel interface goes away
        let _ = self.current_guid.take();
        Ok(())
    }
}

fn set_interface_dns_servers_v4(guid: &GUID, servers: &[&Ipv4Addr]) -> Result<(), Error> {
    set_interface_dns_servers(guid, servers, ConfigureNameServer::Ipv4)
}

fn set_interface_dns_servers_v6(guid: &GUID, servers: &[&Ipv6Addr]) -> Result<(), Error> {
    set_interface_dns_servers(guid, servers, ConfigureNameServer::Ipv6)
}

fn set_interface_dns_servers<T: ToString>(
    guid: &GUID,
    servers: &[T],
    flags: ConfigureNameServer,
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
        Flags: flags as u64,
        Domain: ptr::null_mut(),
        NameServer: nameservers.as_mut_ptr(),
        SearchList: ptr::null_mut(),
        RegistrationEnabled: 0,
        RegisterAdapterName: 0,
        EnableLLMNR: 0,
        QueryAdapterName: 0,
        ProfileNameServer: ptr::null_mut(),
    };

    // SAFETY:
    // - `&raw const dns_interface_settings` is a valid pointer to a DNS_INTERFACE_SETTINGS-type structure.
    // - `dns_interface_settings` is live for the entire call to SetInterfaceDnsSettings.
    // - DNS_INTERFACE_SETTINGS::Version member is set to DNS_INTERFACE_SETTINGS_VERSION1.
    // - DNS_INTERFACE_SETTINGS.NameServer member is populated while the other members are zeroed.
    win32_err!(unsafe {
        // <https://learn.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-setinterfacednssettings>.
        SetInterfaceDnsSettings(guid.to_owned(), &raw const dns_interface_settings)
    })
    .map_err(Error::SetInterfaceDnsSettings)
}

/// DNS_INTERFACE_SETTINGS.Flags argument for configures static adapter DNS servers on the specified interface via the NameServer member.
///
/// See <https://learn.microsoft.com/en-us/windows/win32/api/netioapi/ns-netioapi-dns_interface_settings>.
#[repr(u32)]
enum ConfigureNameServer {
    /// `DNS_SETTING_NAMESERVER`.
    Ipv4 = DNS_SETTING_NAMESERVER,
    /// `DNS_SETTING_NAMESERVER | DNS_SETTING_IPV6`.
    Ipv6 = (DNS_SETTING_NAMESERVER | DNS_SETTING_IPV6),
}

fn flush_dns_cache() -> Result<(), Error> {
    super::dnsapi::flush_resolver_cache().map_err(Error::FlushResolverCache)
}
