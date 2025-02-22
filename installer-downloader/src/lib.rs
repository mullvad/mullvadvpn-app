#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod controller;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod delegate;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod resource;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod ui_downloader;
