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
        call_other_instance(address).is_ok()
    } else {
        false
    }
}

fn call_other_instance(address: String) -> result::Result<(), ()> {
    let method = "get_state";
    let args: [u8; 0] = [];
    // TODO: Authenticate with server
    let mut rpc_client = check_result(
        WsIpcClient::new(address),
        "Failed to connect to other daemon",
    )?;
    let _: DaemonState = check_result(
        rpc_client.call(method, &args),
        "Failed to execute RPC call to other daemon",
    )?;
    Ok(())
}

fn check_result<T, E>(result: Result<T, E>, message: &'static str) -> Result<T, ()> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => {
            info!("{}; assuming it has stopped", message);
            Err(())
        }
    }
}
