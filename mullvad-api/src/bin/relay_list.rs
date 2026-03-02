//! Fetches and prints the full relay list in JSON.
//! Used by the installer artifact packer to bundle the latest available
//! relay list at the time of creating the installer.
//!
//! # Arguments
//! - `internal`: Mangle the relay list and output the representation internal to the `mullvad-types` crate.

#[cfg(not(target_os = "android"))]
mod imp {
    use mullvad_api::{
        ApiEndpoint, RelayListProxy, proxy::ApiConnectionMode, rest::Error as RestError,
    };
    use std::process;
    use talpid_types::ErrorExt;

    pub async fn main() {
        let api_endpoint = ApiEndpoint::from_env_vars();
        let runtime = mullvad_api::Runtime::new(tokio::runtime::Handle::current(), &api_endpoint);

        let relay_list_request = RelayListProxy::new(
            runtime.mullvad_rest_handle(ApiConnectionMode::Direct.into_provider()),
        )
        .relay_list(None)
        .await;

        let relay_list = match relay_list_request {
            Ok(relay_list) => relay_list,
            Err(RestError::TimeoutError) => {
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

        // Poor man's clap.
        if let Some(arg) = std::env::args().nth(1)
            && arg == "--internal"
        {
            println!(
                "{}",
                serde_json::to_string_pretty(&relay_list.unwrap().into_internal_repr()).unwrap()
            );
        } else {
            println!("{}", serde_json::to_string_pretty(&relay_list).unwrap());
        }
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
