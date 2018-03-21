use std::ffi::OsStr;
use std::os::windows::prelude::*;

pub fn to_wide_with_nul<T: AsRef<OsStr>>(os_string: T) -> Vec<u16> {
    os_string.as_ref().encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}