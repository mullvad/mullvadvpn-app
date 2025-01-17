#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod winapi_impl;

#[cfg(target_os = "macos")]
mod cacao_impl;

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod inner {
    pub use installer_downloader::controller;
    pub use installer_downloader::delegate;
    pub use installer_downloader::resource;

    pub fn run() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");
        let _guard = rt.enter();

        #[cfg(target_os = "windows")]
        super::winapi_impl::main();

        #[cfg(target_os = "macos")]
        super::cacao_impl::main();
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod inner {
    pub fn run() {}
}

use inner::*;

fn main() {
    run()
}
