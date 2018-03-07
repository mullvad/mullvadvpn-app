pub mod rpc_info;


use std::result;

use mullvad_types::states::DaemonState;
use talpid_ipc::WsIpcClient;


/// Verifies if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub fn other_instance_is_running() -> bool {
    rpc_info::exists() && other_daemon_responds()
}

fn other_daemon_responds() -> bool {
    if let Err(message) = call_other_daemon() {
        info!("{}; assuming it has stopped", message);
        return false;
    } else {
        return true;
    }
}

fn call_other_daemon() -> result::Result<(), String> {
    let method = "get_state";
    let args: [u8; 0] = [];
    let address = rpc_info::read().map_err(|_| "Failed to read RPC address file of other daemon")?;
    // TODO: Authenticate with server
    let mut rpc_client =
        WsIpcClient::new(address).map_err(|_| "Failed to connect to other daemon")?;
    let _: DaemonState = rpc_client
        .call(method, &args)
        .map_err(|_| "Failed to execute RPC call to other daemon")?;
    Ok(())
}
