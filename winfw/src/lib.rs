//! Windows firewall for the Mullvad app.
//!
//! This crate is a work in progress: it currently only contains the WFP object GUIDs
//! shared between the C++ `winfw.dll` implementation and its Rust replacement, plus
//! the C FFI needed to drive the C++ implementation from the snapshot tests while the
//! Rust implementation is being written.
//!
//! The goal is for this crate to fully replace the C++ code in `windows/winfw`.

#![cfg(target_os = "windows")]

pub mod cpp_ffi;
pub mod guids;
