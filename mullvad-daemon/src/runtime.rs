use std::num::NonZero;
use tokio::runtime;

const MIN_NUM_THREADS: NonZero<usize> = unsafe { NonZero::new_unchecked(4) };

pub fn new_multi_thread() -> runtime::Builder {
    let mut builder = runtime::Builder::new_multi_thread();
    match std::thread::available_parallelism() {
        Ok(num_cpus) if num_cpus < MIN_NUM_THREADS => {
            builder.worker_threads(num_cpus.into());
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
