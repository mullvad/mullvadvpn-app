#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod winapi_impl;

#[cfg(target_os = "macos")]
mod cacao_impl;

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod inner {
    pub use installer_downloader::controller;
    pub use installer_downloader::delegate;
    pub use installer_downloader::log;
    pub use installer_downloader::resource;

    pub fn run() {
        // Independently if log::init() succeed or fails, this value should be dropped last to
        // ensure that every log statement is flushed.
        let _log_flush_guard = log::init();
        // Allow logging of sensitive values in debug builds.
        let _safelog_guard = if cfg!(debug_assertions) {
            safelog::disable_safe_logging().ok()
        } else {
            None
        };

        ::log::debug!("Installer downloader version: {}", resource::VERSION);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");
        let _guard = rt.enter();

        cfg_select! {
            target_os = "windows" => { super::winapi_impl::main(); }
            target_os = "macos"   => { super::cacao_impl::main(); }
        }
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
