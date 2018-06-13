#[cfg(windows)]
mod win {
    use std::env;
    use std::path::PathBuf;

    pub static WINFW_BUILD_DIR: &'static str = "..\\windows\\winfw\\bin";
    pub static WINDNS_BUILD_DIR: &'static str = "..\\windows\\windns\\bin";
    pub static WINROUTE_BUILD_DIR: &'static str = "..\\windows\\winroute\\bin";

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

    fn manifest_dir() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR env var not set")
    }

    fn get_build_mode() -> &'static str {
        let profile = env::var("PROFILE").expect("PROFILE env var not set");
        if profile == "release" {
            "Release"
        } else {
            "Debug"
        }
    }
}

#[cfg(windows)]
fn main() {
    use std::env;
    use std::path::PathBuf;
    use win::*;

    const WINFW_LIB_DIR_VAR: &str = "WINFW_LIB_DIR";
    println!("cargo:rerun-if-env-changed={}", WINFW_LIB_DIR_VAR);
    let winfw_dir = env::var_os(WINFW_LIB_DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| default_windows_build_artifact_dir(WINFW_BUILD_DIR));

    println!(
        "cargo:rustc-link-search={}",
        winfw_dir
            .to_str()
            .expect("failed to construct path for winfw include directory")
    );
    println!("cargo:rustc-link-lib=dylib=winfw");

    let windns_dir = env::var_os("WINDNS_INCLUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_windows_build_artifact_dir(WINDNS_BUILD_DIR));
    println!(
        "cargo:rustc-link-search={}",
        windns_dir
            .to_str()
            .expect("failed to construct path for windns include directory")
    );
    println!("cargo:rustc-link-lib=dylib=windns");

    let winroute_dir = env::var_os("WINROUTE_INCLUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_windows_build_artifact_dir(WINROUTE_BUILD_DIR));
    println!(
        "cargo:rustc-link-search={}",
        winroute_dir
            .to_str()
            .expect("failed to construct path for winroute include directory")
    );
    println!("cargo:rustc-link-lib=dylib=winroute");
}

#[cfg(not(windows))]
fn main() {}
