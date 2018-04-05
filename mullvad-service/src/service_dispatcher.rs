/// Macro to generate a boilerplate for Windows service.
///
/// The generated struct contains a helper method to start the service control dispatcher and
/// a private FFI callback (`service_main`) that is invoked by the dispatcher.
///
/// The `service_main` callback parses service arguments provided by the system
/// and passes them with a call to `$service_main_handler`.
///
/// `$struct_name` - name of the struct to produce.
/// `$service_name` - name of the windows service.
/// `$service_main_handler` - the function that's called from `service_main`. Accepts parsed
/// service arguments as `Vec<OsString>`. Its responsibility is to create a
/// `ServiceControlHandler`, start processing control events and report the service status to the
/// system.
///
/// The signature for the `service_main` function is:
///
/// ```
/// fn handle_service_main(arguments: Vec<OsString>)
/// ```
///
macro_rules! define_windows_service {
    ($struct_name:ident, $service_name:ident, $service_main_handler:ident) => {
        struct $struct_name;

        impl $struct_name {
            /// Start service control dispatcher.
            ///
            /// Once started the service control dispatcher blocks the current thread execution
            /// until the service is stopped.
            ///
            /// Upon successful initialization, system calls the `$struct_name::service_main` in
            /// background thread which parses service arguments received from the system and
            /// passes them to higher level `$service_main_handler` handler.
            ///
            /// On failure: immediately returns an error, no threads are spawned.
            ///
            pub fn start_dispatcher() -> ::std::io::Result<()> {
                use winapi::um::winsvc;

                let service_name =
                    unsafe { ::widestring::WideCString::from_str_unchecked($service_name) };
                let service_table: &[winsvc::SERVICE_TABLE_ENTRYW] = &[
                    winsvc::SERVICE_TABLE_ENTRYW {
                        lpServiceName: service_name.as_ptr(),
                        lpServiceProc: Some($struct_name::service_main),
                    },
                    // the last item has to be { null, null }
                    winsvc::SERVICE_TABLE_ENTRYW {
                        lpServiceName: ::std::ptr::null(),
                        lpServiceProc: None,
                    },
                ];

                let result = unsafe { winsvc::StartServiceCtrlDispatcherW(service_table.as_ptr()) };
                if result == 0 {
                    Err(::std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            }

            /// Static callback used by the system to bootstrap the service.
            ///
            /// Note: this function is private and can be mangled by Rust.
            /// To my understanding this is normal since everything happens
            /// within the same process and we point to the address
            /// when calling StartServiceCtrlDispatcherW.
            #[allow(dead_code)]
            extern "system" fn service_main(argc: u32, argv: *mut *mut u16) {
                let arguments = unsafe { Self::parse_raw_arguments(argc, argv) };

                $service_main_handler(arguments);
            }

            /// Parse an unsafe array of raw string pointers received in
            /// `$struct_name::service_main` from the system.
            unsafe fn parse_raw_arguments(
                argc: u32,
                argv: *mut *mut u16,
            ) -> Vec<::std::ffi::OsString> {
                (0..argc)
                    .into_iter()
                    .map(|i| {
                        let array_element_ptr: *mut *mut u16 = argv.offset(i as isize);
                        ::widestring::WideCStr::from_ptr_str(*array_element_ptr).to_os_string()
                    })
                    .collect()
            }
        }
    }
}
