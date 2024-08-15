use core::{panic, str};
use std::{env, path::PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Missing OUT_DIR");

    // Add DAITA as a conditional configuration
    println!("cargo::rustc-check-cfg=cfg(daita)");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Missing 'CARGO_CFG_TARGET_OS");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("./build-wireguard-go.sh");

    match target_os.as_str() {
        "linux" | "macos" => {
            // Enable DAITA
            println!(r#"cargo::rustc-cfg=daita"#);
            // Tell the build script to build wireguard-go with DAITA support
            cmd.arg("--daita");
        }
        "android" => {
            cmd.arg("--android");
        }
        // building wireguard-go-rs for windows is not implemented
        _ => return,
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

    if target_os.as_str() != "android" {
        println!("cargo::rustc-link-lib=static=wg");
    } else {
        // NOTE: Link dynamically to libwg on Android, as go cannot produce archives
        println!("cargo::rustc-link-lib=dylib=wg");
    }
    declare_libs_dir("../build/lib");

    println!("cargo::rerun-if-changed=libwg");

    // Add `OUT_DIR` to the library search path to facilitate linking of libwg for debug artifacts,
    // such as test binaries.
    if cfg!(debug_assertions) {
        println!("cargo::rustc-link-search={out_dir}");
    }
}

/// Tell linker to check `base`/$TARGET for shared libraries.
fn declare_libs_dir(base: &str) {
    let target_triplet = env::var("TARGET").expect("TARGET is not set");
    let lib_dir = manifest_dir().join(base).join(target_triplet);
    println!("cargo::rustc-link-search={}", lib_dir.display());
}

/// Get the directory containing `Cargo.toml`
fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}
