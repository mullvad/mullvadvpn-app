use mullvad_version::Version;
use std::{env, process::exit};

const ANDROID_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/android-version-name.txt"));

fn main() {
    let command = env::args().nth(1);
    match command.as_deref() {
        None => println!("{}", mullvad_version::VERSION),
        Some("semver") => println!("{}", to_semver(mullvad_version::VERSION)),
        Some("version.h") => println!("{}", to_windows_h_format(mullvad_version::VERSION)),
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
///                 Beta number, 99 if stable  ^^
///
/// # Examples
///
/// Version: 2021.34-beta5
/// versionCode: 21340005
///
/// Version: 2021.34
/// versionCode: 21340099
fn to_android_version_code(version: &str) -> String {
    const ANDROID_STABLE_VERSION_CODE_SUFFIX: &str = "99";

    let version = Version::parse(version);
    format!(
        "{}{:0>2}00{:0>2}",
        version.year,
        version.incremental,
        version
            .beta
            .unwrap_or(ANDROID_STABLE_VERSION_CODE_SUFFIX.to_string())
    )
}

fn to_windows_h_format(version: &str) -> String {
    let Version {
        year, incremental, ..
    } = Version::parse(version);

    format!(
        "#define MAJOR_VERSION 20{year}
#define MINOR_VERSION {incremental}
#define PATCH_VERSION 0
#define PRODUCT_VERSION \"{version}\""
    )
}
