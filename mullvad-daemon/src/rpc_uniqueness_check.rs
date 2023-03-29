use mullvad_management_interface::MullvadProxyClient;
use talpid_types::ErrorExt;

/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub async fn is_another_instance_running() -> bool {
    match MullvadProxyClient::new().await {
        Ok(_) => true,
        Err(error) => {
            let msg =
                "Failed to locate/connect to another daemon instance, assuming there isn't one";
            if log::log_enabled!(log::Level::Trace) {
                log::trace!("{}\n{}", msg, error.display_chain());
            } else {
                log::debug!("{}", msg);
            }
            false
        }
    }
}
