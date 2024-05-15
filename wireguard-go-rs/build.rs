use core::{panic, str};
use std::{env, path::PathBuf};
// use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Missing OUT_DIR");
    eprintln!("OUT_DIR: {out_dir}");
    // Add DAITA as a conditional configuration
    println!("cargo::rustc-check-cfg=cfg(daita)");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Missing 'CARGO_CFG_TARGET_OS");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("./build-wireguard-go.sh");

    match target_os.as_str() {
        "linux" => {
            // Enable DAITA
            println!(r#"cargo::rustc-cfg=daita"#);
            // Build libmaybenot ..
            build_maybenot_lib(&out_dir);
            // .. before building wireguard-go with DAITA support
            cmd.arg("--daita");
        }
        "android" => {
            cmd.arg("--android");
        }
        "macos" => {}
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

    if target_os == "android" {
        // NOTE: Go programs does not support being statically linked on android
        // so we need to dynamically link to libwg
        println!("cargo::rustc-link-lib=wg");
        declare_libs_dir("../build/lib");
    } else {
        // other platforms can statically link to libwg just fine
        // TODO: consider doing dynamic linking everywhere, to keep things simpler
        println!("cargo::rustc-link-lib=static=wg");
        println!("cargo::rustc-link-search={out_dir}");
    }

    println!("cargo::rerun-if-changed=libwg");
}

/// Build maybenot as a library by invoking the Makefile in `maybenot-ffi`.
///
/// `out_dir`: The folder where the `libmaybenot.a` will be placed.
fn build_maybenot_lib(out_dir: &str) {
    let mut maybenot_build_cmd = std::process::Command::new("make");
    maybenot_build_cmd
        .args([
            "--directory",
            "libwg/wireguard-go/maybenot/crates/maybenot-ffi",
        ])
        .arg(format!("DESTINATION={out_dir}"));

    let output = maybenot_build_cmd
        .output()
        .expect("Something failed while building maybenot-ffi");

    if !output.status.success() {
        let stdout = str::from_utf8(&output.stdout).unwrap();
        let stderr = str::from_utf8(&output.stderr).unwrap();
        eprintln!("Invoking make on maybenot-ffi/Makefile failed.");
        eprintln!("stdout:\n{stdout}");
        eprintln!("stderr:\n{stderr}");
        panic!();
    }

    // Link against the static `maybenot` library.
    if cfg!(not(target_os = "android")) {
        println!("cargo::rustc-link-lib=static=maybenot");
    }
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
