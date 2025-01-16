use std::ptr::null_mut;

use tokio::task::JoinHandle;

use super::{completion::SwiftCompletionHandler, response::SwiftMullvadApiResponse};

#[repr(C)]
pub struct SwiftCancelHandle {
    ptr: *mut RequestCancelHandle,
}

impl SwiftCancelHandle {
    pub fn empty() -> Self {
        Self { ptr: null_mut() }
    }

    /// This consumes and nulls out the pointer. The caller is responsible for the pointer being valid
    /// when calling `to_handle`.
    unsafe fn into_handle(mut self) -> RequestCancelHandle {
        // # Safety
        // This call is safe as long as the pointer is only ever used from a single thread and the
        // instance of `SwiftCancelHandle` was created with a valid pointer to
        // `RequestCancelHandle`.
        let handle = unsafe { *Box::from_raw(self.ptr) };
        self.ptr = null_mut();

        handle
    }
}

pub struct RequestCancelHandle {
    task: JoinHandle<()>,
    completion: SwiftCompletionHandler,
}

impl RequestCancelHandle {
    pub fn new(task: JoinHandle<()>, completion: SwiftCompletionHandler) -> Self {
        Self { task, completion }
    }

    pub fn into_swift(self) -> SwiftCancelHandle {
        SwiftCancelHandle {
            ptr: Box::into_raw(Box::new(self)),
        }
    }

    pub fn cancel(self) {
        let Self { task, completion } = self;
        task.abort();
        // TODO: should this call block until the task returns?
        // We can make it do that.
        // let _ = handle.block_on(self.task);
        completion.finish(SwiftMullvadApiResponse::cancelled());
    }
}

/// Called by the Swift side to signal that a Mullvad API call should be cancelled.
/// After this call, the cancel token is no longer valid.
///
/// # Safety
///
/// `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`. This function
/// is not safe to call multiple times with the same `SwiftCancelHandle`.
#[no_mangle]
extern "C" fn mullvad_api_cancel_task(handle_ptr: SwiftCancelHandle) {
    if handle_ptr.ptr.is_null() {
        return;
    }

    let handle = unsafe { handle_ptr.into_handle() };
    handle.cancel()
}

/// Called by the Swift side to signal that the Rust `SwiftCancelHandle` can be safely
/// dropped from memory.
///
/// # Safety
///
/// `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`. This function
/// is not safe to call multiple times with the same `SwiftCancelHandle`.
#[no_mangle]
extern "C" fn mullvad_api_cancel_task_drop(handle_ptr: SwiftCancelHandle) {
    if handle_ptr.ptr.is_null() {
        return;
    }

    let _handle = unsafe { handle_ptr.into_handle() };
}
