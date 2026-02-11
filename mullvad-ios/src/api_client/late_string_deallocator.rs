use libc::c_char;

#[repr(C)]
pub struct LateStringDeallocator {
    pub(crate) ptr: *const c_char,
    deallocate_ptr: unsafe extern "C" fn(*const c_char),
}
