#[repr(C)]
pub struct SwiftData {
    ptr: *mut libc::c_void,
}

const EMPTY: [u8; 0] = [];

impl AsRef<[u8]> for SwiftData {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: swift_data_get_ptr is a synchronous deterministic functions
        // that return a pointer to the data contained in the NSData object
        // contained in self, or null if self does not contain data.
        // Any failure to reach a valid NSData will yield a null result.
        let data_ptr = unsafe { swift_data_get_ptr(self) };
        // SAFETY: swift_data_get_len} is a synchronous deterministic function
        // that return the length of the data contained in self, or 0 if there
        // is none.  Any failure to reach a valid NSData will yield 0.
        let len = unsafe { swift_data_get_len(self) };

        if data_ptr.is_null() {
            &EMPTY
        } else {
            // SAFETY: we know that data_ptr points to len u8s
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
