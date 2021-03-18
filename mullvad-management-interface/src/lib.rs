pub mod types {
    tonic::include_proto!("mullvad_daemon.management_interface");

    pub use prost_types::{Duration, Timestamp};
}

use parity_tokio_ipc::Endpoint as IpcEndpoint;
#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tonic::transport::{server::Connected, Endpoint, Server, Uri};
use tower::service_fn;

pub use tonic::{async_trait, transport::Channel, Code, Request, Response, Status};

pub type ManagementServiceClient =
    types::management_service_client::ManagementServiceClient<Channel>;
pub use types::management_service_server::{ManagementService, ManagementServiceServer};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Management RPC server or client error")]
    GrpcTransportError(#[error(source)] tonic::transport::Error),

    #[error(display = "Failed to start IPC pipe/socket")]
    StartServerError(#[error(source)] io::Error),

    #[error(display = "Failed to initialize pipe/socket security attributes")]
    SecurityAttributes(#[error(source)] io::Error),

    #[error(display = "Unable to set permissions for IPC endpoint")]
    PermissionsError(#[error(source)] io::Error),
}

pub async fn new_rpc_client() -> Result<ManagementServiceClient, Error> {
    let ipc_path = mullvad_paths::get_rpc_socket_path();

    // The URI will be ignored
    let channel = Endpoint::from_static("lttp://[::]:50051")
        .connect_with_connector(service_fn(move |_: Uri| {
            IpcEndpoint::connect(ipc_path.clone())
        }))
        .await
        .map_err(Error::GrpcTransportError)?;

    Ok(ManagementServiceClient::new(channel))
}

pub async fn spawn_rpc_server<T: ManagementService>(
    service: T,
    server_start_tx: std::sync::mpsc::Sender<()>,
    abort_rx: triggered::Listener,
) -> std::result::Result<(), Error> {
    use futures::stream::TryStreamExt;
    use parity_tokio_ipc::SecurityAttributes;

    let socket_path = mullvad_paths::get_rpc_socket_path()
        .to_string_lossy()
        .to_string();

    let mut endpoint = IpcEndpoint::new(socket_path.to_string());
    endpoint.set_security_attributes(
        SecurityAttributes::allow_everyone_create()
            .map_err(Error::SecurityAttributes)?
            .set_mode(777)
            .map_err(Error::SecurityAttributes)?,
    );
    let incoming = endpoint.incoming().map_err(Error::StartServerError)?;

    #[cfg(unix)]
    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))
        .map_err(Error::PermissionsError)?;

    let _ = server_start_tx.send(());

    Server::builder()
        .add_service(ManagementServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming.map_ok(StreamBox), abort_rx)
        .await
        .map_err(Error::GrpcTransportError)
}

#[derive(Debug)]
struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);
impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
