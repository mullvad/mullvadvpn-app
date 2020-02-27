#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::enable;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::enable;
