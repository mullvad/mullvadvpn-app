#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use installer_downloader::controller;

#[cfg(target_os = "macos")]
mod cacao_impl;

#[cfg(target_os = "windows")]
mod winapi_impl;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");
    let _guard = runtime.enter();

    #[cfg(target_os = "macos")]
    cacao_impl::main();
    #[cfg(target_os = "windows")]
    winapi_impl::main();
}
