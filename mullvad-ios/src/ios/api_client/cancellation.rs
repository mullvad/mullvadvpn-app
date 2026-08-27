use std::{
    mem::swap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;

use crate::api_client::{ApiContext, retry_strategy::RetryStrategy};

use super::response::ApiResponse;

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
        I: FnOnce(Arc<ApiContext>, RetryStrategy, Arc<dyn RequestCompletion>) -> F + Send + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let state = HandleState::ToStart {
            retry_strategy: *retry_strategy,
            api_context,
            task: Box::new(move |a, r, c| Box::pin(task(a, r, c))),
        };
        Arc::new(Self {
            inner: Mutex::new(Some(state)),
        })
    }
}

#[uniffi::export]
impl RequestCancelHandle {
    /// Called by the Swift side to signal that a Mullvad API call should be started.
    /// Does nothing on repeated calls.
    pub fn start_task(&self, completion_cookie: Arc<dyn RequestCompletion>) {
        let Ok(mut handle) = self.inner.lock() else {
            return;
        };
        if let Some(handle) = &mut *handle {
            handle.start(completion_cookie);
        }
    }

    /// Called by the Swift side to signal that a Mullvad API call should be cancelled.
    /// Does nothing on repeated calls.
    pub fn cancel_task(&self) {
        let Ok(mut handle) = self.inner.lock() else {
            return;
        };
        if let Some(handle) = handle.take() {
            handle.cancel();
        }
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
                    Arc<dyn RequestCompletion>,
                ) -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    },
    // This is used in `start` to safetly swap out the state.
    // A RequestCancelHandle method should never return and leave the state as this.
    Intermediate,
    Started {
        task: JoinHandle<()>,
        completion: Arc<dyn RequestCompletion>,
    },
}

impl HandleState {
    pub fn start(&mut self, completion: Arc<dyn RequestCompletion>) {
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
            completion.finish(Arc::new(ApiResponse::no_tokio_runtime()));
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
        completion.finish(Arc::new(ApiResponse::cancelled()));
    }
}

#[uniffi::export(with_foreign)]
pub trait RequestCompletion: Send + Sync {
    fn finish(&self, result: Arc<ApiResponse>);
}
