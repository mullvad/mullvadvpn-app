#![cfg(feature = "integration-tests")]

use mullvad_tests::{
    mock_openvpn::search_openvpn_args, watch_event, DaemonRunner, MockOpenVpnPluginRpcClient,
    PathWatcher,
};
use std::{fs, path::Path, sync::mpsc, time::Duration};
use talpid_types::tunnel::TunnelStateTransition;

#[cfg(target_os = "linux")]
const OPENVPN_PLUGIN_NAME: &str = "libtalpid_openvpn_plugin.so";

#[cfg(windows)]
const OPENVPN_PLUGIN_NAME: &str = "talpid_openvpn_plugin.dll";

#[test]
fn spawns_openvpn() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();
}

#[test]
fn respawns_openvpn_if_it_crashes() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    openvpn_args_file_events.set_timeout(Duration::from_secs(10));

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();

    // Stop OpenVPN by removing the mock OpenVPN arguments file
    fs::remove_file(&openvpn_args_file).expect("Failed to remove the mock OpenVPN arguments file");
    assert_eq!(openvpn_args_file_events.next(), Some(watch_event::REMOVE));

    openvpn_args_file_events.assert_create_write_close_sequence();
}

#[test]
fn changes_to_connecting_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connecting);
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelStateTransition::Connecting
    );
}

#[test]
fn changes_to_connected_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connecting);
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connected);
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelStateTransition::Connected
    );
}

#[test]
fn returns_to_connecting_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connecting);
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connected);

    mock_plugin_client.route_predown().unwrap();

    // Wait for new OpenVPN instance
    assert_eq!(openvpn_args_file_events.next(), Some(watch_event::REMOVE));
    openvpn_args_file_events.assert_create_write_close_sequence();

    assert_state_event(&state_events, TunnelStateTransition::Connecting);
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelStateTransition::Connecting
    );
}

#[test]
fn disconnects() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.new_state_subscribe().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connecting);
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Connected);

    rpc_client.disconnect().unwrap();

    assert_state_event(&state_events, TunnelStateTransition::Disconnected);
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelStateTransition::Disconnected
    );
}

fn assert_state_event(
    receiver: &mpsc::Receiver<TunnelStateTransition>,
    expected_state: TunnelStateTransition,
) {
    let received_state = receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("Failed to receive new state event from daemon");

    assert_eq!(received_state, expected_state);
}

fn create_mock_openvpn_plugin_client<P: AsRef<Path>>(
    openvpn_args_file_path: P,
) -> MockOpenVpnPluginRpcClient {
    let address = get_plugin_arguments(openvpn_args_file_path);

    MockOpenVpnPluginRpcClient::new(address)
        .expect("Failed to create mock RPC client to connect to OpenVPN plugin event listener")
}

fn get_plugin_arguments<P: AsRef<Path>>(openvpn_args_file_path: P) -> String {
    let mut arguments = search_openvpn_args(openvpn_args_file_path, OPENVPN_PLUGIN_NAME).skip(1);

    arguments
        .next()
        .expect("Missing OpenVPN plugin RPC listener address argument")
        .expect("Failed to read from mock OpenVPN arguments file")
}
