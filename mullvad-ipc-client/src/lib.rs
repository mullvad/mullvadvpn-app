#![deny(rust_2018_idioms)]

use futures::sync::oneshot;
use jsonrpc_client_core::{Client, ClientHandle, Future};
use jsonrpc_client_ipc::IpcTransport;
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    location::GeoIpLocation,
    relay_constraints::{BridgeSettings, BridgeState, RelaySettings, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::{Settings, TunnelOptions},
    states::TunnelState,
    version::AppVersionInfo,
    wireguard, DaemonEvent,
};
use serde::{Deserialize, Serialize};
use std::{io, path::Path, thread};

static NO_ARGS: [u8; 0] = [];

pub type Result<T> = std::result::Result<T, jsonrpc_client_core::Error>;
pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_pubsub::Error as PubSubError;

mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}
use proto::management_service_client::ManagementServiceClient;
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tower::service_fn;
use tonic::{
    self,
    transport::{Endpoint, Uri},
};

pub fn new_standalone_ipc_client(path: &impl AsRef<Path>) -> io::Result<DaemonRpcClient> {
    let path = path.as_ref().to_string_lossy().to_string();

    new_standalone_transport(path, |path| {
        IpcTransport::new(&path, &tokio::reactor::Handle::default())
    })
}

pub fn new_standalone_transport<
    F: Send + 'static + FnOnce(String) -> io::Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    rpc_path: String,
    transport_func: F,
) -> io::Result<DaemonRpcClient> {
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || match spawn_transport(rpc_path, transport_func) {
        Err(e) => tx
            .send(Err(e))
            .expect("Failed to send error back to caller"),
        Ok((client, server_handle, client_handle)) => {
            let mut rt = tokio::runtime::current_thread::Runtime::new()
                .expect("Failed to start a standalone tokio runtime for mullvad ipc");
            let handle = rt.handle();
            tx.send(Ok((client_handle, server_handle, handle)))
                .expect("Failed to send client handle");

            if let Err(e) = rt.block_on(client) {
                log::error!("JSON-RPC client failed: {}", e.description());
            }
        }
    });

    rx.wait()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "No transport handles returned"))?
        .map(|(rpc_client, server_handle, executor)| {
            let subscriber =
                jsonrpc_client_pubsub::Subscriber::new(executor, rpc_client.clone(), server_handle);

            // FIXME: spawning gRPC client here, temporarily
            let mut runtime = tokio02::runtime::Builder::new()
                .basic_scheduler()
                .core_threads(1)
                .enable_all()
                .build()
                .expect("FIXME: failed to create runtime");

            // FIXME: don't unwrap
            let grpc_client = runtime.block_on(spawn_grpc_client()).unwrap();

            DaemonRpcClient {
                runtime,
                grpc_client,
                rpc_client,
                subscriber,
            }
        })
}

async fn spawn_grpc_client() -> std::result::Result<ManagementServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    // FIXME
    //let ipc_path = "//./pipe/Mullvad VPNQWE";
    let ipc_path = std::path::PathBuf::from("/var/run/mullvad-vpnQWE");

    let channel = Endpoint::from_static("lttp://[::]:50051")
        .connect_with_connector(service_fn(move |_: Uri| {
            IpcEndpoint::connect(ipc_path.clone())
        }))
        .await?;

    Ok(ManagementServiceClient::new(channel))
}

fn spawn_transport<
    F: Send + FnOnce(String) -> io::Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    address: String,
    transport_func: F,
) -> io::Result<(
    Client<T, jsonrpc_client_core::server::Server>,
    jsonrpc_client_core::server::ServerHandle,
    ClientHandle,
)> {
    let (server, server_handle) = jsonrpc_client_core::server::Server::new();
    let transport = transport_func(address)?;
    let (client, client_handle) = jsonrpc_client_core::Client::with_server(transport, server);
    Ok((client, server_handle, client_handle))
}

pub struct DaemonRpcClient {
    runtime: tokio02::runtime::Runtime,
    grpc_client: ManagementServiceClient<tonic::transport::Channel>,
    rpc_client: jsonrpc_client_core::ClientHandle,
    subscriber: jsonrpc_client_pubsub::Subscriber<tokio::runtime::current_thread::Handle>,
}


impl DaemonRpcClient {
    pub fn connect(&mut self) -> Result<()> {
        if let Err(e) = self.runtime.block_on(self.grpc_client.connect_daemon(())) {
            // TODO: Fix return type
            log::error!("command failed: {}", e);
            return Err(jsonrpc_client_core::ErrorKind::TransportError.into());
        }
        Ok(())
        //self.call("connect", &NO_ARGS)
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if let Err(e) = self.runtime.block_on(self.grpc_client.disconnect_daemon(())) {
            // TODO: Fix return type
            log::error!("command failed: {}", e);
            return Err(jsonrpc_client_core::ErrorKind::TransportError.into());
        }
        Ok(())
        //self.call("disconnect", &NO_ARGS)
    }

    pub fn reconnect(&mut self) -> Result<()> {
        self.call("reconnect", &NO_ARGS)
    }

    pub fn create_new_account(&mut self) -> Result<()> {
        self.call("create_new_account", &NO_ARGS)
    }

    pub fn get_account(&mut self) -> Result<Option<AccountToken>> {
        self.call("get_account", &NO_ARGS)
    }

    pub fn get_account_data(&mut self, account: AccountToken) -> Result<AccountData> {
        self.call("get_account_data", &[account])
    }

