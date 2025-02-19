use std::{io, mem};
use windows_sys::Win32::System::IO::OVERLAPPED;

use crate::sync::Event;

/// Abstraction over `OVERLAPPED`.
///
/// - https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-overlapped
/// - https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createeventw
pub struct Overlapped {
    overlapped: OVERLAPPED,
    event: Option<Event>,
}

// SAFETY: Both OVERLAPPED and Event is used for async I/O, so this *should* be safe.
unsafe impl Send for Overlapped {}

impl Overlapped {
    /// Creates an `OVERLAPPED` object with `hEvent` set.
    pub fn new(event: Option<Event>) -> io::Result<Self> {
        let mut overlapped = Overlapped {
            // SAFETY: OVERLAPPED is a C struct and can safely be zeroed.
            overlapped: unsafe { mem::zeroed() },
            event: None,
        };
        overlapped.set_event(event);
        Ok(overlapped)
    }

    /// Borrows the underlying `OVERLAPPED` object.
    pub fn as_mut_ptr(&mut self) -> *mut OVERLAPPED {
        &mut self.overlapped
    }

    /// Returns a reference to the associated event.
    pub fn get_event(&self) -> Option<&Event> {
        self.event.as_ref()
    }

    /// Sets the event object for the underlying `OVERLAPPED` object (i.e., `hEvent`)
    fn set_event(&mut self, event: Option<Event>) {
        match event {
            Some(event) => {
                self.overlapped.hEvent = event.as_raw();
                self.event = Some(event);
            }
            None => {
                self.overlapped.hEvent = 0;
                self.event = None;
            }
        }
    }
}
