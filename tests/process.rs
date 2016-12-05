extern crate talpid_core;

mod util;

use talpid_core::process::OpenVpnBuilder;

#[cfg(target_os = "linux")]
#[test]
fn check_test_environment() {
    use std::env;
    let test_threads = env::var("RUST_TEST_THREADS");
    if !test_threads.is_ok() || test_threads.unwrap() != "1" {
        panic!("Tests must be run with environment variable RUST_TEST_THREADS=1");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn openvpn_builder_starts_correct_process() {
    let mut child = OpenVpnBuilder::new("echo").spawn().unwrap();
    let args = util::read_args_for_proc(child.id());

    assert_eq!(vec!["echo"], args);
    child.kill().unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn openvpn_builder_passes_config() {
    let config_path = "/path/to/config".to_owned();
    let mut child = OpenVpnBuilder::new("echo").config(&config_path).spawn().unwrap();
    let args = util::read_args_for_proc(child.id());

    assert!(args.contains(&config_path));
    child.kill().unwrap();
}
