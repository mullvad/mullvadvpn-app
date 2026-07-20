use std::{io, sync::Arc};

use socket2::SockRef;

/// A trait for implementing socket bypass. This lets individual sockets be excluded (leak) from
/// VPN tunnel traffic.
pub trait SocketBypass {
    /// Begin socket bypass. When called, the socket must be excluded from tunnel traffic until
    /// [Self::revoke_bypass] has been called and the socket has been destroyed.
    fn bypass_socket(&self, socket: SockRef<'_>) -> io::Result<()>;

    /// Allow the bypass to cease.
    ///
    /// When this has succeeded, there is no longer any guarantee that the socket will be
    /// excluded. The bypass must not outlast the lifetime of the socket lifetime, but it may cease
    /// immediately when this is called (depending on the implementation).
    fn revoke_bypass(&self, socket: SockRef<'_>) -> io::Result<()>;
}

/// A guard that, when dropped, allows an excluded socket to no longer be excluded.
///
/// There is no guarantee that dropping this will stop excluding the socket. The contract is
/// that when this guard is dropped, there is no longer any guarantee that the socket will be
/// excluded. Whether it is immediately un-excluded is implementation-dependent.
pub struct BypassedSocket {
    bypass: Arc<dyn SocketBypass>,
    socket: socket2::Socket,
}

impl BypassedSocket {
    /// Begin excluding a socket `s` from tunnel traffic.
    pub fn new<'a, S: Into<SockRef<'a>>>(
        bypass: Arc<dyn SocketBypass>,
        s: S,
    ) -> io::Result<BypassedSocket> {
        let socket = s.into().try_clone()?;

        bypass.bypass_socket(SockRef::from(&socket))?;

        Ok(BypassedSocket { bypass, socket })
    }
}

impl Drop for BypassedSocket {
    fn drop(&mut self) {
        if let Err(err) = self.bypass.revoke_bypass(SockRef::from(&self.socket)) {
            log::error!("Failed to revoke socket bypass: {err}");
        }
    }
}
