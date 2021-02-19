mod driver;
mod windows;

use std::{
    ffi::OsStr,
    io, mem,
    net::{Ipv4Addr, Ipv6Addr},
    os::windows::io::{AsRawHandle, RawHandle},
    ptr,
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

    /// Failed to set up the driver event loop
    #[error(display = "Failed to set up the driver event loop")]
    EventThreadError(#[error(source)] io::Error),
}

/// Manages applications whose traffic to exclude from the tunnel.
pub struct SplitTunnel {
    handle: driver::DeviceHandle,
    event_thread: Option<std::thread::JoinHandle<()>>,
    quit_event: RawHandle,
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
                    _ => "Unknown event ID",
                };

                match event_body {
                    EventBody::SplittingError { process_id, image } => {
                        log::error!(
                            "FAILED: {}:\n\tpid: {}\n\timage: {:?}",
                            event_str,
                            process_id,
                            image,
                        );
                    }
                    _ => (),
                }
            }

            log::debug!("Stopping split tunnel event thread");

            unsafe { CloseHandle(event_context.event_overlapped.hEvent) };
        });

        Ok(SplitTunnel {
            handle,
            event_thread: Some(event_thread),
            quit_event,
        })
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths<T: AsRef<OsStr>>(&self, paths: &[T]) -> Result<(), Error> {
        if paths.len() > 0 {
            self.handle
                .set_config(paths)
                .map_err(Error::SetConfiguration)
        } else {
            self.handle.clear_config().map_err(Error::SetConfiguration)
        }
    }

    /// Configures IP addresses used for socket rebinding.
    pub fn register_ips(
        &self,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> Result<(), Error> {
        self.handle
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
