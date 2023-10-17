use std::{io, mem};
use windows_sys::Win32::System::IO::OVERLAPPED;

use crate::sync::Event;

/// Abstraction over `OVERLAPPED`.
pub struct Overlapped {
    overlapped: OVERLAPPED,
    event: Option<Event>,
}

unsafe impl Send for Overlapped {}
unsafe impl Sync for Overlapped {}

impl Overlapped {
    /// Creates an `OVERLAPPED` object with `hEvent` set.
    pub fn new(event: Option<Event>) -> io::Result<Self> {
        let mut overlapped = Overlapped {
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
