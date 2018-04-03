#[cfg(target_os = "windows")]
mod win {
    use std::env;

    static WFP_SOLUTIONS: &'static [&'static str] = &["libcommon", "libwfp", "wfpctl"];

    fn manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap().to_owned()
    }

    static WFP_BUILD_DIR: &'static str = "\\wfp\\bin";
    static WFP_SRC_DIR: &'static str = "\\wfp\\src";

    pub fn get_lib_dir() -> String {
        let target = env::var("TARGET").expect("TARGET env var not set");
        let mut build_output = manifest_dir();
        build_output += WFP_BUILD_DIR;

        let target_dir = match target.as_str() {
            "i686-pc-windows-msvc" => format!("\\Win32-{}", get_build_mode()),
            "x86_64-pc-windows-msvc" => format!("\\x64-{}", get_build_mode()),
            _ => panic!("uncrecognized target: {}", target),
        };

        let mut lib_dir = manifest_dir();
        lib_dir += WFP_BUILD_DIR;
        lib_dir += &target_dir;

        lib_dir
    }

    fn get_target_platform() -> String {
        let target = env::var("TARGET").expect("TARGET env var not set");
        match target.as_str() {
            "i686-pc-windows-msvc" => "x86".to_owned(),
            "x86_64-pc-windows-msvc" => "x64".to_owned(),
            _ => panic!("uncrecognized target: {}", target),
        }
    }

    pub fn get_build_mode() -> String {
        let profile = env::var("PROFILE").expect("PROFILE env var not set");
        if profile == "release" {
            "Release".to_owned()
        } else {
            "Debug".to_owned()
        }
    }

    pub fn build_wfpctl() {
        use std::process::Command;
        use std::fs::copy;

        let mut wfp_manifest_path = manifest_dir();
        wfp_manifest_path += WFP_SRC_DIR;
        wfp_manifest_path += "\\wfp.sln";

        let result = Command::new("msbuild")
            .arg(wfp_manifest_path)
            .arg(format!{"/t:{}", WFP_SOLUTIONS.join(";")})
            .arg(format!("/p:Configuration={}", get_build_mode()))
            .arg(format!("/p:Platform={}", get_target_platform()))
            .spawn()
            .expect("failed to start building wfpctl library, is msbuild in %PATH% ?")
            .wait()
            .map(|w| w.success())
            .unwrap_or(false);

        if !result {
            panic!("failed to build wfpctl library");
        }

        let mut lib_output_path = env::var("OUT_DIR").expect("OUT_DIR env var not set");
        lib_output_path.push_str("\\wfpctl.dll");
        let mut lib_dir = get_lib_dir();
        lib_dir.push_str("\\wfpctl.dll");

        copy(lib_dir, lib_output_path).expect("failed to copy wfpctl.dll to output directory");
    }
}

fn main() {
    if cfg!(target_os = "windows") {
        use win::*;
        build_wfpctl();
        println!("cargo:rustc-link-search={}", get_lib_dir());
        println!("cargo:rustc-link-lib=wfpctl");
    }
}
