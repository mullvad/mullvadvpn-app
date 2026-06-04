use tracing_subscriber::filter::{EnvFilter, LevelFilter};

const LOG_FILENAME: &str = "mullvad-loader.log";
const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

pub fn init() {
    let Ok(log_dir) = mullvad_paths::frontend_log_dir() else {
        return;
    };
    let file_appender = tracing_appender::rolling::never(log_dir, LOG_FILENAME);
    let (non_blocking_file_appender, _file_appender_guard) =
        tracing_appender::non_blocking(file_appender);

    let log_level = EnvFilter::from_default_env().add_directive(LevelFilter::DEBUG.into());

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_writer(non_blocking_file_appender)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
            DATE_TIME_FORMAT_STR.to_string(),
        ))
        .init()
}
