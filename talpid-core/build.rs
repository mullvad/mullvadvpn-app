#[cfg(windows)]
mod win {
    use std::{env, path::PathBuf};

    pub static WINFW_BUILD_DIR: &str = "..\\windows\\winfw\\bin";

    pub fn default_windows_build_artifact_dir(build_dir: &str) -> PathBuf {
        manifest_dir().join(build_dir).join(target_platform_dir())
    }

    pub fn target() -> String {
        env::var("TARGET").expect("TARGET env var not set")
    }

    fn target_platform_dir() -> PathBuf {
        let target = target();
        let target_dir = match target.as_str() {
            "i686-pc-windows-msvc" => format!("Win32-{}", get_build_mode()),
            "x86_64-pc-windows-msvc" => format!("x64-{}", get_build_mode()),
            "aarch64-pc-windows-msvc" => format!("ARM64-{}", get_build_mode()),
            _ => panic!("unrecognized target: {target}"),
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
        println!("cargo::rerun-if-env-changed={env_var}");
        let lib_dir = env::var_os(env_var)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_windows_build_artifact_dir(default_dir));
        println!("cargo::rustc-link-search={}", lib_dir.display());
        println!("cargo::rustc-link-lib=dylib={lib_name}");
    }

    pub fn manifest_dir() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR env var not set")
    }
}

#[cfg(windows)]
fn main() {
    generate_grpc_code();

    use crate::win::*;

    const WINFW_DIR_VAR: &str = "WINFW_LIB_DIR";
    declare_library(WINFW_DIR_VAR, WINFW_BUILD_DIR, "winfw");
    let lib_dir = manifest_dir().join("../build/lib").join(target());
    println!("cargo::rustc-link-search={}", &lib_dir.display());
}

#[cfg(not(windows))]
fn main() {
    generate_grpc_code()
}

fn generate_grpc_code() {
    tonic_build::compile_protos("../talpid-openvpn-plugin/proto/openvpn_plugin.proto").unwrap();
}
