use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

const LOG_FILENAME: &str = "mullvad-loader.log";
const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

pub fn init() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = mullvad_paths::frontend_log_dir().ok()?;

    let file_appender = tracing_appender::rolling::never(log_dir, LOG_FILENAME);
    let (non_blocking_file_appender, file_appender_guard) =
        tracing_appender::non_blocking(file_appender);

    // In debug mode, also log to stdout
    if cfg!(debug_assertions) {
        let stdout_layer = fmt::layer()
            .with_ansi(true)
            .with_writer(std::io::stdout)
            .with_filter(LevelFilter::DEBUG);

        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender)
            .with_timer(fmt::time::ChronoUtc::new(DATE_TIME_FORMAT_STR.to_string()))
            .with_filter(LevelFilter::DEBUG);
        registry()
            .with(stdout_layer)
            .with(file_layer)
            .try_init()
            .ok()?;
    } else {
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender)
            .with_timer(fmt::time::ChronoUtc::new(DATE_TIME_FORMAT_STR.to_string()))
            .with_filter(LevelFilter::DEBUG);
        registry().with(file_layer).try_init().ok()?;
    }

    Some(file_appender_guard)
}
