/// Generate a first list of IP addresses for Mullvad VPN to use to talk to the API.
use mullvad_rpc::{rest::Error as RestError, ApiProxy, MullvadRpcRuntime};
use std::process;
use talpid_types::ErrorExt;

#[tokio::main]
async fn main() {
    let mut runtime =
        MullvadRpcRuntime::new(tokio::runtime::Handle::current()).expect("Failed to load runtime");

    let api_proxy = ApiProxy::new(runtime.mullvad_rest_handle());
    let request = api_proxy.get_api_addrs().await;

    let api_list = match request {
        Ok(api_list) => api_list,
        Err(RestError::TimeoutError(_)) => {
            eprintln!("Request timed out");
            process::exit(2);
        }
        Err(e @ RestError::DeserializeError(_)) => {
            eprintln!(
                "{}",
                e.display_chain_with_msg("Failed to deserialize API address list")
            );
            process::exit(3);
        }
        Err(e) => {
            eprintln!(
                "{}",
                e.display_chain_with_msg("Failed to fetch API address list")
            );
            process::exit(1);
        }
    };

    for address in api_list {
        println!("{}", address);
    }
}
