#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use installer_downloader::controller;
use installer_downloader::delegate;
use installer_downloader::resource;

#[cfg(target_os = "windows")]
mod winapi_impl;

#[cfg(target_os = "macos")]
mod cacao_impl;

fn main() {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");
        let _guard = rt.enter();

        #[cfg(target_os = "windows")]
        winapi_impl::main();

        #[cfg(target_os = "macos")]
        cacao_impl::main();
    }
}
