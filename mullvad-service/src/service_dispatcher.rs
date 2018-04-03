use std::ffi::OsString;
use widestring::{from_raw_wide_string, to_wide_with_nul};
use winapi::um::winsvc;

macro_rules! define_service_main {
    ($func_name: ident, $handler_name: ident) => {
        extern "system" fn $func_name(argc: u32, argv: *mut *mut u16) {
            use $crate::service_dispatcher::parse_service_main_arguments;
            let arguments = unsafe { parse_service_main_arguments(argc, argv) };
            $handler_name(arguments);
        }
    };
}

// TBD: Rethink the macro, maybe produce a static struct?
macro_rules! start_service_dispatcher {
    ($service_name: ident, $service_main_func: ident) => {{
        use std::ptr;
        use widestring::{from_raw_wide_string, to_wide_with_nul};
        use winapi::um::winsvc;

        let service_name = to_wide_with_nul($service_name);

        let service_table: &[winsvc::SERVICE_TABLE_ENTRYW] = &[
            winsvc::SERVICE_TABLE_ENTRYW {
                lpServiceName: service_name.as_ptr(),
                lpServiceProc: Some($service_main_func),
            },
            // the last item has to be { null, null }
            winsvc::SERVICE_TABLE_ENTRYW {
                lpServiceName: ptr::null(),
                lpServiceProc: None,
            },
        ];

        debug!(
            "Starting service control dispatcher from thread: {:?}",
            thread::current().id()
        );

        // Blocks current thread until the service is stopped
        // This call spawns a new thread and calls `service_main`
        let result = unsafe { winsvc::StartServiceCtrlDispatcherW(service_table.as_ptr()) };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }};
}

/// Parse an unsafe array of raw string pointers received in `service_main` from Windows API.
pub unsafe fn parse_service_main_arguments(argc: u32, argv: *mut *mut u16) -> Vec<OsString> {
    (0..argc)
        .into_iter()
        .map(|i| {
            let ptr = argv.offset(i as isize);
            from_raw_wide_string(*ptr, 256)
        })
        .collect()
}
