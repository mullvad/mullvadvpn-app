use std::ffi::{OsStr, OsString};
use std::{io, ptr};
use widestring::{WideCStr, WideCString};
use winapi::um::winsvc;

mod errors {
    error_chain! {
        errors {
            InvalidServiceName {
                description("Invalid service name")
            }
        }
        foreign_links {
            System(::std::io::Error);
        }
    }
}
pub use self::errors::*;

/// Macro to generate a "service_main" function for Windows service.
///
/// The `service_main` function parses service arguments provided by the system
/// and passes them with a call to `$service_main_handler`.
///
/// `$function_name` - name of the "service_main" callback.
/// `$service_main_handler` - function with a signature `fn(Vec<OsString>)` that's called from
/// generated `$function_name`. Accepts parsed service arguments as `Vec<OsString>`. Its
/// responsibility is to create a `ServiceControlHandler`, start processing control events and
/// report the service status to the system.
///
#[macro_export]
macro_rules! define_windows_service {
    ($function_name:ident, $service_main_handler:ident) => {
        /// Static callback used by the system to bootstrap the service.
        /// Do not call it directly.
        extern "system" fn $function_name(argc: u32, argv: *mut *mut u16) {
            let arguments = unsafe { $crate::service_dispatcher::parse_raw_arguments(argc, argv) };

            $service_main_handler(arguments);
        }
    };
}

/// Start service control dispatcher.
///
/// Once started the service control dispatcher blocks the current thread execution
/// until the service is stopped.
///
/// Upon successful initialization, system calls the `service_main` in
/// background thread which parses service arguments received from the system and
/// passes them to higher level `$service_main_handler` handler.
///
/// On failure: immediately returns an error, no threads are spawned.
///
pub fn start_dispatcher<T: AsRef<OsStr>>(
    service_name: T,
    service_main: extern "system" fn(u32, *mut *mut u16),
) -> Result<()> {
    let service_name =
        WideCString::from_str(service_name).chain_err(|| ErrorKind::InvalidServiceName)?;
    let service_table: &[winsvc::SERVICE_TABLE_ENTRYW] = &[
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: service_name.as_ptr(),
            lpServiceProc: Some(service_main),
        },
        // the last item has to be { null, null }
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: ptr::null(),
            lpServiceProc: None,
        },
    ];

    let result = unsafe { winsvc::StartServiceCtrlDispatcherW(service_table.as_ptr()) };
    if result == 0 {
        Err(io::Error::last_os_error().into())
    } else {
        Ok(())
    }
}

/// Parse raw arguments received from `service_main` into Vec.
pub unsafe fn parse_raw_arguments(argc: u32, argv: *mut *mut u16) -> Vec<OsString> {
    (0..argc)
        .into_iter()
        .map(|i| {
            let array_element_ptr: *mut *mut u16 = argv.offset(i as isize);
            WideCStr::from_ptr_str(*array_element_ptr).to_os_string()
        })
        .collect()
}
