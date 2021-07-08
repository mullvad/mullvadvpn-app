use tokio::runtime;

pub fn new_runtime_builder() -> runtime::Builder {
    let mut builder = runtime::Builder::new_multi_thread();
    builder.worker_threads(4).enable_all();
    builder
}
