use std::{
    mem::swap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;

use crate::api_client::{
    ApiContext, SwiftApiContext,
    completion::CompletionCookie,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
};

use super::{completion::SwiftCompletionHandler, response::SwiftMullvadApiResponse};

// `cbindgen` does not handle structs with generic fields, so this is used to hide that
struct SwiftCancelHandleInner(Mutex<Option<RequestCancelHandle>>);

#[repr(C)]
pub struct SwiftCancelHandle {
    ptr: *mut SwiftCancelHandleInner,
}

impl SwiftCancelHandle {
    /// This consumes and nulls out the pointer. The caller is responsible for the pointer being valid
    /// when calling `to_handle`.
    ///
    /// SAFETY:
    /// This call is safe as long as the instance of `SwiftCancelHandle` was created with a valid pointer to
    /// `RequestCancelHandle`, and `mullvad_api_cancel_task_drop` has not been called on this instace.
    unsafe fn as_handle(&self) -> Option<RequestCancelHandle> {
        // SAFETY: See safety notes above
        let handle = unsafe { &*self.ptr };
        let Ok(mut guard) = handle.0.lock() else {
            return None;
        };
        guard.take()
    }
}

pub struct RequestCancelHandle {
    inner: HandleState,
}

#[expect(clippy::type_complexity)]
enum HandleState {
    ToStart {
        api_context: Arc<ApiContext>,
        retry_strategy: RetryStrategy,
        task: Box<
            dyn FnOnce(
                    Arc<ApiContext>,
                    RetryStrategy,
                    SwiftCompletionHandler,
                ) -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    },
    // This is used in `start` to safetly swap out the state.
    // A RequestCancelHandle method should never return and leave the state as this.
    Intermediate,
    Started {
        task: JoinHandle<()>,
        completion: SwiftCompletionHandler,
    },
}

impl RequestCancelHandle {
    pub fn new<I, F>(
        api_context: SwiftApiContext,
        retry_strategy: SwiftRetryStrategy,
        task: I,
    ) -> Self
    where
        I: FnOnce(Arc<ApiContext>, RetryStrategy, SwiftCompletionHandler) -> F + Send + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            inner: HandleState::ToStart {
                // SAFETY: See notes for `into_rust`
                retry_strategy: unsafe { retry_strategy.into_rust() },
                api_context: api_context.rust_context(),
                task: Box::new(move |a, r, c| Box::pin(task(a, r, c))),
            },
        }
    }

    pub fn start(&mut self, completion: SwiftCompletionHandler) {
        if !matches!(self.inner, HandleState::ToStart { .. }) {
            return;
        }
        let mut data = HandleState::Intermediate;
        swap(&mut data, &mut self.inner);
        let HandleState::ToStart {
            api_context,
            retry_strategy,
            task,
        } = data
        else {
            return;
        };

        let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
            completion.finish(SwiftMullvadApiResponse::no_tokio_runtime());
            return;
        };

        let task = task(api_context, retry_strategy, completion.clone());
        let handle = tokio_handle.spawn(task);

        self.inner = HandleState::Started {
            task: handle,
            completion,
        }
    }

    pub fn into_swift(self) -> SwiftCancelHandle {
        SwiftCancelHandle {
            ptr: Box::into_raw(Box::new(SwiftCancelHandleInner(Mutex::new(Some(self))))),
        }
    }

    pub fn cancel(self) {
        let HandleState::Started { task, completion } = self.inner else {
            return;
        };
        task.abort();
        // TODO: should this call block until the task returns?
        // We can make it do that.
        // let _ = handle.block_on(self.task);
        completion.finish(SwiftMullvadApiResponse::cancelled());
    }
}

/// Called by the Swift side to signal that a Mullvad API call should be started.
/// Does nothing on repeated calls.
/// Must not be called after `mullvad_api_cancel_task_drop.`
///
/// # Safety
///
/// `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`.
#[unsafe(no_mangle)]
extern "C" fn mullvad_api_start_task(
    handle_ptr: SwiftCancelHandle,
    completion_cookie: *mut libc::c_void,
) {
    // SAFETY: See safety notes above
    let handle = unsafe { &*handle_ptr.ptr };
    // SAFETY: It is safe to call CompletionCookie::new with a valid completion cookie
    let completion =
        SwiftCompletionHandler::new(unsafe { CompletionCookie::new(completion_cookie) });
    let Ok(mut handle) = handle.0.lock() else {
        return;
    };
    if let Some(handle) = &mut *handle {
        handle.start(completion);
    }
}

/// Called by the Swift side to signal that a Mullvad API call should be cancelled.
/// Does nothing on repeated calls.
/// Must not be called after `mullvad_api_cancel_task_drop.
///
/// # Safety
///
/// `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`.
#[unsafe(no_mangle)]
extern "C" fn mullvad_api_cancel_task(handle_ptr: SwiftCancelHandle) {
    // SAFETY: See notes for `as_handle`
    let handle = unsafe { handle_ptr.as_handle() };
    if let Some(handle) = handle {
        handle.cancel();
    }
}

/// Called by the Swift side to signal that the Rust `SwiftCancelHandle` can be safely
/// dropped from memory.
/// Must be called once, and at most once.
///
/// # Safety
///
/// `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`.
#[unsafe(no_mangle)]
extern "C" fn mullvad_api_cancel_task_drop(handle_ptr: SwiftCancelHandle) {
    // SAFETY: See safety notes above
    let _ptr = unsafe { Box::from_raw(handle_ptr.ptr) };
}
