#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;


#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_ipc;

extern crate futures;
extern crate mullvad_paths;
extern crate mullvad_types;
extern crate serde;
extern crate talpid_ipc;
extern crate talpid_types;
extern crate tokio_core;

use std::sync::mpsc;

use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::location::GeoIpLocation;
use mullvad_types::relay_constraints::{RelaySettings, RelaySettingsUpdate};
use mullvad_types::relay_list::RelayList;
use mullvad_types::states::DaemonState;
use mullvad_types::version::AppVersionInfo;
use serde::{Deserialize, Serialize};
use talpid_types::net::TunnelOptions;

use jsonrpc_client_core::{Client, ClientHandle, Future};
pub use jsonrpc_client_core::{Error as RpcError, ErrorKind as RpcErrorKind};
use jsonrpc_client_ipc::IpcTransport;
use tokio_core::reactor;

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
    links {
        UnknownRpcAddressPath(mullvad_paths::Error, mullvad_paths::ErrorKind);
    }
}

static NO_ARGS: [u8; 0] = [];

fn new_standalone_transport<
    F: Send + 'static + FnOnce(String, reactor::Handle) -> Result<T>,
    T: jsonrpc_client_core::Transport,
>(
    rpc_path: String,
    transport_func: F,
) -> Result<DaemonRpcClient> {
    use futures::sync::oneshot;
    use std::thread;
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || match spawn_transport(rpc_path, transport_func) {
        Err(e) => tx
            .send(Err(e))
            .expect("Failed to send error back to caller"),
        Ok((mut core, client, client_handle)) => {
            tx.send(Ok(client_handle))
                .expect("Failed to send client handle");
            if let Err(e) = core.run(client) {
                error!("JSON-RPC client failed: {}", e.description());
            }
        }
    });

    let client_handle = rx.wait().chain_err(|| ErrorKind::TransportError)??;
    DaemonRpcClient::new(client_handle)
}

pub fn new_standalone_ipc_client() -> Result<DaemonRpcClient> {
    let path = mullvad_paths::get_rpc_socket_path()
        .to_string_lossy()
        .to_string();
    new_standalone_transport(path, |path, handle| {
        let result = IpcTransport::new(&path, &handle);
        match result {
            Ok(t) => Ok(t),
            Err(e) => {
                use std::error::Error;
                println!("path - {}", &path);
                println!(
                    "encountered error whilst setting up transport - {}",
                    e.description()
                );
                Err(e).chain_err(|| ErrorKind::TransportError)
            }
        }
    })
}

fn spawn_transport<
    F: Send + FnOnce(String, reactor::Handle) -> Result<T>,
    T: jsonrpc_client_core::Transport,
>(
    address: String,
    transport_func: F,
) -> Result<(reactor::Core, Client<T>, ClientHandle)> {
    let core = reactor::Core::new().chain_err(|| ErrorKind::TokioError)?;
    let (client, client_handle) = transport_func(address, core.handle())?.into_client();
    Ok((core, client, client_handle))
}

pub struct DaemonRpcClient {
    rpc_client: jsonrpc_client_core::ClientHandle,
    methods: DaemonMethods,
}


impl DaemonRpcClient {
    pub fn new(rpc_client: ClientHandle) -> Result<Self> {
        let methods = DaemonMethods::new(rpc_client.clone());

        Ok(DaemonRpcClient {
            rpc_client,
            methods,
        })
    }

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

    pub fn get_allow_lan(&mut self) -> Result<bool> {
        self.call("get_allow_lan", &NO_ARGS)
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<()> {
        self.call("set_auto_connect", &[auto_connect])
    }

    pub fn get_auto_connect(&mut self) -> Result<bool> {
        self.call("get_auto_connect", &NO_ARGS)
    }

    pub fn get_current_location(&mut self) -> Result<GeoIpLocation> {
        self.call("get_current_location", &NO_ARGS)
    }

    pub fn get_current_version(&mut self) -> Result<String> {
        self.call("get_current_version", &NO_ARGS)
    }

    pub fn get_relay_locations(&mut self) -> Result<RelayList> {
        self.call("get_relay_locations", &NO_ARGS)
    }

    pub fn get_relay_settings(&mut self) -> Result<RelaySettings> {
        self.call("get_relay_settings", &NO_ARGS)
    }

    pub fn get_state(&mut self) -> Result<DaemonState> {
        self.call("get_state", &NO_ARGS)
    }

    pub fn get_tunnel_options(&mut self) -> Result<TunnelOptions> {
        self.call("get_tunnel_options", &NO_ARGS)
    }

    pub fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        self.call("get_version_info", &NO_ARGS)
    }

    pub fn set_account(&mut self, account: Option<AccountToken>) -> Result<()> {
        self.call("set_account", &[account])
    }

    pub fn set_openvpn_enable_ipv6(&mut self, enabled: bool) -> Result<()> {
        self.call("set_openvpn_enable_ipv6", &[enabled])
    }

    pub fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Result<()> {
        self.call("set_openvpn_mssfix", &[mssfix])
    }

    pub fn from_client_handle(rpc_client: ClientHandle) -> Self {
        let methods = DaemonMethods::new(rpc_client.clone());
        DaemonRpcClient {
            rpc_client,
            methods,
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.call("shutdown", &NO_ARGS)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<()> {
        self.call("update_relay_settings", &[update])
    }


    pub fn methods(&mut self) -> &mut DaemonMethods {
        &mut self.methods
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

    pub fn new_state_subscribe(&mut self) -> Result<mpsc::Receiver<DaemonState>> {
        self.subscribe("new_state")
    }

    pub fn subscribe<T>(&mut self, _event: &str) -> Result<mpsc::Receiver<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let (_event_tx, event_rx) = mpsc::channel();
        // TODO Add subscription support back. Currently subscription stream will be always empty
        // let subscribe_method = format!("{}_subscribe", event);
        // let unsubscribe_method = format!("{}_unsubscribe", event);

        // self.rpc_client
        //     .subscribe::<T, T>(subscribe_method, unsubscribe_method, event_tx)
        //     .chain_err(|| ErrorKind::RpcSubscribeError(event.to_owned()))?;

        Ok(event_rx)
    }
}
jsonrpc_client!{pub struct DaemonMethods{

    pub fn connect(&mut self) -> Future<()>;

    pub fn disconnect(&mut self) -> Future<()>;

    pub fn get_account(&mut self) -> Future<Option<AccountToken>>;

    pub fn get_account_data(&mut self, account: AccountToken) -> Future<AccountData>;

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Future<()>;

    pub fn get_allow_lan(&mut self) -> Future<bool>;

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Future<()>;

    pub fn get_auto_connect(&mut self) -> Future<bool>;

    pub fn get_current_location(&mut self) -> Future<GeoIpLocation>;

    pub fn get_current_version(&mut self) -> Future<String>;

    pub fn get_relay_locations(&mut self) -> Future<RelayList>;

    pub fn get_relay_settings(&mut self) -> Future<RelaySettings>;

    pub fn get_state(&mut self) -> Future<DaemonState>;

    pub fn get_tunnel_options(&mut self) -> Future<TunnelOptions>;

    pub fn get_version_info(&mut self) -> Future<AppVersionInfo>;

    pub fn set_account(&mut self, account: Option<AccountToken>) ->  Future<()>;

    pub fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Future<()>;

    pub fn shutdown(&mut self) -> Future<()>;

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Future<()>;
}}
