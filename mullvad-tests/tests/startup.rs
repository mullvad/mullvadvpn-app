#![cfg(feature = "integration-tests")]

extern crate mullvad_paths;
extern crate mullvad_tests;
extern crate mullvad_types;

use mullvad_tests::DaemonRunner;
use mullvad_types::states::{DaemonState, SecurityState, TargetState};

#[test]
fn starts_in_not_connected_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().expect("Failed to create RPC client");

    let state = rpc_client.get_state().expect("Failed to read daemon state");
    let not_connected_state = DaemonState {
        state: SecurityState::Unsecured,
        target_state: TargetState::Unsecured,
    };

    assert_eq!(state, not_connected_state);
}
