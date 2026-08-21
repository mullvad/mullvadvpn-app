use std::{
    mem::swap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;

use crate::api_client::{ApiContext, retry_strategy::RetryStrategy};

use super::response::SwiftMullvadApiResponse;

#[derive(uniffi::Object)]
pub struct RequestCancelHandle {
    inner: Mutex<Option<HandleState>>,
}

impl RequestCancelHandle {
    pub fn new<I, F>(
        api_context: Arc<ApiContext>,
        retry_strategy: Arc<RetryStrategy>,
        task: I,
    ) -> Arc<Self>
    where
        I: FnOnce(Arc<ApiContext>, RetryStrategy, Arc<dyn CompletionCookieNew>) -> F
            + Send
            + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let state = HandleState::ToStart {
            // SAFETY: See notes for `into_rust`
            retry_strategy: *retry_strategy,
            api_context,
            task: Box::new(move |a, r, c| Box::pin(task(a, r, c))),
        };
        Arc::new(Self {
            inner: Mutex::new(Some(state)),
        })
    }
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
                    Arc<dyn CompletionCookieNew>,
                ) -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    },
    // This is used in `start` to safetly swap out the state.
    // A RequestCancelHandle method should never return and leave the state as this.
    Intermediate,
    Started {
        task: JoinHandle<()>,
        completion: Arc<dyn CompletionCookieNew>,
    },
}

impl HandleState {
    pub fn start(&mut self, completion: Arc<dyn CompletionCookieNew>) {
        if !matches!(self, HandleState::ToStart { .. }) {
            return;
        }
        let mut data = HandleState::Intermediate;
        swap(&mut data, self);
        let HandleState::ToStart {
            api_context,
            retry_strategy,
            task,
        } = data
        else {
            return;
        };

        let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
            completion.finish(Arc::new(SwiftMullvadApiResponse::no_tokio_runtime()));
            return;
        };

        let task = task(api_context, retry_strategy, completion.clone());
        let handle = tokio_handle.spawn(task);

        *self = HandleState::Started {
            task: handle,
            completion,
        }
    }

    pub fn cancel(self) {
        let HandleState::Started { task, completion } = self else {
            return;
        };
        task.abort();
        // TODO: should this call block until the task returns?
        // We can make it do that.
        // let _ = handle.block_on(self.task);
        completion.finish(Arc::new(SwiftMullvadApiResponse::cancelled()));
    }
}

#[uniffi::export(with_foreign)]
pub trait CompletionCookieNew: Send + Sync {
    fn finish(&self, result: Arc<SwiftMullvadApiResponse>);
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
pub fn mullvad_api_start_task(
    handle: &RequestCancelHandle,
    completion_cookie: Arc<dyn CompletionCookieNew>,
) {
    let Ok(mut handle) = handle.inner.lock() else {
        return;
    };
    if let Some(handle) = &mut *handle {
        handle.start(completion_cookie);
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
pub fn mullvad_api_cancel_task(handle: &RequestCancelHandle) {
    // SAFETY: See notes for `as_handle`
    let Ok(mut handle) = handle.inner.lock() else {
        return;
    };
    if let Some(handle) = handle.take() {
        handle.cancel();
    }
}
