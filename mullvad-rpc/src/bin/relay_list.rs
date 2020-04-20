/// Intended to be used to pre-load a relay list when creating an installer for the Mullvad VPN
/// app.
use futures01::future::Future;
use mullvad_rpc::{MullvadRpcRuntime, RelayListProxy};

fn main() {
    let mut runtime = MullvadRpcRuntime::new("dist-assets/api_root_ca.pem".as_ref())
        .expect("Failed to load runtime");

    let relay_list_request = RelayListProxy::new(runtime.mullvad_rest_handle()).relay_list_v3();

    let relay_list = relay_list_request
        .wait()
        .expect("Failed to fetch relay list");

    println!("{}", serde_json::to_string_pretty(&relay_list).unwrap());
}
