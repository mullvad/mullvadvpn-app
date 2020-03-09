use tokio::runtime::{Builder, Runtime};

/// Creates a new tokio event loop on a new thread, runs the provided `init` closure on the thread
/// and sends back the result.
/// Used to spawn futures on the core in the separate thread and be able to return sendable handles.
pub fn create_runtime() -> Runtime {
    let runtime = Builder::new()
        .threaded_scheduler()
        .core_threads(2)
        .enable_all()
        .thread_name("mullvad-rpc-event-loop")
        .build();

    runtime.expect("Failed to initialize mullvad-rpc tokio runtime")
}
