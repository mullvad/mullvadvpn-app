#![cfg(target_os = "linux")]

extern crate mullvad_tests;

use std::time::Duration;

use mullvad_tests::{wait_for_file, DaemonRunner};

#[test]
fn spawns_openvpn() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_command_line_file = daemon.mock_openvpn_command_line_file();

    assert!(!openvpn_command_line_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    wait_for_file(&openvpn_command_line_file, Duration::from_secs(5));

    assert!(openvpn_command_line_file.exists());
}
