use futures::StreamExt;
use std::io;
use std::net::SocketAddr;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to start SOCKS5 server")]
    StartSocksServer(#[error(source)] io::Error),
}

pub async fn spawn(bind_addr: SocketAddr) -> Result<tokio::task::JoinHandle<()>, Error> {
    let socks_server: fast_socks5::server::Socks5Server =
        fast_socks5::server::Socks5Server::bind(bind_addr)
            .await
            .map_err(Error::StartSocksServer)?;

    let handle = tokio::spawn(async move {
        let mut incoming = socks_server.incoming();

        while let Some(new_client) = incoming.next().await {
            match new_client {
                Ok(socket) => {
                    let fut = socket.upgrade_to_socks5();
                    tokio::spawn(async move {
                        match fut.await {
                            Ok(_socket) => log::info!("socks client disconnected"),
                            Err(error) => log::error!("socks client failed: {error}"),
                        }
                    });
                }
                Err(error) => {
                    log::error!("failed to accept socks client: {error}");
                }
            }
        }
    });
    Ok(handle)
}
