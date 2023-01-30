use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    declare_libs_dir("../dist-assets/binaries");
    declare_libs_dir("../build/lib");

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" => "=static",
        // We would like to avoid panicing on windows even if we can not link correctly
        // because we would like to be able to run check and clippy.
        // This does not allow for correct linking or buijding.
        #[cfg(not(windows))]
        "windows" => "",
        #[cfg(windows)]
        "windows" => "dylib",
        _ => panic!("Unsupported platform: {target_os}"),
    };

    println!("cargo:rustc-link-lib{link_type}=wg");
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
