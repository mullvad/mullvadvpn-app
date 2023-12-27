use std::marker::PhantomData;

#[repr(C)]
pub struct SwiftDataArray {
    array_ptr: *mut libc::c_void,
}

impl SwiftDataArray {
    pub unsafe fn from_ptr(array_ptr: *mut libc::c_void) -> SwiftDataArray {
        Self { array_ptr }
    }

    pub fn new() -> Self {
        Self {
            array_ptr: unsafe { swift_data_array_create() },
        }
    }

    pub fn drain(&mut self) -> Self {
        let old_ptr = self.array_ptr;
        self.array_ptr = unsafe { swift_data_array_create() };

        Self { array_ptr: old_ptr }
    }

    pub fn len(&self) -> usize {
        unsafe { swift_data_array_len(self.array_ptr) }
    }

    pub fn append(&mut self, data: &[u8]) {
        let size = data.len();
        let raw_ptr = data.as_ptr();

        unsafe {
            swift_data_array_append(self.array_ptr, raw_ptr, size);
        }
    }

    pub fn get_mut<'a>(&mut self, idx: usize) -> Option<SwiftDataWrapper<'a>> {
        if idx >= self.len() {
            return None;
        }
        let data = unsafe { swift_data_array_get(self.array_ptr, idx) };
        let wrapper = SwiftDataWrapper {
            data,
            _marker: PhantomData,
        };
        Some(wrapper)
    }

    pub fn iter<'a>(&'a mut self) -> SwiftDataArrayIterator<'a> {
        SwiftDataArrayIterator {
            array: self,
            idx: 0,
        }
    }

    pub fn into_raw(self) -> *mut libc::c_void {
        let ptr = self.array_ptr;
        std::mem::forget(self);
        ptr
    }

    pub unsafe fn from_raw(array_ptr: *mut libc::c_void) -> Self {
        Self { array_ptr }
    }
}

impl Drop for SwiftDataArray {
    fn drop(&mut self) {
        unsafe { swift_data_array_drop(self.array_ptr) }
    }
}

pub struct SwiftDataArrayIterator<'a> {
    array: &'a mut SwiftDataArray,
    idx: usize,
}

impl<'a> Iterator for SwiftDataArrayIterator<'a> {
    type Item = SwiftDataWrapper<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.array.get_mut(self.idx)?;
        self.idx += 1;
        Some(next)
    }
}

extern "C" {
    fn swift_data_array_create() -> *mut libc::c_void;
    fn swift_data_array_append(swift_data_ptr: *mut libc::c_void, data: *const u8, data_len: usize);
    fn swift_data_array_drop(swift_data_ptr: *mut libc::c_void);
    fn swift_data_array_get(swift_data_array_ptr: *mut libc::c_void, idx: usize) -> SwiftData;
    fn swift_data_array_len(swift_data_array_ptr: *mut libc::c_void) -> usize;
}

pub struct SwiftDataWrapper<'a> {
    data: SwiftData,
    _marker: PhantomData<&'a ()>,
}

impl<'a> AsMut<[u8]> for SwiftDataWrapper<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        // SAFETY: `self.bytes_ptr` must be valid for `self.size` bytes
        unsafe { std::slice::from_raw_parts_mut(self.data.ptr, self.data.len) }
    }
}

#[repr(C)]
struct SwiftData {
    ptr: *mut u8,
    len: usize,
}

#[no_mangle]
pub extern "C" fn swift_data_array_test() -> *mut libc::c_void {
    let mut arr = SwiftDataArray::new();
    arr.append(&[1, 2, 3]);
    arr.append(&[1, 2, 3]);
    arr.append(&[1, 2, 3]);
    return arr.into_raw();
}
