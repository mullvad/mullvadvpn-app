/// Intended to be used to pre-load a relay list when creating an installer for the Mullvad VPN
/// app.
use mullvad_rpc::{rest::Error as RestError, MullvadRpcRuntime, RelayListProxy};
use std::process;
use talpid_types::ErrorExt;

#[tokio::main]
async fn main() {
    let mut runtime = MullvadRpcRuntime::new().expect("Failed to load runtime");

    let relay_list_request = RelayListProxy::new(runtime.mullvad_rest_handle())
        .relay_list(None)
        .await;

    let relay_list = match relay_list_request {
        Ok(relay_list) => relay_list,
        Err(RestError::TimeoutError(_)) => {
            eprintln!("Request timed out");
            process::exit(2);
        }
        Err(e @ RestError::DeserializeError(_)) => {
            eprintln!(
                "{}",
                e.display_chain_with_msg("Failed to deserialize relay list")
            );
            process::exit(3);
        }
        Err(e) => {
            eprintln!("{}", e.display_chain_with_msg("Failed to fetch relay list"));
            process::exit(1);
        }
    };
    println!("{}", serde_json::to_string_pretty(&relay_list).unwrap());
}
