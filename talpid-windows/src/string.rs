use std::ffi::CStr;
use std::io;
use std::ptr;
use windows_sys::Win32::Globalization::MultiByteToWideChar;

/// Convert `mb_string`, with the given character encoding `codepage`, to a UTF-16 string.
pub fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if mb_string.is_empty() {
        return Ok(vec![]);
    }

    // SAFETY: `mb_string` is null-terminated and valid.
    let wc_size = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            ptr::null_mut(),
            0,
        )
    };

    if wc_size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wc_buffer = vec![0u16; usize::try_from(wc_size).unwrap()];

    // SAFETY: `wc_buffer` can contain up to `wc_size` characters, including a null
    // terminator.
    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            wc_buffer.as_mut_ptr(),
            wc_size,
        )
    };

    if chars_written == 0 {
        return Err(io::Error::last_os_error());
    }

    wc_buffer.truncate(usize::try_from(chars_written - 1).unwrap());

    Ok(wc_buffer)
}

#[cfg(test)]
mod test {
    use super::*;
    use windows_sys::Win32::Globalization::CP_UTF8;

    #[test]
    fn test_multibyte_to_wide() {
        // € = 0x20AC in UTF-16
        let converted = multibyte_to_wide(c"€€", CP_UTF8);
        const EXPECTED: &[u16] = &[0x20AC, 0x20AC];
        assert!(
            matches!(converted.as_deref(), Ok(EXPECTED)),
            "expected Ok({EXPECTED:?}), got {converted:?}",
        );

        // boundary case
        let converted = multibyte_to_wide(c"", CP_UTF8);
        assert!(
            matches!(converted.as_deref(), Ok([])),
            "unexpected result {converted:?}"
        );
    }
}
