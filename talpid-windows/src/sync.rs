#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, DuplicateHandle, BOOL, DUPLICATE_SAME_ACCESS, HANDLE},
    System::Threading::{CreateEventW, GetCurrentProcess, SetEvent},
};

/// Windows event object
pub struct Event(HANDLE);

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Event {
    /// Create a new event object using `CreateEventW`
    pub fn new(manual_reset: bool, initial_state: bool) -> io::Result<Self> {
        let event = unsafe {
            CreateEventW(
                ptr::null_mut(),
                bool_to_winbool(manual_reset),
                bool_to_winbool(initial_state),
                ptr::null(),
            )
        };
        if event == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(event))
    }

    /// Signal the event object
    pub fn set(&self) -> io::Result<()> {
        if unsafe { SetEvent(self.0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Return raw event object
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }

    /// Duplicate the event object with `DuplicateHandle()`
    pub fn duplicate(&self) -> io::Result<Event> {
        let mut new_event = 0;
        let status = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.0,
                GetCurrentProcess(),
                &mut new_event,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if status == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Event(new_event))
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

const fn bool_to_winbool(val: bool) -> BOOL {
    match val {
        true => 1,
        false => 0,
    }
}
