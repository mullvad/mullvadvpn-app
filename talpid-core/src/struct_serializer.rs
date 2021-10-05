use std::mem;

/// Casts a reference to a `T` to a slice of bytes of the same length.
///
/// # Safety
/// It's only safe to call this function if `T` struct is not padded.
#[cfg(target_os = "linux")]
pub unsafe fn as_byte_slice<T: Copy>(value: &T) -> &[u8] {
    // SAFETY: the caller is responsible for not using this iwth structs containing padding
    std::slice::from_raw_parts(value as *const _ as *const u8, mem::size_of::<T>())
}

/// Casts a struct to a slice of possibly uninitialized bytes.
#[cfg(target_os = "windows")]
pub fn as_uninit_byte_slice<T: Copy + Sized>(value: &T) -> &[mem::MaybeUninit<u8>] {
    unsafe { std::slice::from_raw_parts(value as *const _ as *const _, mem::size_of::<T>()) }
}
