use crate::gotatun::smoltcp_network::{SmoltcpHandle, SmoltcpTcpStream};
use std::{ffi::CStr, io, net::SocketAddr, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};

/// A TCP connection provider backed by a smoltcp network stack.
///
/// Replaces `IosTcpProvider` for the GotaTun path: instead of using WireGuard
/// Go FFI function pointers, TCP connections are created through the smoltcp
/// userspace stack whose traffic flows through GotaTun.
#[derive(Clone)]
pub struct SmoltcpTcpProvider {
    handle: SmoltcpHandle,
    timeout: Duration,
}

/// A TCP connection backed by a [`SmoltcpTcpStream`].
///
/// Implements [`AsyncRead`] and [`AsyncWrite`], compatible with the tonic/gRPC
/// transport used by the ephemeral peer exchange.
pub struct SmoltcpTcpConnection {
    stream: SmoltcpTcpStream,
}

impl SmoltcpTcpProvider {
    pub fn new(handle: SmoltcpHandle, timeout: Duration) -> Self {
        Self { handle, timeout }
    }

    /// Connect to the given address through the smoltcp stack.
    pub async fn connect(&self, address: &CStr) -> Result<SmoltcpTcpConnection, io::Error> {
        let addr_str = address
            .to_str()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let stream = self.handle.tcp_connect(addr).await?;
        Ok(SmoltcpTcpConnection { stream })
    }
}

impl AsyncRead for SmoltcpTcpConnection {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for SmoltcpTcpConnection {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
