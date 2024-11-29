use tokio::runtime;

pub fn new_multi_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_multi_thread();

    // When using boringtun, network I/O will happen on tokio threads.
    // in that case, we want all the threads. In other cases, limit thread count to 4.
    if cfg!(not(feature = "boringtun")) {
        builder.worker_threads(4);
    }

    builder.enable_all();
    builder
}

pub fn new_current_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_current_thread();
    builder.enable_all();
    builder
}
