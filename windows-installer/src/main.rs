#![windows_subsystem = "windows"]

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn main() {}
}

pub use imp::*;
