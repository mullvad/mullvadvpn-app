use tokio::runtime;

pub fn new_runtime_builder() -> runtime::Builder {
    let mut builder = runtime::Builder::new();
    builder
        .threaded_scheduler()
        .core_threads(4)
        .max_threads(8)
        .enable_all();
    builder
}
