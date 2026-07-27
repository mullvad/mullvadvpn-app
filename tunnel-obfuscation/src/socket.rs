use std::{net::SocketAddr, ops::Deref, sync::Arc};
use talpid_net::bypass::{BypassGuard, SocketBypass};
use tokio::net::UdpSocket;

use crate::Error;

pub struct RemoteSocket<S = UdpSocket> {
    pub socket: S,
    pub guard: BypassGuard,
}

impl<S> Deref for RemoteSocket<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

/// Bind a UDP socket for talking to a remote obfuscator, and exclude it from tunnel traffic.
///
/// The returned [BypassGuard] must be kept alive for as long as the socket is in use. Dropping it
/// revokes the bypass, after which there is no guarantee that traffic on the socket stays outside
/// the tunnel.
pub async fn create_remote_socket(
    bypass: &Arc<dyn SocketBypass>,
    ipv4: bool,
) -> Result<RemoteSocket, Error> {
    let random_bind_addr = if ipv4 {
        SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
    } else {
        SocketAddr::new("::".parse().unwrap(), 0)
    };
    let socket = UdpSocket::bind(random_bind_addr)
        .await
        .map_err(Error::BindRemoteUdp)?;
    let guard = BypassGuard::new(Arc::clone(bypass), &socket).map_err(Error::Bypass)?;
    Ok(RemoteSocket { socket, guard })
}
