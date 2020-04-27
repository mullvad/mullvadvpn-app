use futures::{sync::oneshot, Future};
use std::thread;
use tokio_core::reactor::{Core, Remote};

pub struct CoreHandle {
    /// Remote used to spawn futures on the daemon's event loop.
    pub remote: Remote,
    /// A sender that will cause the event loop to stop once it's dropped.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for CoreHandle {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if shutdown_tx.send(()).is_err() {
                log::error!("Core already shut down");
            }
        }
    }
}

/// Panics if a new tokio event loop can't be spawned.
pub fn spawn() -> CoreHandle {
    let (tx, rx) = oneshot::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    thread::spawn(move || {
        let mut core = Core::new().expect("Failed to spawn event loop");
        let remote = core.remote();
        let _ = tx.send(remote);
        let _ = core.run(shutdown_rx);
    });
    let remote = rx.wait().expect("Failed to spawn event loop");

    CoreHandle {
        remote,
        shutdown_tx: Some(shutdown_tx),
    }
}
