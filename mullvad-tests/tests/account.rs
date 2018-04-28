#![cfg(all(target_os = "linux", feature = "integration-tests"))]

extern crate mullvad_tests;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;

use mullvad_tests::mock_openvpn::search_openvpn_args;
use mullvad_tests::{wait_for_file_write_finish, DaemonRunner};

#[test]
fn uses_account_token() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();

    let account = "123456";
    rpc_client.set_account(Some(account.to_owned())).unwrap();
    rpc_client.connect().unwrap();

    let account_token = read_account_token(openvpn_args_file).unwrap();

    assert_eq!(account_token, account);
}

fn read_account_token<P: AsRef<Path>>(openvpn_args_file_path: P) -> Result<String, String> {
    let args_file_path = openvpn_args_file_path.as_ref();

    wait_for_file_write_finish(&args_file_path, Duration::from_secs(5));

    let account_token_file_path = search_openvpn_args(&args_file_path, "--auth-user-pass")
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
