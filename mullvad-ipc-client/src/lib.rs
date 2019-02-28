#[macro_use]
extern crate error_chain;

use futures::sync::oneshot;
use jsonrpc_client_core::{Client, ClientHandle, Future};
use jsonrpc_client_ipc::IpcTransport;
use mullvad_types::{
    account::{AccountData, AccountToken},
    location::GeoIpLocation,
    relay_constraints::{RelaySettings, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::{Settings, TunnelOptions},
    version::AppVersionInfo,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, thread};
use talpid_types::{
    net::{openvpn, wireguard},
    tunnel::TunnelStateTransition,
};

pub use jsonrpc_client_core::{Error as RpcError, ErrorKind as RpcErrorKind};

error_chain! {
    errors {
        AuthenticationError {
            description("Failed to authenticate the connection with the daemon")
        }

        RpcCallError(method: String) {
            description("Failed to call RPC method")
            display("Failed to call RPC method \"{}\"", method)
        }

        RpcSubscribeError(event: String) {
            description("Failed to subscribe to RPC event")
            display("Failed to subscribe to RPC event \"{}\"", event)
        }

        StartRpcClient(address: String) {
            description("Failed to start RPC client")
            display("Failed to start RPC client to {}", address)
        }

        TokioError {
            description("Failed to setup a standalone event loop")
        }

        TransportError {
            description("Failed to setup a transport")
        }
    }
}

static NO_ARGS: [u8; 0] = [];


pub fn new_standalone_ipc_client(path: &impl AsRef<Path>) -> Result<DaemonRpcClient> {
    let path = path.as_ref().to_string_lossy().to_string();

    new_standalone_transport(path, |path| {
        IpcTransport::new(&path, &tokio::reactor::Handle::default())
            .chain_err(|| ErrorKind::TransportError)
    })
}

pub fn new_standalone_transport<
    F: Send + 'static + FnOnce(String) -> Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    rpc_path: String,
    transport_func: F,
) -> Result<DaemonRpcClient> {
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

    rx.wait().chain_err(|| ErrorKind::TransportError)?.map(
        |(rpc_client, server_handle, executor)| {
            let subscriber =
                jsonrpc_client_pubsub::Subscriber::new(executor, rpc_client.clone(), server_handle);
            DaemonRpcClient {
                rpc_client,
                subscriber,
            }
        },
    )
}

fn spawn_transport<
    F: Send + FnOnce(String) -> Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    address: String,
    transport_func: F,
) -> Result<(
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
    rpc_client: jsonrpc_client_core::ClientHandle,
    subscriber: jsonrpc_client_pubsub::Subscriber<tokio::runtime::current_thread::Handle>,
}


impl DaemonRpcClient {
    pub fn connect(&mut self) -> Result<()> {
        self.call("connect", &NO_ARGS)
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.call("disconnect", &NO_ARGS)
    }

    pub fn get_account(&mut self) -> Result<Option<AccountToken>> {
        self.call("get_account", &NO_ARGS)
    }

    pub fn get_account_data(&mut self, account: AccountToken) -> Result<AccountData> {
        self.call("get_account_data", &[account])
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<()> {
        self.call("set_allow_lan", &[allow_lan])
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

    pub fn get_state(&mut self) -> Result<TunnelStateTransition> {
        self.call("get_state", &NO_ARGS)
    }

    pub fn get_tunnel_options(&mut self) -> Result<TunnelOptions> {
        self.call("get_tunnel_options", &NO_ARGS)
    }

    pub fn get_settings(&mut self) -> Result<Settings> {
        self.call("get_settings", &NO_ARGS)
    }

    pub fn generate_wireguard_key(&mut self) -> Result<()> {
        self.call("generate_wireguard_key", &NO_ARGS)
    }

    pub fn get_wireguard_key(&mut self) -> Result<Option<wireguard::PublicKey>> {
        self.call("get_wireguard_key", &NO_ARGS)
    }

    pub fn verify_wireguard_key(&mut self) -> Result<bool> {
        self.call("verify_wireguard_key", &NO_ARGS)
    }

    pub fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        self.call("get_version_info", &NO_ARGS)
    }

    pub fn set_account(&mut self, account: Option<AccountToken>) -> Result<()> {
        self.call("set_account", &[account])
    }

    pub fn set_enable_ipv6(&mut self, enabled: bool) -> Result<()> {
        self.call("set_enable_ipv6", &[enabled])
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<()> {
        self.call("set_wireguard_mtu", &[mtu])
    }

    pub fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Result<()> {
        self.call("set_openvpn_mssfix", &[mssfix])
    }

    pub fn set_openvpn_proxy(&mut self, proxy: Option<openvpn::ProxySettings>) -> Result<()> {
        self.call("set_openvpn_proxy", &[proxy])
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.call("shutdown", &NO_ARGS)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<()> {
        self.call("update_relay_settings", &[update])
    }

    pub fn call<A, O>(&mut self, method: &'static str, args: &A) -> Result<O>
    where
        A: Serialize + Send + 'static,
        O: for<'de> Deserialize<'de> + Send + 'static,
    {
        self.rpc_client
            .call_method(method, args)
            .wait()
            .chain_err(|| ErrorKind::RpcCallError(method.to_owned()))
    }

    pub fn new_state_subscribe(
        &mut self,
    ) -> impl Future<
        Item = jsonrpc_client_pubsub::Subscription<TunnelStateTransition>,
        Error = jsonrpc_client_pubsub::Error,
    > {
        self.subscriber.subscribe(
            "new_state_subscribe".to_string(),
            "new_state_unsubscribe".to_string(),
            "new_state".to_string(),
            0,
            &NO_ARGS,
        )
    }
}
