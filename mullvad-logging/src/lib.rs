//! Shared logging configuration for Mullvad VPN.
//!
//! This crate provides common log filtering configuration used across
//! mullvad-daemon, mullvad-ios, and other Mullvad components.

/// Crates where only Error level logs are shown (Warn and below are silenced)
pub const WARNING_SILENCED_CRATES: &[&str] = &["netlink_proto", "quinn_udp"];

/// Crates where only Warn and above logs are shown (Info and below are silenced)
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

/// Crates that are silenced one level below the configured level
pub const SLIGHTLY_SILENCED_CRATES: &[&str] = &["nftnl", "udp_over_tcp"];

/// Returns the effective log level for a given target (crate/module name).
///
/// This checks the target against the silenced crate lists and returns
/// the appropriate maximum log level.
pub fn get_log_level_for_target(target: &str, default_level: log::LevelFilter) -> log::LevelFilter {
    for silenced in WARNING_SILENCED_CRATES {
        if target.starts_with(silenced) {
            return log::LevelFilter::Error;
        }
    }

    for silenced in SILENCED_CRATES {
        if target.starts_with(silenced) {
            return log::LevelFilter::Warn;
        }
    }

    for silenced in SLIGHTLY_SILENCED_CRATES {
        if target.starts_with(silenced) {
            return one_level_quieter(default_level);
        }
    }

    default_level
}

/// Returns a log level that is one level quieter than the input level.
pub fn one_level_quieter(level: log::LevelFilter) -> log::LevelFilter {
    match level {
        log::LevelFilter::Off => log::LevelFilter::Off,
        log::LevelFilter::Error => log::LevelFilter::Off,
        log::LevelFilter::Warn => log::LevelFilter::Error,
        log::LevelFilter::Info => log::LevelFilter::Warn,
        log::LevelFilter::Debug => log::LevelFilter::Info,
        log::LevelFilter::Trace => log::LevelFilter::Debug,
    }
}
