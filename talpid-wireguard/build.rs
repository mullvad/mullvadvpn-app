use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    // Keep track if `binaries/wireguard-nt` changes
    if target_os == "windows" {
        declare_libs_dir("../dist-assets/binaries");
    }
    // Wireguard-Go can be used on all platforms except Windows
    println!("cargo::rustc-check-cfg=cfg(wireguard_go)");
    if matches!(target_os.as_str(), "linux" | "macos" | "android") {
        println!("cargo::rustc-cfg=wireguard_go");
    }
    // Enable DAITA by default on desktop and android
    println!("cargo::rustc-check-cfg=cfg(daita)");
    println!("cargo::rustc-cfg=daita");

    // Ensure that the WireGuard tunnel works before exchanging ephemeral peers.
    // This is useful after updating the WireGuard config, to force a WireGuard handshake. This
    // should reduce the number of PQ timeouts.
    println!("cargo::rustc-check-cfg=cfg(force_wireguard_handshake)");
    if target_os.as_str() == "windows" {
        println!("cargo::rustc-cfg=force_wireguard_handshake");
    }
}

fn declare_libs_dir(base: &str) {
    let target_triplet = env::var("TARGET").expect("TARGET is not set");
    let lib_dir = manifest_dir().join(base).join(target_triplet);
    println!("cargo::rerun-if-changed={}", lib_dir.display());
    println!("cargo::rustc-link-search={}", lib_dir.display());
}

fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}
