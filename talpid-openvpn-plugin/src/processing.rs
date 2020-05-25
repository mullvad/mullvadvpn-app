use super::{Arguments, Error};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use std::{collections::HashMap, convert::TryFrom};
use tower::service_fn;

use tonic::{
    self,
    transport::{Endpoint, Uri},
};

use tokio::runtime::Runtime;


pub mod proto {
    tonic::include_proto!("talpid_openvpn_plugin");
}
use proto::open_vpn_event_proxy_client::OpenVpnEventProxyClient;

/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: OpenVpnEventProxyClient<tonic::transport::Channel>,
    runtime: Runtime,
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor, Error> {
        log::trace!("Creating EventProcessor");
        let mut runtime = Runtime::new().expect("Failed to initialize runtime");
        let ipc_client = runtime.block_on(Self::spawn_client(arguments.ipc_socket_path.clone()));

        Ok(EventProcessor {
            ipc_client,
            runtime,
        })
    }

    async fn spawn_client(ipc_path: String) -> OpenVpnEventProxyClient<tonic::transport::Channel> {
        // The URI will be ignored
        // FIXME: do not unwrap
        let channel = Endpoint::try_from("lttp://[::]:50051")
            .unwrap()
            .connect_with_connector(service_fn(move |_: Uri| {
                IpcEndpoint::connect(ipc_path.clone())
            }))
            .await
            .unwrap();

        OpenVpnEventProxyClient::new(channel)
    }

    pub fn process_event(
        &mut self,
        event: openvpn_plugin::EventType,
        env: HashMap<String, String>,
    ) -> Result<(), Error> {
        log::debug!("Processing \"{:?}\" event", event);

        let future = self.ipc_client.openvpn_event(proto::EventType {
            event: event as i16 as i32,
            env: env.clone(),
        });
        let response = self.runtime.block_on(future);
        response.map(|_| ()).map_err(Error::SendEvent)
    }
}
