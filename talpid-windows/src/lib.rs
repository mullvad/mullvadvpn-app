//! Interface with low-level windows specific bits.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

#[cfg(windows)]
pub mod driver;
#[cfg(windows)]
pub mod logging;
#[cfg(windows)]
pub mod net;
#[cfg(windows)]
pub mod string;
