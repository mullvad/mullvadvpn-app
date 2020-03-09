use tokio::runtime::{Builder, Runtime};

/// Creates a new tokio runtime to be exclusively used for HTTP requests.
// FIXME: Remove this once the daemon has migrated.
pub fn create_runtime() -> Result<Runtime, crate::Error> {
    let runtime = Builder::new()
        .threaded_scheduler()
        .core_threads(2)
        .enable_all()
        .thread_name("mullvad-rpc-event-loop")
        .build();

    runtime.map_err(crate::Error::TokioRuntimeError)
}
