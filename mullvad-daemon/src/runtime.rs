use tokio::runtime;

pub fn new_multi_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder
}

pub fn new_current_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_current_thread();
    builder.enable_all();
    builder
}
