//! Fetches and prints the full relay list in JSON.
//! Used by the installer artifact packer to bundle the latest available
//! relay list at the time of creating the installer.

#[cfg(not(target_os = "android"))]
mod imp {
    use mullvad_api::{
        ApiEndpoint, Error, RelayListProxy, proxy::ApiConnectionMode, relay_list_transparency,
        rest::Error as RestError,
    };
    use std::process;
    use talpid_types::ErrorExt;

    pub async fn main() {
        let api_endpoint = ApiEndpoint::from_env_vars();
        let runtime = mullvad_api::Runtime::new(tokio::runtime::Handle::current(), &api_endpoint);

        let proxy = RelayListProxy::new(
            runtime.mullvad_rest_handle(ApiConnectionMode::Direct.into_provider()),
        );

        let response =
            relay_list_transparency::download_and_verify_relay_list(proxy, None, None).await;

        let relay_list = match response {
            Ok(relay_list) => relay_list,
            Err(Error::RestError(RestError::TimeoutError)) => {
                eprintln!("Request timed out");
                process::exit(2);
            }
            Err(e @ Error::RestError(RestError::DeserializeError(_))) => {
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
}

#[tokio::main]
async fn main() {
    imp::main().await
}

#[cfg(target_os = "android")]
mod imp {
    pub async fn main() {}
}
