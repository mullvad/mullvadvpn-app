use std::{
    borrow::BorrowMut,
    env,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use anyhow::{anyhow, bail, Context};

fn main() -> anyhow::Result<()> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").context("Missing 'CARGO_CFG_TARGET_OS")?;

    // Mark "daita" as a conditional configuration flag
    println!("cargo::rustc-check-cfg=cfg(daita)");

    // Rerun build-script if libwg (or wireguard-go) is changed
    println!("cargo::rerun-if-changed=libwg");

    match target_os.as_str() {
        "linux" => build_static_lib(Os::Linux, true)?,
        "macos" => build_static_lib(Os::MacOs, true)?,
        "android" => build_android_dynamic_lib(true)?,
        // building wireguard-go-rs for windows is not implemented
        _ => {}
    }

    Ok(())
}

#[derive(PartialEq, Eq)]
enum Os {
    MacOs,
    Linux,
}

#[derive(PartialEq, Eq)]
enum Arch {
    Amd64,
    Arm64,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum AndroidTarget {
    Aarch64, // "aarch64"
    X86,     // "x86_64"
    Armv7,   // "armv7"
    I686,    // "i686"
}

impl AndroidTarget {
    fn from_str(input: &str) -> anyhow::Result<Self> {
        use AndroidTarget::*;
        match input {
            "aarch64-linux-android" => Ok(Aarch64),
            "x86_64-linux-android" => Ok(X86),
            "armv7-linux-androideabi" => Ok(Armv7),
            "i686-linux-android" => Ok(I686),
            _ => bail!("{input} is not a supported android target!"),
        }
    }
}

fn host_os() -> anyhow::Result<Os> {
    // this ugliness is a limitation of rust, where we can't directly
    // access the target triple of the build script.
    if cfg!(target_os = "linux") {
        Ok(Os::Linux)
    } else if cfg!(target_os = "macos") {
        Ok(Os::MacOs)
    } else {
        bail!("Unsupported host OS")
    }
}

fn host_arch() -> anyhow::Result<Arch> {
    if cfg!(target_arch = "x86_64") {
        Ok(Arch::Amd64)
    } else if cfg!(target_arch = "aarch64") {
        Ok(Arch::Arm64)
    } else {
        bail!("Unsupported host architecture")
    }
}

/// Compile libwg as a static library and place it in `OUT_DIR`.
fn build_static_lib(target_os: Os, daita: bool) -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").context("Missing OUT_DIR")?;
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").context("Missing 'CARGO_CFG_TARGET_ARCH")?;

    let target_arch = match target_arch.as_str() {
        "x86_64" => Arch::Amd64,
        "aarch64" => Arch::Arm64,
        _ => bail!("Unsupported architecture: {target_arch}"),
    };

    let out_file = format!("{out_dir}/libwg.a");
    let mut go_build = Command::new("go");
    go_build
        .args(["build", "-v", "-o", &out_file])
        .args(["-buildmode", "c-archive"])
        .args(if daita { &["--tags", "daita"][..] } else { &[] })
        .env("CGO_ENABLED", "1")
        .current_dir("./libwg");

    // are we cross compiling?
    let cross_compiling = host_os()? != target_os || host_arch()? != target_arch;

    match target_arch {
        Arch::Amd64 => go_build.env("GOARCH", "amd64"),
        Arch::Arm64 => go_build.env("GOARCH", "arm64"),
    };

    match target_os {
        Os::Linux => {
            go_build.env("GOOS", "linux");

            if cross_compiling {
                match target_arch {
                    Arch::Arm64 => go_build.env("CC", "aarch64-linux-gnu-gcc"),
                    Arch::Amd64 => bail!("cross-compiling to linux x86_64 is not implemented"),
                };
            }
        }
        Os::MacOs => {
            go_build.env("GOOS", "darwin");

            if cross_compiling {
                let sdkroot = env::var("SDKROOT").context("Missing 'SDKROOT'")?;

                let c_arch = match target_arch {
                    Arch::Amd64 => "x86_64",
                    Arch::Arm64 => "arm64",
                };

                let xcrun_output =
                    exec(Command::new("xcrun").args(["-sdk", &sdkroot, "--find", "clang"]))?;
                go_build.env("CC", xcrun_output);

                let cflags = format!("-isysroot {sdkroot} -arch {c_arch} -I{sdkroot}/usr/include");
                go_build.env("CFLAGS", cflags);
                go_build.env("CGO_CFLAGS", format!("-isysroot {sdkroot} -arch {c_arch}"));
                go_build.env("CGO_LDFLAGS", format!("-isysroot {sdkroot} -arch {c_arch}"));
                go_build.env("LD_LIBRARY_PATH", format!("{sdkroot}/usr/lib"));
            }
        }
    }

    exec(go_build)?;

    // make sure to link to the resulting binary
    println!("cargo::rustc-link-search={out_dir}");
    println!("cargo::rustc-link-lib=static=wg");

    // if daita is enabled, also enable the corresponding rust feature flag
    if daita {
        println!(r#"cargo::rustc-cfg=daita"#);
    }

    Ok(())
}

/// Compile libwg as a dynamic library for android and place it in [`android_output_path`].
// NOTE: We use dynamic linking as Go cannot produce static binaries specifically for Android.
fn build_android_dynamic_lib(daita: bool) -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").context("Missing OUT_DIR")?;
    let target_triple = env::var("TARGET").context("Missing 'TARGET'")?;
    let target = AndroidTarget::from_str(&target_triple)?;

