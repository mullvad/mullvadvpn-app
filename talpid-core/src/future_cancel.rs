use futures::{channel::oneshot, FutureExt};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct CancelOnDrop(Option<CancelHandle>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.cancel();
        }
    }
}

impl From<CancelHandle> for CancelOnDrop {
    fn from(handle: CancelHandle) -> Self {
        Self(Some(CancelHandle))
    }
}


#[derive(Debug)]
/// An error returned when a future is cancelled
pub struct CancelErr(());

/// A future that can be cancelled
pub struct Cancellable<F: Future> {
    rx: oneshot::Receiver<()>,
    f: F,
}

/// Handle for cancelling a future
pub struct CancelHandle {
    tx: oneshot::Sender<()>,
}

impl CancelHandle {
    /// Cancel corresponding future
    pub fn cancel(self) {
        let _ = self.tx.send(());
    }
}

impl<F> Cancellable<F>
where
    F: Future,
{
    /// Wraps a future to make it cancellable and returns a handle to cancel it remotely
    pub fn new(f: F) -> (Self, CancelHandle) {
        let (tx, rx) = oneshot::channel();
        (Self { f, rx }, CancelHandle { tx })
    }

    // /// askm
    // pub async fn into_future(self) -> std::result::Result<F::Output, CancelErr> {
    //     futures::select! {
    //         _cancelled = self.rx.fuse() => {
    //             Err(CancelErr(()))
    //         },
    //         value = self.f.fuse() => {
    //             Ok(value)
    //         }
    //     }
    // }
}

impl<F: Future<Output = T> + Unpin, T: Unpin> Future for Cancellable<F> {
    type Output = std::result::Result<T, CancelErr>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.get_mut();

        if let Poll::Ready(ready) = inner.f.poll_unpin(cx) {
            return Poll::Ready(Ok(ready));
        }

        if let Poll::Ready(_) = inner.rx.poll_unpin(cx) {
            return Poll::Ready(Err(CancelErr(())));
        }

        Poll::Pending
    }
}
