//! Forward TCP traffic over various proxy configurations.

use std::{
    io,
    task::{ready, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

use crate::config::Obfuscator;

/// Forwards local traffic to a proxy endpoint, obfuscating it if the proxy config says so.
///
/// Obtain [`ProxyConfig`](crate::config::ProxyConfig)s with
/// [resolve_configs](crate::config_resolver::resolve_configs).
pub struct Forwarder {
    read_obfuscator: Option<Box<dyn Obfuscator>>,
    write_obfuscator: Option<Box<dyn Obfuscator>>,
    server_connection: TcpStream,
}

impl Forwarder {
    /// Create a forwarder that will connect to a given proxy endpoint.
    pub async fn connect(proxy_config: &crate::config::ProxyConfig) -> io::Result<Self> {
        let server_connection = TcpStream::connect(proxy_config.addr).await?;

        let (read_obfuscator, write_obfuscator) =
            if let Some(obfuscation_config) = &proxy_config.obfuscation {
                (
                    Some(obfuscation_config.create_obfuscator()),
                    Some(obfuscation_config.create_obfuscator()),
                )
            } else {
                (None, None)
            };

        Ok(Self {
            read_obfuscator,
            write_obfuscator,
            server_connection,
        })
    }

    /// Forwards traffic from the client stream to the remote proxy, obfuscating and deobfuscating
    /// it in the process.
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
        let socket = std::pin::pin!(&mut self.server_connection);
        match ready!(socket.poll_read(cx, buf)) {
            // in this case, we can read and deobfuscate.
            Ok(()) => {
                if let Some(read_obfuscator) = &mut self.read_obfuscator {
                    read_obfuscator.obfuscate(buf.filled_mut());
                }
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
        if let Err(err) = ready!(socket.poll_write_ready(cx)) {
            return Poll::Ready(Err(err));
        };

        let mut owned_buf = buf.to_vec();
        if let Some(write_obfuscator) = &mut self.write_obfuscator {
            write_obfuscator.obfuscate(&mut owned_buf);
        }
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
    mut obfuscator: Option<Box<dyn Obfuscator>>,
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

        if let Some(obfuscator) = &mut obfuscator {
            obfuscator.obfuscate(bytes_received);
        }
        sink.write_all(bytes_received).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::config::{ObfuscationConfig, XorKey};

    use super::Forwarder;

    // Constructs a server and a client, uses the Xor obfuscator to forward some bytes between to see
    // the obfuscation works.
    #[tokio::test]
    async fn async_methods() {
        const XOR_KEY: [u8; 6] = [0x01, 0x02, 0x03, 0x04, 0x00, 0x00];
        const LISTEN_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;

        let server_listener = tokio::net::TcpListener::bind(SocketAddrV4::new(LISTEN_IP, 0))
            .await
            .unwrap();
        let listen_port = server_listener.local_addr().unwrap().port();
        let listen_addr = SocketAddrV4::new(LISTEN_IP, listen_port);

        let xor_key = XorKey::try_from(XOR_KEY).unwrap();
        let obfuscation_config = ObfuscationConfig::XorV2(xor_key);

        let mut client_read_xor = obfuscation_config.create_obfuscator();
        let mut client_write_xor = obfuscation_config.create_obfuscator();

        // Server future - receives one TCP connection, then echos everything it reads from it back to
        // the client, using obfuscation via the forwarder in both cases.
        tokio::spawn(async move {
            let (client_conn, _) = server_listener.accept().await.unwrap();
            let mut forwarder = Forwarder {
                read_obfuscator: Some(obfuscation_config.create_obfuscator()),
                write_obfuscator: Some(obfuscation_config.create_obfuscator()),
                server_connection: client_conn,
            };
            let mut buf = vec![0u8; 1024];
            while let Ok(bytes_read) = forwarder.read(&mut buf).await {
                eprintln!("Forwarder read {bytes_read} bytes. Echoing them back");
                forwarder.write_all(&buf[..bytes_read]).await.unwrap();
            }
        });

        let mut client_connection = TcpStream::connect(listen_addr).await.unwrap();

        for _ in 0..5 {
            let original_payload = (1..127).collect::<Vec<u8>>();
            let mut payload = original_payload.clone();
            client_write_xor.obfuscate(payload.as_mut_slice());
            client_connection.write_all(&payload).await.unwrap();
            let mut read_buf = vec![0u8; payload.len()];
            client_connection.read_exact(&mut read_buf).await.unwrap();
            client_read_xor.obfuscate(&mut read_buf);
            assert_eq!(original_payload, read_buf);
        }
    }
}
