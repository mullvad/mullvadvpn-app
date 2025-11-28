pub mod client;
pub mod types;

use parity_tokio_ipc::Endpoint as IpcEndpoint;
#[cfg(unix)]
use std::{env, fs, os::unix::fs::PermissionsExt};
use std::{
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
#[cfg(not(target_os = "android"))]
use tonic::transport::{Endpoint, Uri};
use tonic::transport::{Server, server::Connected};
#[cfg(not(target_os = "android"))]
use tower::service_fn;

pub use tonic::{Code, Request, Response, Status, async_trait, transport::Channel};

pub type ManagementServiceClient =
    types::management_service_client::ManagementServiceClient<Channel>;
pub use types::management_service_server::{ManagementService, ManagementServiceServer};

#[cfg(unix)]
use std::sync::LazyLock;
#[cfg(unix)]
static MULLVAD_MANAGEMENT_SOCKET_GROUP: LazyLock<Option<String>> =
    LazyLock::new(|| env::var("MULLVAD_MANAGEMENT_SOCKET_GROUP").ok());

pub const API_ACCESS_METHOD_EXISTS_DETAILS: &[u8] = b"api_access_method_exists";
pub const CUSTOM_LIST_LIST_NOT_FOUND_DETAILS: &[u8] = b"custom_list_list_not_found";
pub const CUSTOM_LIST_LIST_EXISTS_DETAILS: &[u8] = b"custom_list_list_exists";
pub const CUSTOM_LIST_LIST_NAME_TOO_LONG_DETAILS: &[u8] = b"custom_list_list_name_too_long";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Management RPC server or client error")]
    GrpcTransportError(#[source] tonic::transport::Error),

    #[error("Failed to start IPC pipe/socket")]
    StartServerError(#[source] io::Error),

    #[error("Failed to initialize pipe/socket security attributes")]
    SecurityAttributes(#[source] io::Error),

    #[error("Unable to set permissions for IPC endpoint")]
    PermissionsError(#[source] io::Error),

    #[cfg(unix)]
    #[error("Group not found")]
    NoGidError,

    #[cfg(unix)]
    #[error("Failed to obtain group ID")]
    ObtainGidError(#[source] nix::Error),

    #[cfg(unix)]
    #[error("Failed to set group ID")]
    SetGidError(#[source] nix::Error),

    // TODO: Remove box when upgrading tonic to a version with
    // https://github.com/hyperium/tonic/pull/2282
    #[error("gRPC call returned error")]
    Rpc(#[source] Box<tonic::Status>),

    #[error("Failed to parse gRPC response")]
    InvalidResponse(#[source] types::FromProtobufTypeError),

    #[error("Duration is too large")]
    DurationTooLarge,

    #[error("Unexpected non-UTF8 string")]
    PathMustBeUtf8,

    #[error("Missing daemon event")]
    MissingDaemonEvent,

    #[error("This voucher code is invalid")]
    InvalidVoucher,

    #[error("This voucher code has already been used")]
    UsedVoucher,

    #[error("There are too many devices on the account. One must be revoked to log in")]
    TooManyDevices,

    #[error("You are already logged in. Log out to create a new account")]
    AlreadyLoggedIn,

    #[error("The account does not exist")]
    InvalidAccount,

    #[error("There is no such device")]
    DeviceNotFound,

    #[error("Location data is unavailable")]
    NoLocationData,

    #[error("A custom list with that name already exists")]
    CustomListExists,

    #[error("A custom list with that name does not exist")]
    CustomListListNotFound,

    #[error("Location already exists in the custom list")]
    LocationExistsInCustomList,

    #[error("Location was not found in the custom list")]
    LocationNotFoundInCustomlist,

    #[error("Could not retrieve API access methods from settings")]
    ApiAccessMethodSettingsNotFound,

    #[error("An access method with that id does not exist")]
    ApiAccessMethodNotFound,

    #[error("An access method with that name already exists")]
    ApiAccessMethodExists,
}

impl From<tonic::Status> for Error {
    fn from(value: tonic::Status) -> Self {
        Error::Rpc(Box::new(value))
    }
}

#[cfg(not(target_os = "android"))]
#[deprecated(note = "Prefer MullvadProxyClient")]
pub async fn new_rpc_client(
    remote_http_addr: Option<String>,
) -> Result<ManagementServiceClient, Error> {
    use futures::TryFutureExt;

    if let Some(http_address) = remote_http_addr {
        let management_client = ManagementServiceClient::connect(http_address)
            .await
            .unwrap();
        Ok(management_client)
    } else {
        let ipc_path = mullvad_paths::get_rpc_socket_path();

        // The URI will be ignored
        let channel = Endpoint::from_static("lttp://[::]:50051")
            .connect_with_connector(service_fn(move |_: Uri| {
                IpcEndpoint::connect(ipc_path.clone()).map_ok(hyper_util::rt::tokio::TokioIo::new)
            }))
            .await
            .map_err(Error::GrpcTransportError)?;

        Ok(ManagementServiceClient::new(channel))
    }
}

#[cfg(not(target_os = "android"))]
pub use client::MullvadProxyClient;

pub type ServerJoinHandle = tokio::task::JoinHandle<()>;

pub fn spawn_uds_rpc_server<T: ManagementService, F: Future<Output = ()> + Send + 'static>(
    service: T,
    abort_rx: F,
    rpc_socket_path: impl AsRef<std::path::Path>,
) -> std::result::Result<ServerJoinHandle, Error> {
    use futures::stream::TryStreamExt;
    use parity_tokio_ipc::SecurityAttributes;

    let mut endpoint = IpcEndpoint::new(rpc_socket_path.as_ref().to_string_lossy().to_string());
    endpoint.set_security_attributes(
        SecurityAttributes::allow_everyone_create()
            .map_err(Error::SecurityAttributes)?
            .set_mode(0o766)
            .map_err(Error::SecurityAttributes)?,
    );
    let incoming = endpoint.incoming().map_err(Error::StartServerError)?;

    #[cfg(unix)]
    if let Some(group_name) = &*MULLVAD_MANAGEMENT_SOCKET_GROUP {
        let group = nix::unistd::Group::from_name(group_name)
            .map_err(Error::ObtainGidError)?
            .ok_or(Error::NoGidError)?;
        nix::unistd::chown(rpc_socket_path.as_ref(), None, Some(group.gid))
            .map_err(Error::SetGidError)?;
        fs::set_permissions(rpc_socket_path, PermissionsExt::from_mode(0o760))
            .map_err(Error::PermissionsError)?;
    }

    Ok(tokio::spawn(async move {
        if let Err(execution_error) = Server::builder()
            .add_service(ManagementServiceServer::new(service))
            .serve_with_incoming_shutdown(incoming.map_ok(StreamBox), abort_rx)
            .await
            .map_err(Error::GrpcTransportError)
        {
            log::error!("UDP Management server panic: {execution_error}");
        }
        log::trace!("UDP gRPC server is shutting down");
    }))
}

pub fn spawn_http_rpc_server<T: ManagementService>(
    service: T,
    socket_address: SocketAddr,
) -> std::result::Result<ServerJoinHandle, Error> {
    Ok(tokio::spawn(async move {
        if let Err(execution_error) = Server::builder()
            .add_service(ManagementServiceServer::new(service))
            .serve(socket_address)
            .await
            .map_err(Error::GrpcTransportError)
        {
            log::error!("HTTP Management server panic: {execution_error}");
        }
        log::trace!("HTTP gRPC server is shutting down");
    }))
}

#[derive(Debug)]
struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);
impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {
    type ConnectInfo = Option<()>;

    fn connect_info(&self) -> Self::ConnectInfo {
        None
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
