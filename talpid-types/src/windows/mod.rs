use core::ffi::c_void;
use core::ptr::NonNull;

use windows_sys::Win32::Foundation::LocalFree;

/// Frees a local-heap allocation (via LocalFree) on drop.
pub struct LocalFreeGuard<T> {
    /// The pointer that will be freed when self is dropped.
    ptr_to_free: NonNull<c_void>,

    /// A pointer which points somewhere into the originally allocated object. See [`Self::map`].
    ptr: NonNull<T>,
}

impl<T> LocalFreeGuard<T> {
    /// Create a guard for an object that's destroyed with `LocalFree`
    /// when this is dropped.
    ///
    /// # Safety
    ///
    /// It must be correct to free `p` with `LocalFree`. This function transfers ownership of
    /// `p` to this object.
    pub unsafe fn from_ptr(p: NonNull<T>) -> Self {
        LocalFreeGuard {
            ptr_to_free: p.cast::<c_void>(),
            ptr: p,
        }
    }

    /// Return the underlying pointer. It will be invalidated when this guard is dropped.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub unsafe fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    /// Map the pointer to another.
    ///
    /// This will not change the pointer that is passed to `LocalFree` when we're dropped.
    pub fn map<X>(self, f: impl FnOnce(NonNull<T>) -> NonNull<X>) -> LocalFreeGuard<X> {
        let mapped = LocalFreeGuard {
            ptr_to_free: self.ptr_to_free,
            ptr: f(self.ptr),
        };
        core::mem::forget(self); // do not free the pointer
        mapped
    }
}

impl<T> Drop for LocalFreeGuard<T> {
    fn drop(&mut self) {
        // SAFETY:
        // - We own this pointer uniquely.
        // - Pointer is only freed in this function.
        // - `ptr_to_free` was passed to us with the promise that it can be passed to LocalFree.
        //   This is the case for e.g. pointers allocated by Win32 functions that returns
        //   LocalAlloc-style handles (e.g. `SetEntriesInAclW`)
        unsafe { LocalFree(self.ptr_to_free.as_ptr()) };
    }
}
