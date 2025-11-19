#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle},
    ptr,
};
use windows_sys::Win32::{
    Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle},
    System::Threading::{CreateEventW, GetCurrentProcess, SetEvent},
};

/// Windows event object
pub struct Event(OwnedHandle);

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Event {
    /// Create a new event object using `CreateEventW`
    pub fn new(manual_reset: bool, initial_state: bool) -> io::Result<Self> {
        let event = unsafe {
            CreateEventW(
                ptr::null_mut(),
                i32::from(manual_reset),
                i32::from(initial_state),
                ptr::null(),
            )
        };
        if event.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `event` is a valid handle since `CreateEventW` succeeded
        Ok(Self(unsafe { OwnedHandle::from_raw_handle(event) }))
    }

    /// Signal the event object
    pub fn set(&self) -> io::Result<()> {
        // SAFETY: `self.0` is a valid handle
        if unsafe { SetEvent(self.0.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Duplicate the event object with `DuplicateHandle()`
    pub fn duplicate(&self) -> io::Result<Event> {
        let mut new_event = ptr::null_mut();
        let status = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.0.as_raw_handle(),
                GetCurrentProcess(),
                &raw mut new_event,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if status == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `new_event` is a valid handle since `DuplicateHandle` succeeded
        Ok(Event(unsafe { OwnedHandle::from_raw_handle(new_event) }))
    }
}

impl AsRawHandle for Event {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}
