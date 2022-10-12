#[cfg(windows)]
mod win {
    use std::{env, path::PathBuf};

    pub fn declare_winnet_dependency() {
        const WINNET_BUILD_DIR: &'static str = "..\\windows\\winnet\\bin";
        const WINNET_DIR_VAR: &str = "WINNET_LIB_DIR";
        declare_library(WINNET_DIR_VAR, WINNET_BUILD_DIR, "winnet");
    }

    fn default_windows_build_artifact_dir(build_dir: &str) -> PathBuf {
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

    fn declare_library(env_var: &str, default_dir: &str, lib_name: &str) {
        println!("cargo:rerun-if-env-changed={}", env_var);
        let lib_dir = env::var_os(env_var)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_windows_build_artifact_dir(default_dir));
        println!("cargo:rustc-link-search={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib={}", lib_name);
    }

    fn manifest_dir() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR env var not set")
    }
}

fn main() {
    #[cfg(windows)]
    win::declare_winnet_dependency();
}
