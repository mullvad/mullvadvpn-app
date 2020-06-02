use super::{Arguments, Error};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use std::collections::HashMap;
use tower::service_fn;

use tonic::{
    self,
    transport::{Endpoint, Uri},
};

use tokio::runtime::{self, Runtime};


mod proto {
    tonic::include_proto!("talpid_openvpn_plugin");
}
use proto::openvpn_event_proxy_client::OpenvpnEventProxyClient;

/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: OpenvpnEventProxyClient<tonic::transport::Channel>,
    runtime: Runtime,
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor, Error> {
        log::trace!("Creating EventProcessor");
        let mut runtime = runtime::Builder::new()
            .basic_scheduler()
            .core_threads(1)
            .enable_all()
            .build()
            .map_err(Error::CreateRuntime)?;
        let ipc_client = runtime
            .block_on(Self::spawn_client(arguments.ipc_socket_path.clone()))
            .map_err(Error::CreateTransport)?;

        Ok(EventProcessor {
            ipc_client,
            runtime,
        })
    }

    async fn spawn_client(
        ipc_path: String,
    ) -> Result<OpenvpnEventProxyClient<tonic::transport::Channel>, tonic::transport::Error> {
        // The URI will be ignored
        let channel = Endpoint::from_static("lttp://[::]:50051")
            .connect_with_connector(service_fn(move |_: Uri| {
                IpcEndpoint::connect(ipc_path.clone())
            }))
            .await?;

        Ok(OpenvpnEventProxyClient::new(channel))
    }

    pub fn process_event(
        &mut self,
        event: openvpn_plugin::EventType,
        env: HashMap<String, String>,
    ) -> Result<(), Error> {
        log::debug!("Processing \"{:?}\" event", event);

        let future = self.ipc_client.event(proto::EventType {
            event: event as i16 as i32,
            env,
        });
        let response = self.runtime.block_on(future);
        response.map(|_| ()).map_err(Error::SendEvent)
    }
}
