use std::mem;

/// Casts a struct to a slice of possibly uninitialized bytes.
#[cfg(target_os = "windows")]
pub fn as_uninit_byte_slice<T: Copy + Sized>(value: &T) -> &[mem::MaybeUninit<u8>] {
    unsafe { std::slice::from_raw_parts(value as *const _ as *const _, mem::size_of::<T>()) }
}
