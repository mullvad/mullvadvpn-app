/// Macro to generate the windows service struct that includes all what's needed to support the
/// service's lifecycle..
///
/// `$struct_name` - the name of structure to produce
/// `$service_name` - the name of windows service
/// `$service_main` - the `service_main` handler
///
/// The signature for the `service_main` function is:
///
/// ```
/// fn handle_service_main(arguments: Vec<OsString>)
/// ```
///
macro_rules! define_windows_service {
    ($struct_name:ident, $service_name:ident, $service_main:ident) => {
        struct $struct_name;

        impl $struct_name {
            /// Start service control dispatcher.
            ///
            /// Once started the service control dispatcher blocks the current thread execution
            /// until the service is stopped.
            ///
            /// Upon successful initialization, system calls the `$struct_name::service_main` in
            /// background thread which parses raw arguments and passes them to higher level
            /// `$service_main` handler.
            ///
            /// On failure: immediately returns an error, no threads are spawned.
            ///
            pub fn start_dispatcher() -> ::std::io::Result<()> {
                use widestring::to_wide_with_nul;
                use winapi::um::winsvc;

                let service_name = to_wide_with_nul($service_name);
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
                    Err(io::Error::last_os_error())
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

                $service_main(arguments);
            }

            /// Parse an unsafe array of raw string pointers received in `service_main` from system.
            unsafe fn parse_raw_arguments(
                argc: u32,
                argv: *mut *mut u16,
            ) -> Vec<std::ffi::OsString> {
                use widestring::from_raw_wide_string;

                (0..argc)
                    .into_iter()
                    .map(|i| {
                        let ptr = argv.offset(i as isize);
                        from_raw_wide_string(*ptr, 256)
                    })
                    .collect()
            }
        }
    };
}
