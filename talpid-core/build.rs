use std::{env, path::PathBuf};

#[cfg(windows)]
mod win {
    use super::manifest_dir;
    use std::{env, path::PathBuf};

    pub static WINFW_BUILD_DIR: &'static str = "..\\windows\\winfw\\bin";

    pub fn default_windows_build_artifact_dir(build_dir: &str) -> PathBuf {
        manifest_dir().join(build_dir).join(&target_platform_dir())
    }

    fn target_platform_dir() -> PathBuf {
        let target = env::var("TARGET").expect("TARGET env var not set");

        let target_dir = match target.as_str() {
            "i686-pc-windows-msvc" => format!("Win32-{}", get_build_mode()),
            "x86_64-pc-windows-msvc" => format!("x64-{}", get_build_mode()),
            _ => panic!("uncrecognized target: {}", target),
        };
        target_dir.into()
    }

    fn get_build_mode() -> &'static str {
        let profile = env::var("PROFILE").expect("PROFILE env var not set");
        if profile == "release" {
            "Release"
        } else {
            "Debug"
        }
    }

    pub fn declare_library(env_var: &str, default_dir: &str, lib_name: &str) {
        println!("cargo:rerun-if-env-changed={}", env_var);
        let lib_dir = env::var_os(env_var)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_windows_build_artifact_dir(default_dir));
        println!("cargo:rustc-link-search={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib={}", lib_name);
    }
}

#[cfg(windows)]
fn main() {
    generate_grpc_code();

    use crate::win::*;

    const WINFW_DIR_VAR: &str = "WINFW_LIB_DIR";
    declare_library(WINFW_DIR_VAR, WINFW_BUILD_DIR, "winfw");
    let lib_dir = manifest_dir().join("../build/lib/x86_64-pc-windows-msvc");
    println!("cargo:rustc-link-search={}", &lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=libwg");
}

#[cfg(not(windows))]
fn main() {
    generate_grpc_code();

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    declare_libs_dir("../dist-assets/binaries");
    declare_libs_dir("../build/lib");

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" => "=static",
        // We would like to avoid panicing on windows even if we can not link correctly
        // because we would like to be able to run check and clippy.
        // This does not allow for correct linking or building.
        "windows" => "",
        _ => panic!("Unsupported platform: {}", target_os),
    };

    println!("cargo:rustc-link-lib{}=wg", link_type);
}

fn manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR env var not set")
}

#[cfg(not(windows))]
fn declare_libs_dir(base: &str) {
    let target_triplet = env::var("TARGET").expect("TARGET is not set");
    let lib_dir = manifest_dir().join(base).join(target_triplet);
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rustc-link-search={}", lib_dir.display());
}

fn generate_grpc_code() {
    const PROTO_FILE: &str = "../talpid-openvpn-plugin/proto/openvpn_plugin.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);
}
