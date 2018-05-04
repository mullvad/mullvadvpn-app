/// Some docs
extern crate duct;
use std::io;

/// Proc handle for an openvpn process
pub struct OpenVpnProcHandle {
    /// Duct handle
    pub inner: duct::Handle,
}

/// Impl for proc handle
impl OpenVpnProcHandle {
    /// Constructor for a new openvpn proc handle
    pub fn new(expr: duct::Expression) -> io::Result<Self> {
        Ok(Self {
            inner: expr.start()?,
        })
    }

    /// Sends a SIGTERM signal to the OpenVPN process. Does not block, no guarantee that the
    /// process will have exited after this function returns.
    #[cfg(unix)]
    pub fn try_stop(&self) -> io::Result<()> {
        use duct::unix::HandleExt;
        extern crate libc;
        self.inner.send_signal(libc::SIGTERM)
    }

    /// Tries to shut down the OpenVPN process in a nondestructive manner. Does not block, no
    /// guarantee that the process will have exited after this function returns.
    #[cfg(windows)]
    pub fn try_stop(&self) -> io::Result<()> {
        Ok(())
    }

    /// Brutally kills the underlinyg process
    pub fn kill_process(&self) -> io::Result<()> {
        self.inner.kill()
    }
}
