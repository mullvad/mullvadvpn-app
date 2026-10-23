use std::{
    pin::{Pin, pin},
    task::{Context, Poll},
};

use tokio::task::JoinHandle;

/// Bridges a future to be run inside of the current Tokio runtime. Used to call and await
/// futures that rely on Tokio from a UniFFI context.
///
/// The future is automatically cancelled on drop, which in most cases will be when a parent task stops being polled.
pub struct TokioBridge<T>(JoinHandle<T>);
impl<T: Send + 'static> TokioBridge<T> {
    #[track_caller]
    pub fn spawn<F: Future<Output = T> + Send + 'static>(future: F) -> Self {
        let runtime = match crate::mullvad_ios_runtime() {
            Ok(o) => o,
            Err(e) => panic!("failed to grab runtime: {e}"),
        };
        let handle = runtime.spawn(future);
        Self(handle)
    }
}
impl<T> Future for TokioBridge<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let token_ref = pin!(&mut self.0);
        match token_ref.poll(cx) {
            Poll::Ready(r) => match r {
                Ok(o) => Poll::Ready(o),
                Err(e) => panic!("task failed: {e}"),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
impl<T> Drop for TokioBridge<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
