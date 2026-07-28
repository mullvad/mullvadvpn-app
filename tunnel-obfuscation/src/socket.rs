use std::{net::SocketAddr, sync::Arc};
use talpid_net::bypass::{BypassGuard, SocketBypass};
use tokio::net::UdpSocket;

use crate::Error;

/// Bind a UDP socket for talking to a remote obfuscator, and exclude it from tunnel traffic.
///
/// The returned [BypassGuard] must be kept alive for as long as the socket is in use. Dropping it
/// revokes the bypass, after which there is no guarantee that traffic on the socket stays outside
/// the tunnel.
pub async fn create_remote_socket(
    bypass: &Arc<dyn SocketBypass>,
    ipv4: bool,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<(UdpSocket, BypassGuard), Error> {
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

    let guard = BypassGuard::new(Arc::clone(bypass), &socket).map_err(Error::Bypass)?;
    Ok((socket, guard))
}
