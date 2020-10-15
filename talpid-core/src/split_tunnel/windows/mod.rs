mod driver;
mod windows;

use std::{
    ffi::OsStr,
    io, mem,
    net::{Ipv4Addr, Ipv6Addr},
    os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle},
    ptr,
};
use talpid_types::ErrorExt;
use winapi::{
    shared::minwindef::{FALSE, TRUE},
    um::{minwinbase::OVERLAPPED, processthreadsapi::TerminateThread, synchapi::CreateEventW},
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
}

struct EventThreadContext {
    handle: RawHandle,
    event_overlapped: OVERLAPPED,
}
// FIXME: ! This is not safe. The driver handle will be invalidated when SplitTunnel is dropped
unsafe impl Send for EventThreadContext {}

impl SplitTunnel {
    /// Initialize the driver.
    pub fn new() -> Result<Self, Error> {
        let handle = driver::DeviceHandle::new().map_err(Error::InitializationFailed)?;

        // FIXME: Want to use same pointer, but must be certain that the thread dies after this dies

        let mut event_overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        event_overlapped.hEvent =
            unsafe { CreateEventW(ptr::null_mut(), TRUE, FALSE, ptr::null()) };
        if event_overlapped.hEvent == ptr::null_mut() {
            return Err(Error::EventThreadError(io::Error::last_os_error()));
        }

        let mut event_context = EventThreadContext {
            handle: handle.as_raw_handle(),
            event_overlapped,
        };

        let event_thread = std::thread::spawn(move || {
            use driver::{EventBody, EventId};

            let mut data_buffer = Vec::with_capacity(DRIVER_EVENT_BUFFER_SIZE);

            loop {
                match driver::deque_event(
                    event_context.handle,
                    &mut data_buffer,
                    &mut event_context.event_overlapped,
                ) {
                    Ok((event_id, event_body)) => {
                        let event_str = match &event_id {
                            EventId::StartSplittingProcess
                            | EventId::ErrorStartSplittingProcess => "Start splitting process",
                            EventId::StopSplittingProcess | EventId::ErrorStopSplittingProcess => {
                                "Stop splitting process"
                            }
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
                    Err(error) => {
                        log::error!("{}", error.display_chain_with_msg("deque_event failed"));
                    }
                }

                // TODO: Quit when signaled. Overlapping + WaitForMultipleObjects?
            }

            // FIXME: The event object will not be destroyed since we use TerminateThread
            // unsafe { CloseHandle(event_overlapped.hEvent) };
        });

        Ok(SplitTunnel {
            handle,
            event_thread: Some(event_thread),
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
        // FIXME: Use signals to close the thread gracefully, followed by a join
        if let Some(event_thread) = self.event_thread.take() {
            unsafe {
                TerminateThread(event_thread.into_raw_handle(), 0);
            }
        }

        let paths: [&OsStr; 0] = [];
        if let Err(error) = self.set_paths(&paths) {
            log::error!("{}", error.display_chain());
        }
    }
}
