#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod controller;
mod fetch;
mod verify;

#[cfg(target_os = "macos")]
mod cacao_impl;

#[cfg(target_os = "windows")]
mod libui_impl;

#[cfg(target_os = "macos")]
fn main() {
    #[cfg(target_os = "macos")]
    cacao_impl::main();
    #[cfg(target_os = "windows")]
    libui_impl::main();
}
