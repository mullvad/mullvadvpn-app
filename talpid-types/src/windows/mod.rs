use core::ffi::c_void;
use core::ptr::NonNull;

use windows_sys::Win32::Foundation::LocalFree;

/// Frees a local-heap allocation (via LocalFree) on drop.
pub struct LocalFreeGuard<T>(NonNull<T>);

impl<T> LocalFreeGuard<T> {
    /// Create a guard for an object that's destroyed with `LocalFree`
    /// when this is dropped.
    ///
    /// # Safety
    ///
    /// It must be correct to free `p` with `LocalFree`. This function transfers ownership of
    /// `p` to this object.
    pub unsafe fn from_ptr(p: NonNull<T>) -> Self {
        LocalFreeGuard(p)
    }

    /// Return the underlying pointer.
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}

impl<T> Drop for LocalFreeGuard<T> {
    fn drop(&mut self) {
        // SAFETY: `self.0` was allocated by a Win32 function that returns
        // a LocalAlloc-style handle (e.g. `SetEntriesInAclW`); we own it
        // uniquely and have not freed it.
        unsafe { LocalFree(self.0.as_ptr().cast::<c_void>()) };
    }
}
