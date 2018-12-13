#![cfg(feature = "integration-tests")]

use mullvad_tests::DaemonRunner;
use talpid_types::tunnel::TunnelStateTransition;

#[test]
fn starts_in_disconnected_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().expect("Failed to create RPC client");

    let state = rpc_client.get_state().expect("Failed to read daemon state");

    assert_eq!(state, TunnelStateTransition::Disconnected);
}
