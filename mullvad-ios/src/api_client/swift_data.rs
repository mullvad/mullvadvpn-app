#[repr(C)]
pub struct SwiftData {
    ptr: *mut libc::c_void,
}

const EMPTY: [u8;0] = [];

impl AsRef<[u8]> for SwiftData {
    fn as_ref(&self) -> &[u8] {
        let data_ptr = unsafe { swift_data_get_ptr(self) };
        let len = unsafe { swift_data_get_len(self)  };

        if data_ptr.is_null() {
            &EMPTY
        } else {
            unsafe {
                std::slice::from_raw_parts(data_ptr, len)
            }
        }
    }
}

impl Drop for SwiftData {
    fn drop(&mut self) {
        unsafe { swift_data_drop(self) }
    }
}

unsafe extern "C" {
    fn swift_data_get_ptr(data: &SwiftData) -> *mut u8;
    fn swift_data_get_len(data: &SwiftData) -> usize;
    fn swift_data_drop(data: &mut SwiftData);
}