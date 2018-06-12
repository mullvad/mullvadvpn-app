#![cfg(all(target_os = "linux", feature = "integration-tests"))]

extern crate mullvad_ipc_client;
extern crate mullvad_tests;
extern crate mullvad_types;

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use mullvad_tests::{wait_for_file_write_finish, DaemonRunner, MockOpenVpnPluginRpcClient};
use mullvad_types::states::{DaemonState, SecurityState, TargetState};

#[cfg(target_os = "linux")]
const OPENVPN_PLUGIN_NAME: &str = "libtalpid_openvpn_plugin.so";

#[cfg(windows)]
const OPENVPN_PLUGIN_NAME: &str = "talpid_openvpn_plugin.dll";

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

    assert_state_event(&state_events, CONNECTING_STATE);
    assert_eq!(rpc_client.get_state().unwrap(), CONNECTING_STATE);
}

#[test]
fn ignores_event_from_unauthorized_connection_from_openvpn_plugin() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, CONNECTING_STATE);

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);
    let call_result = mock_plugin_client.up();

    assert!(call_result.is_err());
    assert_no_state_event(&state_events);
    assert_eq!(rpc_client.get_state().unwrap(), CONNECTING_STATE);
}

#[test]
fn authentication_credentials() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, CONNECTING_STATE);

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    assert_eq!(
        mock_plugin_client.authenticate_with(&String::new()),
        Ok(false)
    );
    assert_eq!(
        mock_plugin_client.authenticate_with(&"fake-secret".to_owned()),
        Ok(false)
    );
    assert_eq!(mock_plugin_client.authenticate(), Ok(true));
    // Ensure it doesn't accept additional incorrect credentials
    assert_eq!(
        mock_plugin_client.authenticate_with(&"different-secret".to_owned()),
        Ok(false)
    );
}

#[test]
fn separate_connections_have_independent_authentication() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, CONNECTING_STATE);

    let mut first_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);
    let mut second_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    let auth_result = first_plugin_client.authenticate();
    let call_result = second_plugin_client.up();

    assert_eq!(auth_result, Ok(true));
    assert!(call_result.is_err());
    assert_no_state_event(&state_events);
    assert_eq!(rpc_client.get_state().unwrap(), CONNECTING_STATE);
}

fn assert_state_event(receiver: &mpsc::Receiver<DaemonState>, expected_state: DaemonState) {
    let received_state = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive new state event from daemon");

    assert_eq!(received_state, expected_state);
}

fn assert_no_state_event(receiver: &mpsc::Receiver<DaemonState>) {
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(1)),
        Err(mpsc::RecvTimeoutError::Timeout),
    );
}

fn create_mock_openvpn_plugin_client<P: AsRef<Path>>(
    openvpn_args_file_path: P,
) -> MockOpenVpnPluginRpcClient {
    let (address, credentials) = get_plugin_arguments(openvpn_args_file_path);

    MockOpenVpnPluginRpcClient::new(address, credentials)
        .expect("Failed to create mock RPC client to connect to OpenVPN plugin event listener")
}

fn get_plugin_arguments<P: AsRef<Path>>(openvpn_args_file_path: P) -> (String, String) {
    let args_file_path = openvpn_args_file_path.as_ref();

    wait_for_file_write_finish(&args_file_path, Duration::from_secs(5));

    let args_file = File::open(&args_file_path).expect(&format!(
        "Failed to open mock OpenVPN command-line file: {}",
        args_file_path.display(),
    ));

    let args_reader = BufReader::new(args_file).lines();
    let mut arguments = args_reader
        .skip_while(|element| {
            element.is_ok() && !element.as_ref().unwrap().contains(OPENVPN_PLUGIN_NAME)
        })
        .skip(1);

    let address = arguments
        .next()
        .expect("Missing OpenVPN plugin RPC listener address argument")
        .expect("Failed to read from mock OpenVPN arguments file");
    let credentials = arguments
        .next()
        .expect("Missing OpenVPN plugin RPC listener credentials argument")
        .expect("Failed to read from mock OpenVPN arguments file");

    (address, credentials)
}
