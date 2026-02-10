use mullvad_logging::{EnvFilter, LevelFilter, silence_crates};
use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use talpid_core::logging::rotate_log;
use tracing_appender::non_blocking;
use tracing_subscriber::{
    Registry,
    fmt::{MakeWriter, format::FmtSpan, writer::OptionalWriter},
    layer::SubscriberExt,
    reload::Handle,
    util::SubscriberInitExt,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unable to open log file for writing
    #[error("Unable to open log file for writing: {path}")]
    WriteFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Unable to rotate daemon log file")]
    RotateLog(#[from] talpid_core::logging::RotateLogError),
}

/// A [`MakeWriter`] that wraps an [`OptionalWriter`].
struct OptionalMakeWriter<T>(Option<T>);

impl<'a, T: Clone + io::Write> MakeWriter<'a> for OptionalMakeWriter<T> {
    type Writer = OptionalWriter<T>;

    fn make_writer(&'a self) -> Self::Writer {
        match &self.0 {
            Some(writer) => OptionalWriter::some(writer.clone()),
            None => OptionalWriter::none(),
        }
    }
}

const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

/// Whether a [log] logger has been initialized.
// the log crate doesn't provide a nice way to tell if a logger has been initialized :(
static LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check whether logging has been enabled, i.e. if [init_logger] has been called successfully.
pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::SeqCst)
}

/// Handle to interact with the logs. Use it to change the log level at runtime or
/// to receive a stream of logs.
#[derive(Clone)]
pub struct LogHandle {
    env_filter: Handle<EnvFilter, Registry>,
    log_stream: LogStreamer,
}

/// A location to put logs.
///
/// It is necessary to logically separate the directory from the absolute path of the log file due
/// to the API of [`tracing_appender::rolling`].
#[derive(Clone, Debug, PartialEq)]
pub struct LogLocation {
    /// The directory where the logs will be recorded.
    pub directory: PathBuf,
    /// The filename where the logs will be recorded (relative to [Self::directory]).
    pub filename: PathBuf,
}

impl LogLocation {
    /// Construct the final path of the log file made up by the components of this [`LogLocation`].
    ///
    /// `self.directory/self.filename`
    pub fn log_path(&self) -> PathBuf {
        self.directory.join(&self.filename)
    }
}

/// A simple, asynchronous log sink.
///
/// To read from a [`LogStreamer`] sink, check out the associated [`LogHandle`] and [`LogHandle::get_log_stream`].
#[derive(Clone)]
struct LogStreamer {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl io::Write for LogStreamer {
    /// Will always write the entire `buf` or nothing (`0` bytes) in case there are no subscribers.
    ///
    /// See [`std::io::Write`].
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.tx.send(String::from_utf8(buf.to_vec()).unwrap()) {
            Ok(_n_subscribers) => Ok(buf.len()),
            // From the docs of `std::io::Write`:
            // "A return value of Ok(0) typically means that the underlying object is no longer able to accept bytes
            // and will likely not be able to in the future as well, or that the buffer provided is empty."
            // =>
            // Thus, returning `Ok(0)` is correct if no-one is subscribed and can received the `buf` message.
            Err(_e) => Ok(0),
        }
    }

    /// There is no intermediately buffered content, so `flush` will always succeed and is always
    /// a NOOP.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl LogHandle {
    /// Adjust the log level.
    ///
    /// - `level_filter`: A `RUST_LOG` string. See `env_logger` for more information:
    ///   https://docs.rs/env_logger/latest/env_logger/
    pub fn set_log_filter(
        &self,
        level_filter: impl AsRef<str>,
    ) -> Result<(), tracing_subscriber::reload::Error> {
        let new = silence_crates(EnvFilter::new(level_filter));
        self.env_filter.modify(|env_filter| *env_filter = new)
    }

    /// Subscribe to new log events.
    pub fn get_log_stream(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.log_stream.tx.subscribe()
    }
}

/// Initialize a global logger.
///
/// * log_level: Base log level, used if `RUST_LOG` is not set.
/// * log_location: Path to the log file, see [`LogLocation`].
/// * output_timestamp: Whether timestamps should be included in the log output.
pub fn init_logger(
    log_level: log::LevelFilter,
    log_location: Option<LogLocation>,
    output_timestamp: bool,
) -> Result<LogHandle, Error> {
    let level_filter = match log_level {
        log::LevelFilter::Off => LevelFilter::OFF,
        log::LevelFilter::Error => LevelFilter::ERROR,
        log::LevelFilter::Warn => LevelFilter::WARN,
        log::LevelFilter::Info => LevelFilter::INFO,
        log::LevelFilter::Debug => LevelFilter::DEBUG,
        log::LevelFilter::Trace => LevelFilter::TRACE,
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_filter.to_string()));

    let default_filter = silence_crates(env_filter);

    // TODO: Switch this to a rolling appender, likely daily or hourly
    let non_blocking_file_appender = if let Some(log_location) = log_location.as_ref() {
        // NOTE: Make sure to rotate log file *before* initializing any kind of logger.
        rotate_log(&log_location.log_path()).map_err(Error::RotateLog)?;
        let file_appender =
            tracing_appender::rolling::never(&log_location.directory, &log_location.filename);
        let (appender, guard) = non_blocking(file_appender);
        // Spawn a task to keep file logger guard alive for the duration of the program. When the tokio
        // executor shuts down, its drop function will be called to flush any remaining logs to file.
        // Note that calling e.g. `std::process::exit` will prevent this and might result in lost logs.
        tokio::spawn(async move {
            std::future::pending::<()>().await;
            drop(guard);
        });
        OptionalMakeWriter(Some(appender))
    } else {
        OptionalMakeWriter(None)
    };

    let (tx, _) = tokio::sync::broadcast::channel(128);
    let log_stream = LogStreamer { tx };

    let (user_filter, reload_handle) = tracing_subscriber::reload::Layer::new(default_filter);
    let reload_handle = LogHandle {
        env_filter: reload_handle,
        log_stream: log_stream.clone(),
    };

    let reg = tracing_subscriber::registry().with(user_filter);
    let stdout_formatter = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    #[cfg(all(target_os = "android", debug_assertions))]
    let reg = {
        let android_layer = paranoid_android::layer("mullvad-daemon");
        reg.with(android_layer)
    };

    if output_timestamp {
        let file_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender);
        let grpc_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_writer(std::sync::Mutex::new(log_stream));
        reg.with(
            stdout_formatter.with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
                DATE_TIME_FORMAT_STR.to_string(),
            )),
        )
        .with(
            grpc_formatter.with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
                DATE_TIME_FORMAT_STR.to_string(),
            )),
        )
        .with(
            file_formatter.with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
                DATE_TIME_FORMAT_STR.to_string(),
            )),
        )
        .init();
    } else {
        let grpc_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_writer(std::sync::Mutex::new(log_stream));
        let file_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender);
        reg.with(stdout_formatter.without_time())
            .with(file_formatter.without_time())
            .with(grpc_formatter.without_time())
            .init();
    }

    LOG_ENABLED.store(true, Ordering::SeqCst);

    Ok(reload_handle)
}
