use std::{io, task::Poll};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

use crate::config::Obfuscator;

pub struct Forwarder {
    read_obfuscator: Box<dyn Obfuscator>,
    write_obfuscator: Box<dyn Obfuscator>,
    server_connection: TcpStream,
}

impl Forwarder {
    pub async fn connect(read_obfuscator: Box<dyn Obfuscator>) -> io::Result<Self> {
        let server_connection = TcpStream::connect(read_obfuscator.addr()).await?;
        let write_obfuscator = read_obfuscator.clone();

        Ok(Self {
            read_obfuscator,
            write_obfuscator,
            server_connection,
        })
    }
    pub async fn forward(self, client_stream: TcpStream) {
        let (server_read, server_write) = self.server_connection.into_split();
        let (client_read, client_write) = client_stream.into_split();
        let handle = tokio::spawn(async move {
            tokio::spawn(forward(self.read_obfuscator, client_read, server_write));
        });
        let _ = forward(self.write_obfuscator, server_read, client_write).await;
        let _ = handle.await;
    }
}

impl tokio::io::AsyncRead for Forwarder {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Need to keep track of how many bytes in the buffer have already been deobfuscated.
        let new_read_start = buf.remaining();

        let socket = std::pin::pin!(&mut self.server_connection);
        match socket.poll_read(cx, buf) {
            // in this case, we can read and deobfuscate.
            Poll::Ready(Ok(())) => {
                let newly_read_bytes = &mut buf.filled_mut()[new_read_start..];
                self.read_obfuscator.obfuscate(newly_read_bytes);
                Poll::Ready(Ok(()))
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }
}

impl tokio::io::AsyncWrite for Forwarder {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let socket = std::pin::pin!(&mut self.server_connection);
        match socket.poll_write_ready(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(err)) => {
                return Poll::Ready(Err(err));
            }
            Poll::Pending => {
                return Poll::Pending;
            }
        };
        std::mem::drop(socket);
        let mut owned_buf = buf.to_vec();
        self.write_obfuscator.obfuscate(owned_buf.as_mut_slice());
        let socket = std::pin::pin!(&mut self.server_connection);
        socket.poll_write(cx, &owned_buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        std::pin::pin!(&mut self.server_connection).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        std::pin::pin!(&mut self.server_connection).poll_shutdown(cx)
    }
}

pub async fn forward(
    mut obfuscator: Box<dyn Obfuscator + Send>,
    mut source: impl AsyncRead + Unpin,
    mut sink: impl AsyncWrite + Unpin,
) -> io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = vec![0u8; 1024 * 64];
    while let Ok(n_bytes_read) = AsyncReadExt::read(&mut source, &mut buf).await {
        if n_bytes_read == 0 {
            break;
        }
        let mut bytes_received = &mut buf[..n_bytes_read];

        obfuscator.obfuscate(&mut bytes_received);
        sink.write_all(&bytes_received).await?;
    }
    Ok(())
}
