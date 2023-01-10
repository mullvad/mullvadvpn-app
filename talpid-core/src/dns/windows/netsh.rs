use crate::dns::DnsMonitorT;
use std::{
    ffi::OsString,
    io::{self, Write},
    net::IpAddr,
    os::windows::prelude::{AsRawHandle, OsStringExt},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};
use talpid_types::{net::IpVersion, ErrorExt};
use talpid_windows_net::{index_from_luid, luid_from_alias};
use windows_sys::Win32::{
    Foundation::{MAX_PATH, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::{
        SystemInformation::GetSystemDirectoryW, Threading::WaitForSingleObject,
        WindowsProgramming::INFINITE,
    },
};

const NETSH_TIMEOUT: Duration = Duration::from_secs(10);

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failure to obtain an interface LUID given an alias.
    #[error(display = "Failed to obtain LUID for the interface alias")]
    InterfaceLuidError(#[error(source)] io::Error),

    /// Failure to obtain an interface index.
    #[error(display = "Failed to obtain index of the interface")]
    InterfaceIndexError(#[error(source)] io::Error),

    /// Failure to spawn netsh subprocess.
    #[error(display = "Failed to spawn 'netsh'")]
    SpawnNetsh(#[error(source)] io::Error),

    /// Failure to spawn netsh subprocess.
    #[error(display = "Failed to obtain system directory")]
    GetSystemDir(#[error(source)] io::Error),

    /// Failure to write to stdin.
    #[error(display = "Failed to write to stdin for 'netsh'")]
    NetshInput(#[error(source)] io::Error),

    /// Failure to wait for netsh result.
    #[error(display = "Failed to wait for 'netsh'")]
    WaitNetsh(#[error(source)] io::Error),

    /// netsh returned a non-zero status.
    #[error(display = "'netsh' returned an error: {:?}", _0)]
    NetshError(Option<i32>),

    /// netsh did not return in a timely manner.
    #[error(display = "'netsh' took too long to complete")]
    NetshTimeout,
}

pub struct DnsMonitor {
    current_index: Option<u32>,
}

impl DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        Ok(DnsMonitor {
            current_index: None,
        })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        let interface_luid = luid_from_alias(interface).map_err(Error::InterfaceLuidError)?;
        let interface_index =
            index_from_luid(&interface_luid).map_err(Error::InterfaceIndexError)?;

        self.current_index = Some(interface_index);

        let mut added_ipv4_server = false;
        let mut added_ipv6_server = false;

        let mut netsh_input = String::new();

        for server in servers {
            let is_additional_server;

            if server.is_ipv4() {
                is_additional_server = added_ipv4_server;
                added_ipv4_server = true;
            } else {
                is_additional_server = added_ipv6_server;
                added_ipv6_server = true;
            };

            if is_additional_server {
                netsh_input.push_str(&create_netsh_add_command(interface_index, server));
            } else {
                netsh_input.push_str(&create_netsh_set_command(interface_index, server));
            }
        }

        if !added_ipv4_server {
            netsh_input.push_str(&create_netsh_flush_command(interface_index, IpVersion::V4));
        }
        if !added_ipv6_server {
            netsh_input.push_str(&create_netsh_flush_command(interface_index, IpVersion::V6));
        }

        run_netsh_with_timeout(netsh_input, NETSH_TIMEOUT)?;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        if let Some(index) = self.current_index.take() {
            let mut netsh_input = String::new();
            netsh_input.push_str(&create_netsh_flush_command(index, IpVersion::V4));
            netsh_input.push_str(&create_netsh_flush_command(index, IpVersion::V6));

            if let Err(error) = run_netsh_with_timeout(netsh_input, NETSH_TIMEOUT) {
                log::error!("{}", error.display_chain_with_msg("Failed to reset DNS"));
            }
        }
        Ok(())
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        // do nothing since the tunnel interface goes away
        let _ = self.current_index.take();
        Ok(())
    }
}

fn run_netsh_with_timeout(netsh_input: String, timeout: Duration) -> Result<(), Error> {
    log::debug!("running netsh:\n{}", netsh_input);

    let sysdir = get_system_dir().map_err(Error::GetSystemDir)?;
    let mut netsh = Command::new(sysdir.join(r"netsh.exe"));

    let mut subproc = netsh
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::SpawnNetsh)?;

    let mut stdin = subproc.stdin.take().unwrap();
    stdin
        .write_all(netsh_input.as_bytes())
        .map_err(Error::NetshInput)?;
    drop(stdin);

    match wait_for_child(&mut subproc, timeout) {
        Ok(Some(status)) => {
            if !status.success() {
                return Err(Error::NetshError(status.code()));
            }
            Ok(())
        }
        Ok(None) => {
            let _ = subproc.kill();
            Err(Error::NetshTimeout)
        }
        Err(error) => Err(Error::WaitNetsh(error)),
    }
}

fn wait_for_child(subproc: &mut Child, timeout: Duration) -> io::Result<Option<ExitStatus>> {
    let dur_millis = u32::try_from(timeout.as_millis()).unwrap_or(INFINITE);

    let subproc_handle = subproc.as_raw_handle();
    match unsafe { WaitForSingleObject(subproc_handle as isize, dur_millis) } {
        WAIT_OBJECT_0 => subproc.try_wait(),
        WAIT_TIMEOUT => Ok(None),
        _error => Err(io::Error::last_os_error()),
    }
}

fn create_netsh_set_command(interface_index: u32, server: &IpAddr) -> String {
    // Set primary DNS server:
    // netsh interface ipv4 set dnsservers name="Mullvad" source=static address=10.64.0.1
    // validate=no

    let interface_type = if server.is_ipv4() { "ipv4" } else { "ipv6" };
    format!("interface {interface_type} set dnsservers name={interface_index} source=static address={server} validate=no\r\n")
}

fn create_netsh_add_command(interface_index: u32, server: &IpAddr) -> String {
    // Add DNS server:
    // netsh interface ipv4 add dnsservers name="Mullvad" address=10.64.0.2 validate=no

    let interface_type = if server.is_ipv4() { "ipv4" } else { "ipv6" };
    format!("interface {interface_type} add dnsservers name={interface_index} address={server} validate=no\r\n")
}

fn create_netsh_flush_command(interface_index: u32, ip_version: IpVersion) -> String {
    // Flush DNS settings:
    // netsh interface ipv4 set dnsservers name="Mullvad" source=static address=none validate=no

    let interface_type = match ip_version {
        IpVersion::V4 => "ipv4",
        IpVersion::V6 => "ipv6",
    };

    format!("interface {interface_type} set dnsservers name={interface_index} source=static address=none validate=no\r\n")
}

fn get_system_dir() -> io::Result<PathBuf> {
    let mut sysdir = [0u16; MAX_PATH as usize + 1];
    let len = unsafe { GetSystemDirectoryW(sysdir.as_mut_ptr(), (sysdir.len() - 1) as u32) };
    if len == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(PathBuf::from(OsString::from_wide(
        &sysdir[0..(len as usize)],
    )))
}
