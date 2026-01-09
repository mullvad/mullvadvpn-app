use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use talpid_core::logging::rotate_log;
use tracing_appender::non_blocking;
use tracing_subscriber::{
    EnvFilter, Registry, filter::LevelFilter, fmt::format::FmtSpan, layer::SubscriberExt,
    reload::Handle, util::SubscriberInitExt,
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

    #[error("Unable to set logger")]
    SetLoggerError(#[from] log::SetLoggerError),
}

pub const WARNING_SILENCED_CRATES: &[&str] = &["netlink_proto", "quinn_udp"];
const DAEMON_LOG_FILENAME: &str = "daemon.log";
pub const SILENCED_CRATES: &[&str] = &[
    "h2",
    "tokio_core",
    "tokio_io",
    "tokio_proto",
    "tokio_reactor",
    "tokio_threadpool",
    "tokio_util",
    "tower",
    "want",
    "ws",
    "mio",
    "mnl",
    "hyper",
    "hyper_util",
    "rtnetlink",
    "rustls",
    "netlink_sys",
    "tracing",
    "hickory_proto",
    "hickory_server",
    "hickory_resolver",
    "shadowsocks::relay::udprelay",
    "quinn_proto",
    "quinn",
];
const SLIGHTLY_SILENCED_CRATES: &[&str] = &["nftnl", "udp_over_tcp"];

const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

/// Whether a [log] logger has been initialized.
// the log crate doesn't provide a nice way to tell if a logger has been initialized :(
static LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check whether logging has been enabled, i.e. if [init_logger] has been called successfully.
pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::SeqCst)
}

pub struct LogHandle {
    env_filter: Handle<EnvFilter, Registry>,
    log_stream: LogStreamer,
    _file_appender_guard: Option<non_blocking::WorkerGuard>,
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

pub fn init_logger(
    log_level: log::LevelFilter,
    log_dir: Option<&PathBuf>,
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
    let (non_blocking_file_appender, _file_appender_guard) = if let Some(log_dir) = log_dir {
        let file_appender = tracing_appender::rolling::never(log_dir, DAEMON_LOG_FILENAME);
        let (appender, guard) = non_blocking(file_appender);
        (Some(appender), Some(guard))
    } else {
        (None, None)
    };

    let (tx, _) = tokio::sync::broadcast::channel(128);
    let log_stream = LogStreamer { tx };

    let (user_filter, reload_handle) = tracing_subscriber::reload::Layer::new(default_filter);
    let reload_handle = LogHandle {
        env_filter: reload_handle,
        log_stream: log_stream.clone(),
        _file_appender_guard,
    };

    let reg = tracing_subscriber::registry().with(user_filter);
    let stdout_formatter = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    if let Some(log_dir) = log_dir {
        rotate_log(&log_dir.join(DAEMON_LOG_FILENAME)).map_err(Error::RotateLog)?;
    }

    #[cfg(all(target_os = "android", debug_assertions))]
    let reg = {
        let android_layer = paranoid_android::layer("mullvad-daemon");
        reg.with(android_layer)
    };

    match (non_blocking_file_appender, output_timestamp) {
        (Some(non_blocking_file_appender), true) => {
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
        }
        (Some(non_blocking_file_appender), false) => {
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
        (None, true) => {
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
            .init();
        }
        (None, false) => {
            let grpc_formatter = tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(std::sync::Mutex::new(log_stream));
            reg.with(stdout_formatter.without_time())
                .with(grpc_formatter.without_time())
                .init();
        }
    }

    LOG_ENABLED.store(true, Ordering::SeqCst);

    Ok(reload_handle)
}

fn silence_crates(mut env_filter: EnvFilter) -> EnvFilter {
    for silenced_crate in WARNING_SILENCED_CRATES {
        env_filter = env_filter.add_directive(format!("{silenced_crate}=error").parse().unwrap());
    }
    for silenced_crate in SILENCED_CRATES {
        env_filter = env_filter.add_directive(format!("{silenced_crate}=warn").parse().unwrap());
    }

    // NOTE: the levels set here will never be overwritten, since the default filter cannot be
    // reloaded
    for silenced_crate in SLIGHTLY_SILENCED_CRATES {
        let level_filter = env_filter.max_level_hint().unwrap();
        env_filter = env_filter.add_directive(
            format!("{silenced_crate}={}", one_level_quieter(level_filter))
                .parse()
                .unwrap(),
        );
    }
    env_filter
}

fn one_level_quieter(level: LevelFilter) -> LevelFilter {
    match level {
        LevelFilter::OFF => LevelFilter::OFF,
        LevelFilter::ERROR => LevelFilter::OFF,
        LevelFilter::WARN => LevelFilter::ERROR,
        LevelFilter::INFO => LevelFilter::WARN,
        LevelFilter::DEBUG => LevelFilter::INFO,
        LevelFilter::TRACE => LevelFilter::DEBUG,
    }
}