    // TODO: Since `libwg.so` is always copied to `android_output_path`, this rerun-directive will
    // always trigger cargo to rebuild this crate. Some mechanism to detected changes to `android_output_path`
    // is needed, because some external program may clean it at any time (e.g. gradle).
    println!(
        "cargo::rerun-if-changed={}",
        android_output_path(target)?.display()
    );

    // Before calling `canonicalize`, the directory we're referring to actually has to exist.
    std::fs::create_dir_all("../build")?;
    let tmp_build_dir = Path::new("../build").canonicalize()?;
    let go_path = tmp_build_dir.join("android-go-path");
    // Invoke the Makefile in wireguard-go-rs/libwg
    let mut build_command = Command::new("make");
    build_command
        .args(["-C", "./libwg"])
        .args(["-f", "Android.mk"]);
    // Set up the correct Android toolchain for building libwg
    build_command
        .env("ANDROID_C_COMPILER", android_c_compiler(target)?)
        .env("ANDROID_ABI", android_abi(target))
        .env("ANDROID_ARCH_NAME", android_arch_name(target))
        .env("GOPATH", &go_path)
        // Note: -w -s results in a stripped binary
        .env("LDFLAGS", format!("-L{out_dir} -w -s"))
        // Note: the build container overrides CARGO_TARGET_DIR, which will cause problems
        // since we will spawn another cargo process as part of building maybenot (which we
        // link into libwg). A work around is to simply override the overridden value, and we
        // do this by pointing to a target folder in our temporary build folder.
        .env("CARGO_TARGET_DIR", tmp_build_dir.join("target"));

    exec(build_command)?;

    // Move the resulting binary to the path where the Android project expects it to be
    let binary = Path::new(&out_dir).join("libwg.so");
    let android_output_path = android_output_path(target)?;
    let output = android_output_path.join("libwg.so");
    android_move_binary(&binary, &output)?;

    // Tell linker to check android_output_path for the dynamic library.
    println!("cargo::rustc-link-search={}", android_output_path.display());
    println!("cargo::rustc-link-lib=dylib=wg");

    // If daita is enabled, also enable the corresponding rust feature flag
    if daita {
        println!(r#"cargo::rustc-cfg=daita"#);
    }

    Ok(())
}

/// Copy `binary` to `output`.
///
/// Note: This function will create the parent directory/directories to `output` if necessary.
fn android_move_binary(binary: &Path, output: &Path) -> anyhow::Result<()> {
    let parent_of_output = output.parent().context(format!(
        "Could not find parent directory of {}",
        output.display()
    ))?;
    std::fs::create_dir_all(parent_of_output)?;

    let mut move_command = Command::new("mv");
    move_command
        .arg(binary.to_str().unwrap())
        .arg(output.to_str().unwrap());

    exec(&mut move_command)?;

    Ok(())
}

fn android_c_compiler(target: AndroidTarget) -> anyhow::Result<PathBuf> {
    let toolchain = env::var("NDK_TOOLCHAIN_DIR").context("Missing 'NDK_TOOLCHAIN_DIR")?;
    let ccompiler = match target {
        AndroidTarget::Aarch64 => "aarch64-linux-android26-clang",
        AndroidTarget::X86 => "x86_64-linux-android26-clang",
        AndroidTarget::Armv7 => "armv7a-linux-androideabi26-clang",
        AndroidTarget::I686 => "i686-linux-android26-clang",
    };
    let compiler = Path::new(&toolchain).join(ccompiler);
    Ok(compiler)
}

fn android_abi(target: AndroidTarget) -> String {
    match target {
        AndroidTarget::Aarch64 => "arm64-v8a",
        AndroidTarget::X86 => "x86_64",
        AndroidTarget::Armv7 => "armeabi-v7a",
        AndroidTarget::I686 => "x86",
    }
    .to_string()
}

fn android_arch_name(target: AndroidTarget) -> String {
    match target {
        AndroidTarget::Aarch64 => "arm64",
        AndroidTarget::X86 => "x86_64",
        AndroidTarget::Armv7 => "arm",
        AndroidTarget::I686 => "x86",
    }
    .to_string()
}

// Returns the path where the Android project expects Rust binaries to be
fn android_output_path(target: AndroidTarget) -> anyhow::Result<PathBuf> {
    let relative_output_path = Path::new("../android/app/build/rustJniLibs/android").join(android_abi(target));
    std::fs::create_dir_all(relative_output_path.clone())?;
    let output_path = relative_output_path.canonicalize()?;
    Ok(output_path)
}

/// Execute a command, assert that it succeeds, and return stdout as a string.
fn exec(mut command: impl BorrowMut<Command>) -> anyhow::Result<String> {
    let command = command.borrow_mut();

    let output = command
        .output()
        .with_context(|| anyhow!("Failed to execute command: {command:?}"))?;

    let stdout = str::from_utf8(&output.stdout).unwrap_or("Invalid UTF-8");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr).unwrap_or("Invalid UTF-8");

        eprintln!("Error from {command:?}");
        eprintln!();
        eprintln!("stdout:");
        eprintln!();
        eprintln!("{stdout}");
        eprintln!();
        eprintln!("-------");
        eprintln!("stderr:");
        eprintln!();
        eprintln!("{stderr}");
        eprintln!();
        eprintln!("-------");

        return Err(anyhow!("Failed to execute command: {command:?}")).with_context(|| {
            anyhow!(
                "Command exited with a non-zero exit code: {}",
                output.status
            )
        });
    }

    Ok(stdout.to_string())
}
