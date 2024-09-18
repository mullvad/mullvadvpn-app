use std::{io, task::{ready, Poll}};

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
        let _ = tokio::join!(
            forward(self.read_obfuscator, client_read, server_write),
            forward(self.write_obfuscator, server_read, client_write)
        );
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
        match ready!(socket.poll_read(cx, buf)) {
            // in this case, we can read and deobfuscate.
            Ok(()) => {
                let newly_read_bytes = &mut buf.initialized_mut()[new_read_start..];
                self.read_obfuscator.obfuscate(newly_read_bytes);
                Poll::Ready(Ok(()))
            }
            Err(err) => Poll::Ready(Err(err)),
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
        if let Err(err) =  ready!(socket.poll_write_ready(cx)) {
                return Poll::Ready(Err(err));
        };

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

async fn forward(
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
        let bytes_received = &mut buf[..n_bytes_read];

        obfuscator.obfuscate(bytes_received);
        sink.write_all(bytes_received).await?;
    }
    Ok(())
}

// Constructs a server and a client, uses the Xor obfuscator to forward some bytes between to see
// the obfuscation works.
#[tokio::test]
async fn test_async_methods() {
    use std::net::Ipv6Addr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let server_listener =
        tokio::net::TcpListener::bind("127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap())
            .await
            .unwrap();
    let listener_addr = server_listener.local_addr().unwrap();
    let xor_key: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x00, 0x00];
    let address_bytes: &[u8] = &[127, 0, 0, 1];
    let port: &[u8] = &listener_addr.port().to_ne_bytes();

    // 0x2001 - bogus IPv6 bytes
    // 0x0300 - XOR proxy type
    let mut ipv6_bytes = vec![0x20, 0x01, 0x03, 0x00];
    ipv6_bytes.extend_from_slice(address_bytes);
    ipv6_bytes.extend_from_slice(port);
    ipv6_bytes.extend_from_slice(xor_key);
    let mut ipv6_buf = [0u8; 16];
    ipv6_buf.copy_from_slice(&ipv6_bytes);

    let ipv6 = Ipv6Addr::from(ipv6_buf);

    let xor = crate::config::Xor::try_from(ipv6).unwrap();
    let server_xor = Obfuscator::clone(&xor);

    tokio::spawn(async move {
        let (client_conn, _) = server_listener.accept().await.unwrap();
        let mut forwarder = Forwarder {
            read_obfuscator: server_xor.clone(),
            write_obfuscator: server_xor.clone(),
            server_connection: client_conn,
        };
        let mut buf = vec![0u8; 1024];
        while let Ok(bytes_read) = forwarder.read(&mut buf).await {
            forwarder.write_all(&buf[..bytes_read]).await.unwrap();
        }
    });

    let mut client = Forwarder::connect(Box::new(xor)).await.unwrap();

    for _ in 0..5 {
        let payload = (1..127).collect::<Vec<u8>>();
        client.write_all(&payload).await.unwrap();
        let mut read_buf = vec![0u8; payload.len()];
        client.read_exact(&mut read_buf).await.unwrap();
        assert_eq!(payload, read_buf);
    }
}
