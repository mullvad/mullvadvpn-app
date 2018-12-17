#![cfg(feature = "integration-tests")]

use mullvad_tests::{mock_openvpn::search_openvpn_args, watch_event, DaemonRunner, PathWatcher};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[test]
fn uses_account_token() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    let specified_account = "123456";
    rpc_client
        .set_account(Some(specified_account.to_owned()))
        .unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();

    let account_token_sent_to_plugin = read_account_token(openvpn_args_file).unwrap();

    assert_eq!(account_token_sent_to_plugin, specified_account);
}

#[test]
fn uses_updated_account_token() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    let first_account_specified = "123456";
    rpc_client
        .set_account(Some(first_account_specified.to_owned()))
        .unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();

    let second_account_specified = "654321";
    rpc_client
        .set_account(Some(second_account_specified.to_owned()))
        .unwrap();

    assert_eq!(openvpn_args_file_events.next(), Some(watch_event::REMOVE));
    openvpn_args_file_events.assert_create_write_close_sequence();

    let account_token_sent_to_plugin = read_account_token(openvpn_args_file).unwrap();

    assert_eq!(account_token_sent_to_plugin, second_account_specified);
}

fn read_account_token<P: AsRef<Path>>(openvpn_args_file_path: P) -> Result<String, String> {
    let account_token_file_path = search_openvpn_args(openvpn_args_file_path, "--auth-user-pass")
        .skip(1)
        .next()
        .ok_or_else(|| "Missing account token file parameter to Talpid OpenVPN plugin".to_owned())?
        .map_err(|error| {
            format!(
                "Failed to read from mock OpenVPN command line file: {}",
                error
            )
        })?;

    let account_token_file = File::open(account_token_file_path)
        .map_err(|error| format!("Failed to open account token file: {}", error))?;

    let mut reader = BufReader::new(account_token_file);
    let mut account = String::new();

    reader
        .read_line(&mut account)
        .map_err(|error| format!("Failed to read from account token file: {}", error))?;

    Ok(account.trim().to_owned())
}
