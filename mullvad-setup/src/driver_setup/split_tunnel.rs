use std::{
    fs::{self, OpenOptions},
    io,
    mem::size_of,
    os::windows::{
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    ptr,
};
use windows_sys::Win32::{
    Foundation::{FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::FILE_FLAG_OVERLAPPED,
    System::{
        IO::{DeviceIoControl, GetOverlappedResult, OVERLAPPED},
        Threading::CreateEventW,
    },
};

/// Name of the service that represents the kernel driver/device
pub const SERVICE_NAME: &str = "mullvad-split-tunnel";

const ST_DEVICE_TYPE: u32 = 0x8000;

/// https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-i-o-control-codes.
const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

// METHOD_BUFFERED = 0, METHOD_NEITHER = 3, FILE_ANY_ACCESS = 0
const IOCTL_ST_GET_STATE: u32 = ctl_code(ST_DEVICE_TYPE, 9, 0, 0);
const IOCTL_ST_RESET: u32 = ctl_code(ST_DEVICE_TYPE, 11, 3, 0);

// State value indicating the driver is ready
const ST_DRIVER_STATE_STARTED: u64 = 1;

// Win32 device path for the Mullvad split tunnel device.
const ST_DEVICE_PATH: &str = r"\\.\MULLVADSPLITTUNNEL";

struct StDevice {
    file: fs::File,
}

impl StDevice {
    pub fn open() -> Result<Self, crate::Error> {
        let file = OpenOptions::new()
            .access_mode(GENERIC_READ | GENERIC_WRITE)
            .share_mode(0) // FILE_SHARE_NONE
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(ST_DEVICE_PATH)
            .map_err(crate::Error::OpenDevice)?;

        Ok(Self { file })
    }

    /// Send reset ioctl to device
    pub fn reset(&self) -> Result<(), crate::Error> {
        // SAFETY: IOCTL_ST_RESET takes no input or output buffer, so passing NULL pointers
        // with zero sizes is correct.
        unsafe {
            send_ioctl(
                self.file.as_raw_handle(),
                IOCTL_ST_RESET,
                ptr::null(),
                0,
                ptr::null_mut(),
                0,
            )
        }
        .map_err(crate::Error::IoControl)?;
        Ok(())
    }

    /// Get split tunnel device state
    pub fn state(&self) -> Result<u64, crate::Error> {
        let mut state: u64 = 0;
        // SAFETY: IOCTL_ST_GET_STATE writes a u64 to the output buffer; `state` is a writable
        // u64 and the size matches.
        let bytes_returned = unsafe {
            send_ioctl(
                self.file.as_raw_handle(),
                IOCTL_ST_GET_STATE,
                ptr::null(),
                0,
                (&raw mut state).cast(),
                size_of::<u64>() as u32,
            )
        }
        .map_err(crate::Error::IoControl)?;

        if bytes_returned != size_of::<u64>() as u32 {
            return Err(crate::Error::UnexpectedDriverState(state));
        }
        Ok(state)
    }
}

/// Open the split tunnel device, send IOCTL_ST_RESET, then verify
/// the driver state is `ST_DRIVER_STATE_STARTED`.
pub fn reset_driver_state() -> Result<(), crate::Error> {
    let device = StDevice::open()?;
    device.reset()?;
    let state = device.state()?;
    if state != ST_DRIVER_STATE_STARTED {
        return Err(crate::Error::UnexpectedDriverState(state));
    }
    Ok(())
}

/// Send an IO control code to the device using overlapped I/O, waiting for completion.
/// Returns the number of bytes transferred.
///
/// # Safety
///
/// `in_buffer`/`in_size` and `out_buffer`/`out_size` must describe valid buffers (or be
/// NULL/0) appropriate for the given IOCTL `code`. `device` must be a valid handle opened
/// for overlapped I/O.
unsafe fn send_ioctl(
    device: HANDLE,
    code: u32,
    in_buffer: *const core::ffi::c_void,
    in_size: u32,
    out_buffer: *mut core::ffi::c_void,
    out_size: u32,
) -> io::Result<u32> {
    // SAFETY: All pointer arguments are documented as optional and we pass NULL.
    let event = unsafe { CreateEventW(ptr::null_mut(), 1, 0, ptr::null()) };
    if event == INVALID_HANDLE_VALUE || event.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `event` is a freshly created HANDLE that we own and do not close elsewhere.
    let _event_guard = unsafe { OwnedHandle::from_raw_handle(event) };

    let mut overlapped = OVERLAPPED {
        hEvent: event,
        ..OVERLAPPED::default()
    };

    let mut bytes_returned: u32 = 0;
    // SAFETY: `device` is a valid handle owned by the caller; the in/out buffer pointers
    // and sizes are the caller's responsibility (this is an internal helper). `overlapped`
    // is a valid OVERLAPPED with a live event handle.
    let result = unsafe {
        DeviceIoControl(
            device,
            code,
            in_buffer,
            in_size,
            out_buffer,
            out_size,
            &raw mut bytes_returned,
            &raw mut overlapped,
        )
    };

    if result != FALSE {
        return Ok(bytes_returned);
    }

    let err = io::Error::last_os_error();
    if err.raw_os_error() != Some(windows_sys::Win32::Foundation::ERROR_IO_PENDING as i32) {
        return Err(err);
    }

    // Wait for async completion.
    // SAFETY: `device` and `overlapped` are still valid; `bytes_returned` is a writable u32.
    let result = unsafe {
        GetOverlappedResult(
            device,
            &raw const overlapped,
            &raw mut bytes_returned,
            1, /* TRUE */
        )
    };
    if result == FALSE {
        return Err(io::Error::last_os_error());
    }

    Ok(bytes_returned)
}
