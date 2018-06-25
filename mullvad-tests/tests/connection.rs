#![cfg(all(target_os = "linux", feature = "integration-tests"))]

extern crate mullvad_ipc_client;
extern crate mullvad_tests;
extern crate mullvad_types;

use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use mullvad_tests::{wait_for_file_write_finish, DaemonRunner};
use mullvad_types::states::{DaemonState, SecurityState, TargetState};

const CONNECTING_STATE: DaemonState = DaemonState {
    state: SecurityState::Unsecured,
    target_state: TargetState::Secured,
};

#[test]
fn spawns_openvpn() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    wait_for_file_write_finish(&openvpn_args_file, Duration::from_secs(5));

    assert!(openvpn_args_file.exists());
}

#[test]
fn respawns_openvpn_if_it_crashes() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    wait_for_file_write_finish(&openvpn_args_file, Duration::from_secs(5));

    // Stop OpenVPN by removing the mock OpenVPN arguments file
    fs::remove_file(&openvpn_args_file).expect("Failed to remove the mock OpenVPN arguments file");

    wait_for_file_write_finish(&openvpn_args_file, Duration::from_secs(5));

    assert!(openvpn_args_file.exists());
}

#[test]
fn changes_to_connecting_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(state_events, CONNECTING_STATE);
    assert_eq!(rpc_client.get_state().unwrap(), CONNECTING_STATE);
}

fn assert_state_event(receiver: mpsc::Receiver<DaemonState>, expected_state: DaemonState) {
    let received_state = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive new state event from daemon");

    assert_eq!(received_state, expected_state);
}
