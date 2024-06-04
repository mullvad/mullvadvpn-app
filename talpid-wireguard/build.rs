use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    declare_libs_dir("../dist-assets/binaries");

    add_wireguard_go_cfg(&target_os);
}

fn add_wireguard_go_cfg(target_os: &str) {
    println!("cargo:rustc-check-cfg=cfg(wireguard_go)");
    if matches!(target_os, "linux" | "macos" | "android") {
        println!("cargo:rustc-cfg=wireguard_go");
    }
}

fn declare_libs_dir(base: &str) {
    let target_triplet = env::var("TARGET").expect("TARGET is not set");
    let lib_dir = manifest_dir().join(base).join(target_triplet);
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rustc-link-search={}", lib_dir.display());
}

fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}
