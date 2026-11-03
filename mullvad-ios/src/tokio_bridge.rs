use std::{
    pin::{Pin, pin},
    task::{Context, Poll},
};

use tokio::task::JoinHandle;

pub struct TokioBridge<T>(JoinHandle<T>);
impl<T: Send + 'static> TokioBridge<T> {
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

    #[track_caller]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let token_ref = pin!(&mut self.0);
        match token_ref.poll(cx) {
            Poll::Ready(r) => match r {
                Ok(o) => Poll::Ready(o),
                Err(e) => panic!("task panic'ed: {e}"),
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

#[cfg(test)]
mod tests {
    use super::TokioBridge;
    use std::time::Duration;
    use tokio::sync::mpsc;

    #[test]
    fn cancellation_bridge() {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();

        let bridge = TokioBridge::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            assert!(tx.send(()).is_ok());
        });
        drop(bridge);
        assert!(rx.blocking_recv().is_none());
    }
}
