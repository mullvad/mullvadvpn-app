use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use talpid_core::logging::rotate_log;
use tracing_appender::non_blocking;
use tracing_subscriber::{
    self, filter::LevelFilter, fmt::format::FmtSpan, layer::SubscriberExt, reload::Handle,
    util::SubscriberInitExt, EnvFilter, Registry,
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
// TODO: Should this be removed?
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

pub struct ReloadHandle {
    handle: Handle<EnvFilter, Registry>,
    _file_appender_guard: non_blocking::WorkerGuard,
}

impl ReloadHandle {
    pub fn set_log_filter(
        &self,
        level_filter: impl AsRef<str>,
    ) -> Result<(), tracing_subscriber::reload::Error> {
        self.handle
            .modify(|filter| *filter = tracing_subscriber::EnvFilter::new(level_filter))
    }
}

pub fn init_logger(
    log_level: log::LevelFilter,
    log_dir: Option<&PathBuf>,
    output_timestamp: bool,
) -> Result<ReloadHandle, Error> {
    let level_filter = match log_level {
        log::LevelFilter::Off => LevelFilter::OFF,
        log::LevelFilter::Error => LevelFilter::ERROR,
        log::LevelFilter::Warn => LevelFilter::WARN,
        log::LevelFilter::Info => LevelFilter::INFO,
        log::LevelFilter::Debug => LevelFilter::DEBUG,
        log::LevelFilter::Trace => LevelFilter::TRACE,
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::from_default_env().add_directive(level_filter.into()));

    let default_filter = get_default_filter(level_filter);

    // TODO: Switch this to a rolling appender, likely daily or hourly
    let file_appender = tracing_appender::rolling::never(log_dir.unwrap(), DAEMON_LOG_FILENAME);
    let (non_blocking_file_appender, _file_appender_guard) = non_blocking(file_appender);

    let stdout_formatter = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let (user_filter, reload_handle) = tracing_subscriber::reload::Layer::new(env_filter);
    let reload_handle = ReloadHandle {
        handle: reload_handle,
        _file_appender_guard,
    };

    let reg = tracing_subscriber::registry()
        .with(user_filter)
        .with(default_filter);

    // This is how you would hot reload the log level, give the handle to the proto server
    // handle
    //     .modify(|filter| *filter = EnvFilter::new(LevelFilter::ERROR.to_string()))
    //     .unwrap();
    if let Some(log_dir) = log_dir {
        rotate_log(&log_dir.join(DAEMON_LOG_FILENAME)).map_err(Error::RotateLog)?;
    }

    if output_timestamp {
        let file_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender);
        reg.with(
            stdout_formatter.with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
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
        let file_formatter = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking_file_appender);
        reg.with(stdout_formatter.without_time())
            .with(file_formatter.without_time())
            .init();
    }

    #[cfg(all(target_os = "android", debug_assertions))]
    {
        use android_logger::{AndroidLogger, Config};
        let logger: Box<dyn log::Log> = Box::new(AndroidLogger::new(
            Config::default().with_tag("mullvad-daemon"),
        ));
        top_dispatcher = top_dispatcher.chain(logger);
    }

    LOG_ENABLED.store(true, Ordering::SeqCst);

    Ok(reload_handle)
}

fn get_default_filter(level_filter: LevelFilter) -> EnvFilter {
    let mut env_filter = EnvFilter::builder().parse("trace").unwrap();
    for silenced_crate in WARNING_SILENCED_CRATES {
        env_filter = env_filter.add_directive(format!("{silenced_crate}=error").parse().unwrap());
    }
    for silenced_crate in SILENCED_CRATES {
        env_filter = env_filter.add_directive(format!("{silenced_crate}=warn").parse().unwrap());
    }

    // NOTE: the levels set here will never be overwritten, since the default filter cannot be
    // reloaded
    for silenced_crate in SLIGHTLY_SILENCED_CRATES {
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
