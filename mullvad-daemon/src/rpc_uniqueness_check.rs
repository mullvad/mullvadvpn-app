use error_chain::ChainedError;

use log::Level;
use mullvad_ipc_client::new_standalone_ipc_client;
use mullvad_paths;


/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub fn is_another_instance_running() -> bool {
    match new_standalone_ipc_client(&mullvad_paths::get_rpc_socket_path()) {
        Ok(_) => true,
        Err(error) => {
            let msg =
                "Failed to locate/connect to another daemon instance, assuming there isn't one";
            if log_enabled!(Level::Debug) {
                debug!("{}\n{}", msg, error.display_chain());
            } else {
                info!("{}", msg);
            }
            false
        }
    }
}
