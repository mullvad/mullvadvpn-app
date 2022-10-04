use regex::Regex;
use std::{env, process::exit};

const ANDROID_VERSION: &str =
    include_str!(concat!(env!("OUT_DIR"), "/android-product-version.txt"));

fn main() {
    let command = env::args().nth(1);
    match command.as_deref() {
        None => println!("{}", mullvad_version::VERSION),
        Some("semver") => println!("{}", to_semver(mullvad_version::VERSION)),
        Some("versionName") => println!("{ANDROID_VERSION}"),
        Some("versionCode") => println!("{}", to_android_version_code(ANDROID_VERSION)),
        Some(command) => {
            eprintln!("Unknown command: {command}");
            exit(1);
        }
    }
}

/// Takes a version without a patch number and adds the patch (set to zero).
///
/// Converts `x.y[-z]` into `x.y.0[-z]` to make the version semver compatible.
fn to_semver(version: &str) -> String {
    let mut parts = version.splitn(2, '-');

    let version = parts.next().expect("Year component");
    let remainder = parts.next().map(|s| format!("-{s}")).unwrap_or_default();
    assert_eq!(parts.next(), None);

    format!("{version}.0{remainder}")
}

/// Takes a version in the normal Mullvad VPN app version format and returns the Android
/// `versionCode` formatted version.
///
/// The format of the code is:           YYVV00XX
/// Last two digits of the year (major)  ^^
///          Incrementing version (minor)  ^^
///                                  Unused  ^^
///                 Beta number, 00 if stable  ^^
///
/// # Example
///
/// Version: 2021.34-beta5
/// versionCode: 21340005
fn to_android_version_code(version: &str) -> String {
    let re =
        Regex::new(r"^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?(-dev-[0-9a-f]+)?$").unwrap();
    let captures = re
        .captures(version)
        .expect("Version does not match expected format");
    let year = captures.get(1).expect("Missing year").as_str();
    let incremental = captures.get(2).expect("Missing incremental").as_str();
    let beta = captures.get(4).map(|m| m.as_str()).unwrap_or_default();

    format!("{year}{:0>2}00{:0>2}", incremental, beta)
}
