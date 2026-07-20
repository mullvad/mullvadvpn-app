use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::Error;

pub async fn create_remote_socket(ipv4: bool) -> Result<UdpSocket, Error> {
    let random_bind_addr = if ipv4 {
        SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
    } else {
        SocketAddr::new("::".parse().unwrap(), 0)
    };
    UdpSocket::bind(random_bind_addr)
        .await
        .map_err(Error::BindRemoteUdp)
}
