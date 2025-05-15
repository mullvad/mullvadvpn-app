use std::num::NonZero;
use tokio::runtime;

const MIN_NUM_THREADS: NonZero<usize> = NonZero::new(4).unwrap();

pub fn new_multi_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_multi_thread();
    match std::thread::available_parallelism() {
        Ok(num_cpus) if num_cpus < MIN_NUM_THREADS => {
            builder.worker_threads(MIN_NUM_THREADS.into());
        }
        // Use default number of workers
        Ok(_) => (),
        Err(error) => {
            log::warn!("Failed to retrieve number of CPU cores: {error}");
        }
    }
    builder.enable_all();
    builder
}

pub fn new_current_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_current_thread();
    builder.enable_all();
    builder
}
