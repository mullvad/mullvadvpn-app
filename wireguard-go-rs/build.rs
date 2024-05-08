use core::{panic, str};
use std::{env, path::PathBuf};
// use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Missing OUT_DIR");
    eprintln!("OUT_DIR: {out_dir}");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Missing 'CARGO_CFG_TARGET_OS");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("./build-wireguard-go.sh");

    match target_os.as_str() {
        "linux" => {}
        "android" => {
            cmd.arg("--android").arg("--no-docker");
        }
        _ => unimplemented!("building wireguard-go-rs for {target_os} is not implemented."),
    }

    let output = cmd.output().expect("build-wireguard-go.sh failed");
    if !output.status.success() {
        let stdout = str::from_utf8(&output.stdout).unwrap();
        let stderr = str::from_utf8(&output.stderr).unwrap();
        eprintln!("build-wireguard-go.sh failed.");
        eprintln!("stdout:\n{stdout}");
        eprintln!("stderr:\n{stderr}");
        panic!();
    }

    // declare_libs_dir("../dist-assets/binaries");
    declare_libs_dir("../build/lib");

    println!("cargo::rustc-link-search={out_dir}");

    if target_os == "android" {
        // NOTE: Go programs does not support being statically linked on android
        // so we need to dynamically link to libwg
        println!("cargo::rustc-link-lib=wg")
    } else {
        // other platforms can statically link to libwg just fine
        // TODO: consider doing dynamic linking everywhere, to keep things simpler
        println!("cargo::rustc-link-lib=static=wg");
    }

    // TODO: check that these are correct
    println!("cargo::rerun-if-changed=libwg");
}

/// Tell linker to check `base`/$TARGET for shared libraries.
fn declare_libs_dir(base: &str) {
    let target_triplet = env::var("TARGET").expect("TARGET is not set");
    let lib_dir = manifest_dir().join(base).join(target_triplet);
    println!("cargo::rerun-if-changed={}", lib_dir.display());
    println!("cargo::rustc-link-search={}", lib_dir.display());
}

/// Get the directory containing `Cargo.toml`
fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}
