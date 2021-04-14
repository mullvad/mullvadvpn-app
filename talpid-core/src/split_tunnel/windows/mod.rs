mod driver;
mod windows;

use crate::{
    tunnel::TunnelMetadata,
    winnet::{
        self, get_best_default_route, interface_luid_to_ip, WinNetAddrFamily, WinNetCallbackHandle,
    },
};
use lazy_static::lazy_static;
use std::{
    ffi::OsStr,
    io, mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::io::{AsRawHandle, RawHandle},
    ptr,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;
use winapi::{
    shared::minwindef::{FALSE, TRUE},
    um::{
        handleapi::CloseHandle,
        ioapiset::GetOverlappedResult,
        minwinbase::OVERLAPPED,
        synchapi::{CreateEventW, SetEvent, WaitForMultipleObjects, WaitForSingleObject},
        winbase::{INFINITE, WAIT_ABANDONED_0, WAIT_OBJECT_0},
    },
};

const DRIVER_EVENT_BUFFER_SIZE: usize = 2048;

lazy_static! {
    static ref RESERVED_IP_V4: Ipv4Addr = "192.0.2.123".parse().unwrap();
}

/// Errors that may occur in [`SplitTunnel`].
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to identify or initialize the driver
    #[error(display = "Failed to find or initialize driver")]
    InitializationFailed(#[error(source)] io::Error),

    /// Failed to set paths to excluded applications
    #[error(display = "Failed to set list of excluded applications")]
    SetConfiguration(#[error(source)] io::Error),

    /// Failed to register interface IP addresses
    #[error(display = "Failed to register IP addresses for exclusions")]
    RegisterIps(#[error(source)] io::Error),

    /// Failed to clear interface IP addresses
    #[error(display = "Failed to clear registered IP addresses")]
    ClearIps(#[error(source)] io::Error),

    /// Failed to set up the driver event loop
    #[error(display = "Failed to set up the driver event loop")]
    EventThreadError(#[error(source)] io::Error),

    /// Failed to obtain default route
    #[error(display = "Failed to obtain the default route")]
    ObtainDefaultRoute(#[error(source)] winnet::Error),

    /// Failed to obtain an IP address given a network interface LUID
    #[error(display = "Failed to obtain IP address for interface LUID")]
    LuidToIp(#[error(source)] winnet::Error),

    /// Failed to set up callback for monitoring default route changes
    #[error(display = "Failed to register default route change callback")]
    RegisterRouteChangeCallback,
}

/// Manages applications whose traffic to exclude from the tunnel.
pub struct SplitTunnel {
    handle: Arc<Mutex<driver::DeviceHandle>>,
    event_thread: Option<std::thread::JoinHandle<()>>,
    quit_event: RawHandle,
    _route_change_callback: Option<WinNetCallbackHandle>,
}

struct EventThreadContext {
    handle: RawHandle,
    event_overlapped: OVERLAPPED,
    quit_event: RawHandle,
}
unsafe impl Send for EventThreadContext {}

impl SplitTunnel {
    /// Initialize the driver.
    pub fn new() -> Result<Self, Error> {
        let handle = driver::DeviceHandle::new().map_err(Error::InitializationFailed)?;

        let mut event_overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        event_overlapped.hEvent =
            unsafe { CreateEventW(ptr::null_mut(), TRUE, FALSE, ptr::null()) };
        if event_overlapped.hEvent == ptr::null_mut() {
            return Err(Error::EventThreadError(io::Error::last_os_error()));
        }

        let quit_event = unsafe { CreateEventW(ptr::null_mut(), TRUE, FALSE, ptr::null()) };

        let event_context = EventThreadContext {
            handle: handle.as_raw_handle(),
            event_overlapped,
            quit_event,
        };

        let event_thread = std::thread::spawn(move || {
            use driver::{EventBody, EventId};

            let mut data_buffer = Vec::with_capacity(DRIVER_EVENT_BUFFER_SIZE);
            let mut returned_bytes = 0u32;

            let event_objects = [
                event_context.event_overlapped.hEvent,
                event_context.quit_event,
            ];

            loop {
                if unsafe { WaitForSingleObject(event_context.quit_event, 0) == WAIT_OBJECT_0 } {
                    // Quit event was signaled
                    break;
                }

                if let Err(error) = unsafe {
                    driver::device_io_control_buffer_async(
                        event_context.handle,
                        driver::DriverIoctlCode::DequeEvent as u32,
                        Some(&mut data_buffer),
                        None,
                        &event_context.event_overlapped,
                    )
                } {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("device_io_control failed")
                    );
                    continue;
                }

                let result = unsafe {
                    WaitForMultipleObjects(
                        event_objects.len() as u32,
                        &event_objects[0],
                        FALSE,
                        INFINITE,
                    )
                };

                let signaled_index = if result >= WAIT_OBJECT_0
                    && result < WAIT_OBJECT_0 + event_objects.len() as u32
                {
                    result - WAIT_OBJECT_0
                } else if result >= WAIT_ABANDONED_0
                    && result < WAIT_ABANDONED_0 + event_objects.len() as u32
                {
                    result - WAIT_ABANDONED_0
                } else {
                    let error = io::Error::last_os_error();
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("WaitForMultipleObjects failed")
                    );

                    continue;
                };

                if event_context.quit_event == event_objects[signaled_index as usize] {
                    // Quit event was signaled
                    break;
                }

                let result = unsafe {
                    GetOverlappedResult(
                        event_context.handle,
                        &event_context.event_overlapped as *const _ as *mut _,
                        &mut returned_bytes,
                        TRUE,
                    )
                };

                if result == 0 {
                    let error = io::Error::last_os_error();
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("GetOverlappedResult failed")
                    );

                    continue;
                }

                unsafe { data_buffer.set_len(returned_bytes as usize) };

                let event = driver::parse_event_buffer(&data_buffer);

                let (event_id, event_body) = match event {
                    Some((event_id, event_body)) => (event_id, event_body),
                    None => continue,
                };

                let event_str = match &event_id {
                    EventId::StartSplittingProcess | EventId::ErrorStartSplittingProcess => {
                        "Start splitting process"
                    }
                    EventId::StopSplittingProcess | EventId::ErrorStopSplittingProcess => {
                        "Stop splitting process"
                    }
                    EventId::ErrorMessage => "ErrorMessage",
                    _ => "Unknown event ID",
                };

                match event_body {
                    EventBody::SplittingEvent {
                        process_id,
                        reason,
                        image,
                    } => {
                        log::trace!(
                            "{}:\n\tpid: {}\n\treason: {:?}\n\timage: {:?}",
                            event_str,
                            process_id,
                            reason,
                            image,
                        );
                    }
                    EventBody::SplittingError { process_id, image } => {
                        log::error!(
                            "FAILED: {}:\n\tpid: {}\n\timage: {:?}",
                            event_str,
                            process_id,
                            image,
                        );
                    }
                    EventBody::ErrorMessage { status, message } => {
                        log::error!("NTSTATUS {:#x}: {}", status, message.to_string_lossy())
                    }
                }
            }

            log::debug!("Stopping split tunnel event thread");

            unsafe { CloseHandle(event_context.event_overlapped.hEvent) };
        });

        Ok(SplitTunnel {
            handle: Arc::new(Mutex::new(handle)),
            event_thread: Some(event_thread),
            quit_event,
            _route_change_callback: None,
        })
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths<T: AsRef<OsStr>>(&self, paths: &[T]) -> Result<(), Error> {
        if paths.len() > 0 {
            self.handle
                .lock()
                .expect("ST driver mutex poisoned")
                .set_config(paths)
                .map_err(Error::SetConfiguration)
        } else {
            self.handle
                .lock()
                .expect("ST driver mutex poisoned")
                .clear_config()
                .map_err(Error::SetConfiguration)
        }
    }

    /// Instructs the driver to redirect traffic from sockets bound to 0.0.0.0, ::, or the
    /// tunnel addresses (if any) to the default route.
    pub fn set_tunnel_addresses(&mut self, metadata: Option<&TunnelMetadata>) -> Result<(), Error> {
        let mut tunnel_ipv4 = None;
        let mut tunnel_ipv6 = None;

        if let Some(metadata) = metadata {
            for ip in &metadata.ips {
                match ip {
                    IpAddr::V4(address) => tunnel_ipv4 = Some(address.clone()),
                    IpAddr::V6(address) => tunnel_ipv6 = Some(address.clone()),
                }
            }
        }

        // Identify IP address that gives us Internet access
        let internet_ipv4 = get_best_default_route(WinNetAddrFamily::IPV4)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV4, route.interface_luid))
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();
        let internet_ipv6 = get_best_default_route(WinNetAddrFamily::IPV6)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV6, route.interface_luid))
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();

        let tunnel_ipv4 = tunnel_ipv4.unwrap_or(*RESERVED_IP_V4);
        let internet_ipv4 = Ipv4Addr::from(internet_ipv4.unwrap_or_default());
        let internet_ipv6 = internet_ipv6.map(|addr| Ipv6Addr::from(addr));

        let context = SplitTunnelDefaultRouteChangeHandlerContext::new(
            self.handle.clone(),
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        );

        self._route_change_callback = None;

        Self::register_ips(
            &*self.handle,
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        )?;

        self._route_change_callback = winnet::add_default_route_change_callback(
            Some(split_tunnel_default_route_change_handler),
            context,
        )
        .map(Some)
        .map_err(|_| Error::RegisterRouteChangeCallback)?;

        Ok(())
    }

    /// Instructs the driver to stop redirecting tunnel traffic and INADDR_ANY.
    pub fn clear_tunnel_addresses(&mut self) -> Result<(), Error> {
        self._route_change_callback = None;

        Self::register_ips(
            &*self.handle,
            Ipv4Addr::new(0, 0, 0, 0),
            None,
            Ipv4Addr::new(0, 0, 0, 0),
            None,
        )
    }

    /// Configures IP addresses used for socket rebinding.
    fn register_ips(
        handle: &Mutex<driver::DeviceHandle>,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> Result<(), Error> {
        log::debug!(
            "Register IPs: {} {:?} {} {:?}",
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6
        );

        // If there is no valid internet IPv4 address, ignore any tunnel addresses.
        // This should only be the case if a reserved tunnel IP is used to keep the driver engaged.
        let tunnel_ipv4 = if internet_ipv4.is_unspecified() {
            Ipv4Addr::new(0, 0, 0, 0)
        } else {
            tunnel_ipv4
        };

        handle
            .lock()
            .expect("ST driver mutex poisoned")
            .register_ips(tunnel_ipv4, tunnel_ipv6, internet_ipv4, internet_ipv6)
            .map_err(Error::RegisterIps)
    }
}

impl Drop for SplitTunnel {
    fn drop(&mut self) {
        if let Some(event_thread) = self.event_thread.take() {
            if unsafe { SetEvent(self.quit_event) } != 0 {
                let _ = event_thread.join();
            } else {
                let error = io::Error::last_os_error();
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to close ST event thread")
                );
            }
        }

        unsafe { CloseHandle(self.quit_event) };

        let paths: [&OsStr; 0] = [];
        if let Err(error) = self.set_paths(&paths) {
            log::error!("{}", error.display_chain());
        }
    }
}

struct SplitTunnelDefaultRouteChangeHandlerContext {
    handle: Arc<Mutex<driver::DeviceHandle>>,
    pub tunnel_ipv4: Ipv4Addr,
    pub tunnel_ipv6: Option<Ipv6Addr>,
    pub internet_ipv4: Ipv4Addr,
    pub internet_ipv6: Option<Ipv6Addr>,
}

impl SplitTunnelDefaultRouteChangeHandlerContext {
    pub fn new(
        handle: Arc<Mutex<driver::DeviceHandle>>,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> Self {
        SplitTunnelDefaultRouteChangeHandlerContext {
            handle,
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        }
    }

    pub fn register_ips(&self) -> Result<(), Error> {
        SplitTunnel::register_ips(
            &*self.handle,
            self.tunnel_ipv4,
            self.tunnel_ipv6,
            self.internet_ipv4,
            self.internet_ipv6,
        )
    }
}

unsafe extern "system" fn split_tunnel_default_route_change_handler(
    event_type: winnet::WinNetDefaultRouteChangeEventType,
    address_family: WinNetAddrFamily,
    default_route: winnet::WinNetDefaultRoute,
    ctx: *mut libc::c_void,
) {
    // Update the "internet interface" IP when best default route changes
    let ctx = &mut *(ctx as *mut SplitTunnelDefaultRouteChangeHandlerContext);

    let result = match event_type {
        winnet::WinNetDefaultRouteChangeEventType::DefaultRouteChanged => {
            let ip = interface_luid_to_ip(address_family.clone(), default_route.interface_luid);

            let ip = match ip {
                Ok(Some(ip)) => ip,
                Ok(None) => {
                    log::error!("Failed to obtain new default route address: none found",);
                    // TODO: Send tunnel command
                    return;
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to obtain new default route address")
                    );
                    // TODO: Send tunnel command
                    return;
                }
            };

            match address_family {
                WinNetAddrFamily::IPV4 => {
                    let ip = Ipv4Addr::from(ip);
                    ctx.internet_ipv4 = ip;
                }
                WinNetAddrFamily::IPV6 => {
                    let ip = Ipv6Addr::from(ip);
                    ctx.internet_ipv6 = Some(ip);
                }
            }

            ctx.register_ips()
        }
        // no default route
        winnet::WinNetDefaultRouteChangeEventType::DefaultRouteRemoved => {
            match address_family {
                WinNetAddrFamily::IPV4 => {
                    ctx.internet_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
                }
                WinNetAddrFamily::IPV6 => {
                    ctx.internet_ipv6 = None;
                }
            }
            ctx.register_ips()
        }
    };

    if let Err(error) = result {
        // TODO: Send tunnel command
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to register new addresses in split tunnel driver")
        );
    }
}
