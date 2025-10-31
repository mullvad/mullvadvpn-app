use tracing_subscriber::EnvFilter;

fn main() {
    // Pretty-print to stderr; control verbosity with RUST_LOG
    let sub = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_writer(std::io::stderr)
        .finish();

    // replay(LOG_PATH, sub)
}
