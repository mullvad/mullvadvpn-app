use std::{
    mem::swap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;

use crate::api_client::{ApiContext, completion::CompletionCookie, retry_strategy::RetryStrategy};

use super::{completion::SwiftCompletionHandler, response::SwiftMullvadApiResponse};

// TODO FIX THIS
#[derive(uniffi::Object)]
pub struct SwiftCancelHandle {
    inner: Mutex<Option<RequestCancelHandle>>,
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
        api_context: Arc<ApiContext>,
        retry_strategy: Arc<RetryStrategy>,
        task: I,
    ) -> Self
    where
        I: FnOnce(Arc<ApiContext>, RetryStrategy, SwiftCompletionHandler) -> F + Send + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            inner: HandleState::ToStart {
                // SAFETY: See notes for `into_rust`
                retry_strategy: *retry_strategy,
                api_context,
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
            inner: Mutex::new(Some(self)),
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
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is safe
/// because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this type is
/// intended to be used.
#[uniffi::export]
pub fn mullvad_api_start_task(handle: &SwiftCancelHandle, completion_cookie: u64) {
    let completion_cookie = completion_cookie as *mut libc::c_void;
    // SAFETY: It is safe to call CompletionCookie::new with a valid completion cookie
    let completion =
        SwiftCompletionHandler::new(unsafe { CompletionCookie::new(completion_cookie) });
    let Ok(mut handle) = handle.inner.lock() else {
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
#[uniffi::export]
pub fn mullvad_api_cancel_task(handle: &SwiftCancelHandle) {
    // SAFETY: See notes for `as_handle`
    let Ok(mut handle) = handle.inner.lock() else {
        return;
    };
    if let Some(handle) = handle.take() {
        handle.cancel();
    }
}
