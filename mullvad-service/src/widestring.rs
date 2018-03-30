use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;

/// Converts string to widechar vector with nul terminator
pub fn to_wide_with_nul<T: AsRef<OsStr>>(os_string: T) -> Vec<u16> {
    os_string
        .as_ref()
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

/// Converts a raw nul terminated widechar string pointer to OsString
pub unsafe fn from_raw_wide_string(raw_wide_char_string: *mut u16, max_length: usize) -> OsString {
    let nul_position = (0..max_length)
        .into_iter()
        .find(|&i| {
            let wchar = raw_wide_char_string.offset(i as isize);
            *wchar == 0
        })
        .unwrap_or(max_length);

    if nul_position > 0 {
        let mut buf: Vec<u16> = Vec::with_capacity(nul_position);
        buf.set_len(nul_position);
        ptr::copy(raw_wide_char_string, buf.as_mut_ptr(), nul_position);
        OsString::from_wide(&buf)
    } else {
        OsString::new()
    }
}
