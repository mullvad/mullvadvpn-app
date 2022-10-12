use windows_sys::{core::GUID, Win32::System::Com::StringFromGUID2};

/// Obtain a string representation for a GUID object.
pub fn string_from_guid(guid: &GUID) -> String {
    let mut buffer = [0u16; 40];
    let length = unsafe { StringFromGUID2(guid, &mut buffer[0] as *mut _, buffer.len() as i32 - 1) }
        as usize;
    // cannot fail because `buffer` is large enough
    assert!(length > 0);
    let length = length - 1;
    String::from_utf16(&buffer[0..length]).unwrap()
}
