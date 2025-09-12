use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::Error;

pub async fn create_remote_socket(
    ipv4: bool,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<UdpSocket, Error> {
    let random_bind_addr = if ipv4 {
        SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
    } else {
        SocketAddr::new("::".parse().unwrap(), 0)
    };
    let socket = UdpSocket::bind(random_bind_addr)
        .await
        .map_err(Error::BindRemoteUdp)?;
    #[cfg(target_os = "linux")]
    if let Some(fwmark) = fwmark {
        use nix::sys::socket::{setsockopt, sockopt};

        setsockopt(&socket, sockopt::Mark, &fwmark).map_err(Error::SetFwmark)?;
    }

    Ok(socket)
}
