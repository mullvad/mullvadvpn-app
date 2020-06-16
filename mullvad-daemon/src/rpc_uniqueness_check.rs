use mullvad_paths;
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use talpid_types::ErrorExt;
use tonic::{
    self,
    transport::{self, Endpoint, Uri},
};
use tower::service_fn;

mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}
use proto::management_service_client::ManagementServiceClient;

async fn new_grpc_client() -> Result<ManagementServiceClient<transport::Channel>, transport::Error>
{
    let ipc_path = mullvad_paths::get_rpc_socket_path();

    // The URI will be ignored
    let channel = Endpoint::from_static("lttp://[::]:50051")
        .connect_with_connector(service_fn(move |_: Uri| {
            IpcEndpoint::connect(ipc_path.clone())
        }))
        .await?;

    Ok(ManagementServiceClient::new(channel))
}

/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub async fn is_another_instance_running() -> bool {
    match new_grpc_client().await {
        Ok(_) => true,
        Err(error) => {
            let msg =
                "Failed to locate/connect to another daemon instance, assuming there isn't one";
            if log::log_enabled!(log::Level::Trace) {
                log::trace!("{}\n{}", msg, error.display_chain());
            } else {
                log::debug!("{}", msg);
            }
            false
        }
    }
}
