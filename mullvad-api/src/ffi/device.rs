use mullvad_types::device::Device;
use std::{ffi::CString, ptr};

#[repr(C)]
pub struct MullvadApiDeviceIterator {
    ptr: *mut DeviceIterator,
}

impl MullvadApiDeviceIterator {
    pub fn new(iter: impl IntoIterator<Item = Device> + 'static) -> Self {
        let iter = Box::new(DeviceIterator::from(iter));

        Self {
            ptr: Box::into_raw(iter),
        }
    }

    fn is_done(&self) -> bool {
        self.ptr.is_null()
    }

    unsafe fn as_iter(&mut self) -> &mut Box<dyn Iterator<Item = Device>> {
        let wrapper = unsafe { &mut *self.ptr };
        &mut wrapper.iter
    }

    fn drop(mut self) {
        if self.ptr.is_null() {
            return;
        }

        let _ = unsafe { Box::from_raw(self.ptr) };
        self.ptr = ptr::null_mut();
    }
}

#[repr(C)]
pub struct MullvadApiDevice {
    name_ptr: *const libc::c_char,
    id: [u8; 16],
}

impl From<Device> for MullvadApiDevice {
    fn from(dev: Device) -> Self {
        let name = CString::new(dev.name).expect("Null bytes in name from API response");
        let name_ptr = name.into_raw();
        let id = *uuid::Uuid::parse_str(&dev.id)
            .expect("Failed to parse UUID")
            .as_bytes();

        Self { name_ptr, id }
    }
}

impl MullvadApiDevice {
    fn drop(self) {
        let _ = unsafe { CString::from_raw(self.name_ptr as *mut _) };
    }
}

struct DeviceIterator {
    iter: Box<dyn Iterator<Item = Device>>,
}

impl<T> From<T> for DeviceIterator
where
    T: IntoIterator<Item = Device> + 'static,
{
    fn from(i: T) -> Self {
        let iter: Box<dyn Iterator<Item = Device>> = Box::new(i.into_iter());
        Self { iter }
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_device_iter_next(
    mut iter: MullvadApiDeviceIterator,
    device_ptr: *mut MullvadApiDevice,
) -> bool {
    if iter.is_done() {
        return false;
    }

    // SAFETY: Asuming self.ptr is still valid since iter.is_done() returned false;
    let iter = unsafe { iter.as_iter() };
    let Some(device) = iter.next() else {
        return false;
    };

    // SAFETY: Assuming device pointer is valid
    unsafe { ptr::write(device_ptr, device.into()) }

    return true;
}

#[no_mangle]
pub extern "C" fn mullvad_api_device_iter_drop(iter: MullvadApiDeviceIterator) {
    iter.drop()
}

#[no_mangle]
pub extern "C" fn mullvad_api_device_drop(device: MullvadApiDevice) {
    device.drop()
}
