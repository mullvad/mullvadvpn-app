use std::result;

use mullvad_types::states::DaemonState;
use talpid_ipc::WsIpcClient;

use rpc_address_file;


/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub fn is_another_instance_running() -> bool {
    if let Ok(address) = rpc_address_file::read() {
        match call_other_instance(address) {
            Ok(_) => true,
            Err(message) => {
                info!("{}; assuming it has stopped", message);
                false
            }
        }
    } else {
        false
    }
}

fn call_other_instance(address: String) -> result::Result<(), String> {
    let method = "get_state";
    let args: [u8; 0] = [];
    // TODO: Authenticate with server
    let mut rpc_client =
        WsIpcClient::new(address).map_err(|_| "Failed to connect to other daemon")?;
    let _: DaemonState = rpc_client
        .call(method, &args)
        .map_err(|_| "Failed to execute RPC call to other daemon")?;
    Ok(())
}
