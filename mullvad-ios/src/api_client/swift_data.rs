#[repr(C)]
pub struct SwiftData {
    ptr: *mut libc::c_void,
}

const EMPTY: [u8; 0] = [];

impl AsRef<[u8]> for SwiftData {
    // SAFETY: swift_data_get_{ptr,len} are synchronous deterministic functions
    // that return a pointer/length or a null/0 value in all cases.
    fn as_ref(&self) -> &[u8] {
        let data_ptr = unsafe { swift_data_get_ptr(self) };
        let len = unsafe { swift_data_get_len(self) };

        if data_ptr.is_null() {
            &EMPTY
        } else {
            unsafe { std::slice::from_raw_parts(data_ptr, len) }
        }
    }
}

impl Drop for SwiftData {
    // SAFETY: swift_data_drop is a deterministic function that releases and
    // zeroes the pointer to any data held, if present. It is idempotent.
    fn drop(&mut self) {
        unsafe { swift_data_drop(self) }
    }
}

unsafe extern "C" {
    fn swift_data_get_ptr(data: &SwiftData) -> *mut u8;
    fn swift_data_get_len(data: &SwiftData) -> usize;
    fn swift_data_drop(data: &mut SwiftData);
}
