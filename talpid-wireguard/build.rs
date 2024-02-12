use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    declare_libs_dir("../dist-assets/binaries");
    declare_libs_dir("../build/lib");

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" => "=static",
        // We would like to avoid panicking on windows even if we can not link correctly
        // because we would like to be able to run check and clippy.
        // This does not allow for correct linking or buijding.
        #[cfg(not(windows))]
        "windows" => "",
        #[cfg(windows)]
        "windows" => "dylib",
        _ => panic!("Unsupported platform: {target_os}"),
    };

    println!("cargo:rustc-link-lib{link_type}=wg");
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") || cfg!(target_os = "android") {
        // Build wireguard-go if it does not already exist
        let lib_path = std::path::Path::new(&"../build/lib").join(target_triplet());
        if !lib_path.exists() {
            build_wireguard_go();
        }
        println!("cargo:rustc-cfg=wireguard_go");
    }
}

fn declare_libs_dir(base: &str) {
    let lib_dir = manifest_dir().join(base).join(target_triplet());
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rustc-link-search={}", lib_dir.display());
}

fn target_triplet() -> String {
    env::var("TARGET").expect("TARGET is not set")
}

fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}

fn build_wireguard_go() -> std::process::Output {
    std::process::Command::new("../wireguard/build-wireguard-go.sh")
        .args(&[target_triplet()])
        .output()
        .unwrap()
}
