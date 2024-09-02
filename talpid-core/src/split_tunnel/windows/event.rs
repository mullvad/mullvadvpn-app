//! Handle events dispatched from win-split-tunnel.
//!
//! It follows a typical inverted-call model, in which we request the next event, and block until
//! such an event has been received, or until a quit event is signaled.

use super::driver;
use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};
use talpid_types::{split_tunnel::ExcludedProcess, ErrorExt};
use talpid_windows::{io::Overlapped, sync::Event};
use windows_sys::Win32::Foundation::ERROR_OPERATION_ABORTED;

enum EventResult {
    /// Result containing the next event.
    Event(driver::EventId, driver::EventBody),
    /// Quit event was signaled.
    Quit,
}

const DRIVER_EVENT_BUFFER_SIZE: usize = 2048;

/// Spawns an event loop thread that processes events from the driver service.
pub fn spawn_listener(
    handle: Arc<driver::DeviceHandle>,
    excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
) -> io::Result<(std::thread::JoinHandle<()>, Arc<Event>)> {
    let mut event_overlapped = Overlapped::new(Some(Event::new(true, false)?))?;

    let quit_event = Arc::new(Event::new(true, false)?);
    let quit_event_copy = quit_event.clone();

    let event_thread = std::thread::spawn(move || {
        log::debug!("Starting split tunnel event thread");
        let mut data_buffer = vec![];

        loop {
            // Wait until either the next event is received or the quit event is signaled.
            let (event_id, event_body) = match fetch_next_event(
                &handle,
                &quit_event,
                &mut event_overlapped,
                &mut data_buffer,
            ) {
                Ok(EventResult::Event(event_id, event_body)) => (event_id, event_body),
                Ok(EventResult::Quit) => break,
                Err(error) => {
                    if error.raw_os_error() == Some(ERROR_OPERATION_ABORTED as i32) {
                        // The driver will normally abort the request if the driver state
                        // is reset. Give the driver service some time to recover before
                        // retrying.
                        std::thread::sleep(Duration::from_millis(500));
                    }
                    continue;
                }
            };

            handle_event(event_id, event_body, &excluded_processes);
        }

        log::debug!("Stopping split tunnel event thread");
    });

    Ok((event_thread, quit_event_copy))
}

fn fetch_next_event(
    device: &Arc<driver::DeviceHandle>,
    quit_event: &Event,
    overlapped: &mut Overlapped,
    data_buffer: &mut Vec<u8>,
) -> io::Result<EventResult> {
    if unsafe { driver::wait_for_single_object(quit_event.as_raw(), Some(Duration::ZERO)) }.is_ok()
    {
        return Ok(EventResult::Quit);
    }

    data_buffer.resize(DRIVER_EVENT_BUFFER_SIZE, 0u8);

    unsafe {
        driver::device_io_control_buffer_async(
            device,
            driver::DriverIoctlCode::DequeEvent as u32,
            None,
            data_buffer.as_mut_ptr(),
            u32::try_from(data_buffer.len()).expect("buffer must be smaller than u32"),
            overlapped.as_mut_ptr(),
        )
    }
    .map_err(|error| {
        log::error!(
            "{}",
            error.display_chain_with_msg("DeviceIoControl failed to deque event")
        );
        error
    })?;

    let event_objects = [
        overlapped.get_event().unwrap().as_raw(),
        quit_event.as_raw(),
    ];

    let signaled_object = unsafe { driver::wait_for_multiple_objects(&event_objects[..], false) }
        .map_err(|error| {
        log::error!(
            "{}",
            error.display_chain_with_msg("wait_for_multiple_objects failed")
        );
        error
    })?;

    if signaled_object == quit_event.as_raw() {
        // Quit event was signaled
        return Ok(EventResult::Quit);
    }

    let returned_bytes = driver::get_overlapped_result(device, overlapped).map_err(|error| {
        if error.raw_os_error() != Some(ERROR_OPERATION_ABORTED as i32) {
            log::error!(
                "{}",
                error.display_chain_with_msg("get_overlapped_result failed for dequeued event"),
            );
        }
        error
    })?;

    data_buffer
        .truncate(usize::try_from(returned_bytes).expect("usize must be no smaller than u32"));

    driver::parse_event_buffer(data_buffer)
        .map(|(id, body)| EventResult::Event(id, body))
        .map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to parse ST event buffer")
            );
            io::Error::new(io::ErrorKind::Other, "Failed to parse ST event buffer")
        })
}

fn handle_event(
    event_id: driver::EventId,
    event_body: driver::EventBody,
    excluded_processes: &Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
) {
    use driver::{EventBody, EventId};

    let event_str = match &event_id {
        EventId::StartSplittingProcess | EventId::ErrorStartSplittingProcess => {
            "Start splitting process"
        }
        EventId::StopSplittingProcess | EventId::ErrorStopSplittingProcess => {
            "Stop splitting process"
        }
        EventId::ErrorMessage => "ErrorMessage",
    };

    match event_body {
        EventBody::SplittingEvent {
            process_id,
            reason,
            image,
        } => {
            let mut pids = excluded_processes.write().unwrap();
            match event_id {
                EventId::StartSplittingProcess => {
                    if let Some(prev_entry) = pids.get(&process_id) {
                        log::error!("PID collision: {process_id} is already in the list of excluded processes. New image: {:?}. Current image: {:?}", image, prev_entry);
                    }
                    pids.insert(
                        process_id,
                        ExcludedProcess {
                            pid: u32::try_from(process_id)
                                .expect("PID should be containable in a DWORD"),
                            image: Path::new(&image).to_path_buf(),
                            inherited: reason
                                .contains(driver::SplittingChangeReason::BY_INHERITANCE),
                        },
                    );
                }
                EventId::StopSplittingProcess => {
                    if pids.remove(&process_id).is_none() {
                        log::error!("Inconsistent process tree: {process_id} was not found");
                    }
                }
                _ => (),
            }

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