    pub fn submit_voucher(&mut self, voucher: String) -> Result<VoucherSubmission> {
        self.call("submit_voucher", &[voucher])
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<()> {
        self.call("set_allow_lan", &[allow_lan])
    }

    pub fn set_show_beta_releases(&mut self, enabled: bool) -> Result<()> {
        self.call("set_show_beta_releases", &[enabled])
    }

    pub fn set_block_when_disconnected(&mut self, block_when_disconnected: bool) -> Result<()> {
        self.call("set_block_when_disconnected", &[block_when_disconnected])
    }

    pub fn get_allow_lan(&mut self) -> Result<bool> {
        self.call("get_allow_lan", &NO_ARGS)
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<()> {
        self.call("set_auto_connect", &[auto_connect])
    }

    pub fn get_auto_connect(&mut self) -> Result<bool> {
        self.call("get_auto_connect", &NO_ARGS)
    }

    pub fn get_current_location(&mut self) -> Result<Option<GeoIpLocation>> {
        self.call("get_current_location", &NO_ARGS)
    }

    pub fn get_current_version(&mut self) -> Result<String> {
        self.call("get_current_version", &NO_ARGS)
    }

    pub fn get_relay_locations(&mut self) -> Result<RelayList> {
        self.call("get_relay_locations", &NO_ARGS)
    }

    pub fn update_relay_locations(&mut self) -> Result<()> {
        self.call("update_relay_locations", &NO_ARGS)
    }

    pub fn get_relay_settings(&mut self) -> Result<RelaySettings> {
        self.call("get_relay_settings", &NO_ARGS)
    }

    pub fn get_state(&mut self) -> Result<TunnelState> {
        self.call("get_state", &NO_ARGS)
    }

    pub fn get_tunnel_options(&mut self) -> Result<TunnelOptions> {
        self.call("get_tunnel_options", &NO_ARGS)
    }

    pub fn get_settings(&mut self) -> Result<Settings> {
        self.call("get_settings", &NO_ARGS)
    }

    pub fn generate_wireguard_key(&mut self) -> Result<wireguard::KeygenEvent> {
        self.call("generate_wireguard_key", &NO_ARGS)
    }

    pub fn get_wireguard_key(&mut self) -> Result<Option<wireguard::PublicKey>> {
        self.call("get_wireguard_key", &NO_ARGS)
    }

    pub fn verify_wireguard_key(&mut self) -> Result<bool> {
        self.call("verify_wireguard_key", &NO_ARGS)
    }

    pub fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        match self.runtime.block_on(self.grpc_client.get_version_info(())) {
            Ok(response) => {
                let info = response.into_inner();
                Ok(AppVersionInfo {
                    supported: info.supported,
                    latest_stable: info.latest_stable,
                    latest_beta: info.latest_beta,
                    suggested_upgrade: Some(info.suggested_upgrade),
                })
            }
            Err(e) => {
                // TODO: Fix return type
                log::error!("command failed: {}", e);
                Err(jsonrpc_client_core::ErrorKind::TransportError.into())
            }
        }
    }

    pub fn set_account(&mut self, account: Option<AccountToken>) -> Result<()> {
        self.call("set_account", &[account])
    }

    pub fn clear_account_history(&mut self) -> Result<()> {
        self.call("clear_account_history", &NO_ARGS)
    }

    pub fn set_enable_ipv6(&mut self, enabled: bool) -> Result<()> {
        self.call("set_enable_ipv6", &[enabled])
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<()> {
        self.call("set_wireguard_mtu", &[mtu])
    }

    pub fn set_wireguard_rotation_interval(&mut self, interval: Option<u32>) -> Result<()> {
        self.call("set_wireguard_rotation_interval", &[interval])
    }

    pub fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Result<()> {
        self.call("set_openvpn_mssfix", &[mssfix])
    }

    pub fn set_bridge_settings(&mut self, settings: BridgeSettings) -> Result<()> {
        self.call("set_bridge_settings", &[settings])
    }

    pub fn set_bridge_state(&mut self, state: BridgeState) -> Result<()> {
        self.call("set_bridge_state", &[state])
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.call("shutdown", &NO_ARGS)
    }

    pub fn prepare_restart(&mut self) -> Result<()> {
        self.call("prepare_restart", &NO_ARGS)
    }

    pub fn factory_reset(&mut self) -> Result<()> {
        self.call("factory_reset", &NO_ARGS)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<()> {
        self.call("update_relay_settings", &[update])
    }

    pub fn get_split_tunnel_processes(&mut self) -> Result<Vec<i32>> {
        self.call("get_split_tunnel_processes", &NO_ARGS)
    }

    pub fn add_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.call("add_split_tunnel_process", &[pid])
    }

    pub fn remove_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.call("remove_split_tunnel_process", &[pid])
    }

    pub fn clear_split_tunnel_processes(&mut self) -> Result<()> {
        self.call("clear_split_tunnel_processes", &NO_ARGS)
    }


    pub fn call<A, O>(&mut self, method: &'static str, args: &A) -> Result<O>
    where
        A: Serialize + Send + 'static,
        O: for<'de> Deserialize<'de> + Send + 'static,
    {
        self.rpc_client.call_method(method, args).wait()
    }

    pub fn daemon_event_subscribe(
        &mut self,
    ) -> impl Future<
        Item = jsonrpc_client_pubsub::Subscription<DaemonEvent>,
        Error = jsonrpc_client_pubsub::Error,
    > {
        self.subscriber.subscribe(
            "daemon_event_subscribe".to_string(),
            "daemon_event_unsubscribe".to_string(),
            "daemon_event".to_string(),
            0,
            &NO_ARGS,
        )
    }
}
