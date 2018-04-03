#[cfg(windows)]
mod win {
    use std::env;
    use std::path::{Path, PathBuf};

    static WFP_SOLUTIONS: &'static [&'static str] = &["libcommon", "libwfp", "wfpctl"];
    static WFP_BUILD_DIR: &'static str = "wfp\\bin";
    static WFP_SRC_DIR: &'static str = "wfp\\src";

    fn manifest_dir() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR env var not set")
    }

    pub fn get_wfp_build_dir() -> &'static Path {
        Path::new(WFP_BUILD_DIR)
    }

    pub fn get_wfp_src_dir() -> &'static Path {
        Path::new(WFP_SRC_DIR)
    }

    pub fn get_lib_dir() -> PathBuf {
        let target = env::var("TARGET").expect("TARGET env var not set");

        let target_dir = match target.as_str() {
            "i686-pc-windows-msvc" => format!("Win32-{}", get_build_mode()),
            "x86_64-pc-windows-msvc" => format!("x64-{}", get_build_mode()),
            _ => panic!("uncrecognized target: {}", target),
        };

        let mut lib_dir = manifest_dir();
        lib_dir.push(get_wfp_build_dir());
        lib_dir.push(&target_dir);

        lib_dir
    }

    fn get_target_platform() -> &'static str {
        let target = env::var("TARGET").expect("TARGET env var not set");
        match target.as_str() {
            "i686-pc-windows-msvc" => "x86",
            "x86_64-pc-windows-msvc" => "x64",
            _ => panic!("uncrecognized target: {}", target),
        }
    }

    pub fn get_build_mode() -> &'static str {
        let profile = env::var("PROFILE").expect("PROFILE env var not set");
        if profile == "release" {
            "Release"
        } else {
            "Debug"
        }
    }

    pub fn build_wfpctl() {
        use std::process::Command;
        use std::fs;

        let mut wfp_manifest_path = manifest_dir();
        wfp_manifest_path.push(get_wfp_src_dir());
        wfp_manifest_path.push("wfp.sln");

        let mut cmd = Command::new("msbuild");
        cmd.arg(wfp_manifest_path)
            .arg(format!{"/t:{}", WFP_SOLUTIONS.join(";")})
            .arg(format!("/p:Configuration={}", get_build_mode()))
            .arg(format!("/p:Platform={}", get_target_platform()));
        println!("running build command: {:?}", cmd);
        let result = cmd.spawn()
            .expect("failed to start building wfpctl library, is msbuild in %PATH% ?")
            .wait()
            .map(|w| w.success())
            .unwrap_or(false);

        if !result {
            panic!("failed to build wfpctl library");
        }

        let mut lib_output_path = env::var("OUT_DIR")
            .map(PathBuf::from)
            .expect("OUT_DIR env var not set");
        lib_output_path.push("wfpctl.dll");
        let mut lib_dir = get_lib_dir();
        lib_dir.push("wfpctl.dll");

        fs::copy(lib_dir, lib_output_path).expect("failed to copy wfpctl.dll to output directory");
    }
}

#[cfg(windows)]
fn main() {
    use win::*;
    build_wfpctl();
    let wfpctl_build_dir = get_lib_dir();
    let wfpctl_build_dir = wfpctl_build_dir
        .to_str()
        .expect("failed construct a wfpctl build dir path");

    println!("cargo:rustc-link-search={}", wfpctl_build_dir);
    println!("cargo:rustc-link-lib=wfpctl");
}

#[cfg(not(windows))]
fn main() {}
