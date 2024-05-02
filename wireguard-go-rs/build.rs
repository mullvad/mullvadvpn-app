use core::{panic, str};
use std::env;
// use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Missing OUT_DIR");
    eprintln!("OUT_DIR: {out_dir}");

    let output = std::process::Command::new("bash")
        .arg("./build-wireguard-go.sh")
        .output()
        .expect("build-wireguard-go.sh failed");
    if !output.status.success() {
        let stdout = str::from_utf8(&output.stdout).unwrap();
        let stderr = str::from_utf8(&output.stderr).unwrap();
        eprintln!("build-wireguard-go.sh failed.");
        eprintln!("stdout:\n{stdout}");
        eprintln!("stderr:\n{stderr}");
        panic!();
    }

    // declare_libs_dir("../dist-assets/binaries");
    // declare_libs_dir("../build/lib");

    println!("cargo:rustc-link-search={out_dir}");
    println!("cargo:rustc-link-lib=static=wg");
    // TODO: check that these are correct
    println!("cargo:rerun-if-changed=libwg");
}

// fn declare_libs_dir(base: &str) {
//     let target_triplet = env::var("TARGET").expect("TARGET is not set");
//     let lib_dir = manifest_dir().join(base).join(target_triplet);
//     println!("cargo:rerun-if-changed={}", lib_dir.display());
//     println!("cargo:rustc-link-search={}", lib_dir.display());
// }

// fn manifest_dir() -> PathBuf {
//     env::var("CARGO_MANIFEST_DIR")
//         .map(PathBuf::from)
//         .expect("CARGO_MANIFEST_DIR env var not set")
// }
