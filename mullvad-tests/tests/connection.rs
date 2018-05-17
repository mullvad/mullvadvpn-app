#![cfg(target_os = "linux")]

extern crate mullvad_tests;

use std::time::Duration;

use mullvad_tests::{wait_for_file_write_finish, DaemonRunner};

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
