use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use mullvad_paths;
use {Error, Result};

pub fn new_client() -> Result<DaemonRpcClient> {
    new_standalone_ipc_client(&mullvad_paths::get_rpc_socket_path()).map_err(|e| Error::from(e))
}
