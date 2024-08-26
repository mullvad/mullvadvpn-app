use std::{borrow::BorrowMut, env, path::PathBuf, process::Command, str};

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
        "android" => build_android_dynamic_lib()?,
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

/// Compile libwg as a dynamic library for android and place it in `../build/lib/$TARGET`.
// NOTE: We use dynamic linking as Go cannot produce static binaries specifically for Android.
fn build_android_dynamic_lib() -> anyhow::Result<()> {
    let target_triplet = env::var("TARGET").context("TARGET is not set")?;

    exec(Command::new("./libwg/build-android.sh"))?;

    // Tell linker to check `base`/$TARGET for the dynamic library.
    let lib_dir = manifest_dir()?.join("../build/lib").join(target_triplet);
    println!("cargo::rustc-link-search={}", lib_dir.display());
    println!("cargo::rustc-link-lib=dylib=wg");

    // Enable daita
    println!(r#"cargo::rustc-cfg=daita"#);

    Ok(())
}

/// Get the directory containing `Cargo.toml`
fn manifest_dir() -> anyhow::Result<PathBuf> {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .context("CARGO_MANIFEST_DIR env var not set")
}

/// Execute a command, assert that it succeeds, and return stdout as a string.
fn exec(mut command: impl BorrowMut<Command>) -> anyhow::Result<String> {
    let command = command.borrow_mut();

    let output = command
        .output()
        .with_context(|| anyhow!("Failed to execute command: {command:?}"))?;

    let stdout = str::from_utf8(&output.stdout).unwrap_or("Invalid UTF-8");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stdout).unwrap_or("Invalid UTF-8");

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
