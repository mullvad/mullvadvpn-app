use error_chain::ChainedError;

use mullvad_ipc_client::DaemonRpcClient;


/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub fn is_another_instance_running() -> bool {
    match DaemonRpcClient::new() {
        Ok(client) => match client.get_state() {
            Ok(_) => true,
            Err(error) => {
                let chained_error = error.chain_err(|| {
                    "Failed to communicate with another daemon instance, assuming it has stopped"
                });
                info!("{}", chained_error.display_chain());
                false
            }
        },
        Err(error) => {
            let chained_error = error.chain_err(|| {
                "Failed to load RPC address for another daemon instance, assuming there isn't one"
            });
            debug!("{}", chained_error.display_chain());
            false
        }
    }
}
