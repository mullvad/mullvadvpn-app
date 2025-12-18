use tracing_subscriber::filter::LevelFilter;

const LOG_FILENAME: &str = "mullvad-loader.log";
const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

pub fn init() {
    let file_appender = tracing_appender::rolling::never(log_dir.unwrap(), DAEMON_LOG_FILENAME);
    let (non_blocking_file_appender, _file_appender_guard) =
        tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(non_blocking_file_appender)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
            DATE_TIME_FORMAT_STR.to_string(),
        ))
        .init()
}
