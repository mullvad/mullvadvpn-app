#[cfg(windows)]
mod win {
    use std::env;
    use std::path::PathBuf;

    static WFP_BUILD_DIR: &'static str = "..\\wfpctl\\bin";

    pub fn default_wfpctl_output_dir() -> PathBuf {
        let target = env::var("TARGET").expect("TARGET env var not set");

        let target_dir = match target.as_str() {
            "i686-pc-windows-msvc" => format!("Win32-{}", get_build_mode()),
            "x86_64-pc-windows-msvc" => format!("x64-{}", get_build_mode()),
            _ => panic!("uncrecognized target: {}", target),
        };

        let mut lib_dir = manifest_dir();
        lib_dir.push(WFP_BUILD_DIR);
        lib_dir.push(&target_dir);

        lib_dir
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
    use win::*;
    use std::env;
    use std::path::PathBuf;

    let wfpctl_dir = env::var_os("WFPCTL_INCLUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(default_wfpctl_output_dir);

    println!(
        "cargo:rustc-link-search={}",
        wfpctl_dir
            .to_str()
            .expect("failed to construct path for wfpctl include directory")
    );
    println!("cargo:rustc-link-lib=dylib=wfpctl");
}

#[cfg(not(windows))]
fn main() {}
