//! Mullvad Encrypted DNS proxy is a custom protocol for reaching the Mullvad API over proxies,
//! with some amount of simple obfuscation applied.
//!
//! The proxy endpoints and what obfuscation they expect is fetched over DNS-over-HTTPS (DoH)
//! in AAAA records. The AAAA (IPv6) records are then decoded into a proxy config consisting
//! of a remote endpoint to connect to, and what obfuscation to use.
pub use forwarder::Forwarder;

pub mod config;
pub mod config_resolver;
mod forwarder;
pub mod state;
