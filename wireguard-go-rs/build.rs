use std::{
    borrow::BorrowMut,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
    str,
};

use anyhow::{Context, anyhow, bail};

fn main() -> anyhow::Result<()> {
    // Mark "daita" as a conditional configuration flag
    println!("cargo::rustc-check-cfg=cfg(daita)");

    // Enable the DAITA (rust) feature flag
    println!(r#"cargo::rustc-cfg=daita"#);

    // Rerun build-script if libwg (or wireguard-go) is changed
    println!("cargo::rerun-if-changed=libwg");

    let out_dir = env::var("OUT_DIR").context("Missing OUT_DIR")?;
    if target_os()? == Os::Windows && host_os() == Os::Windows {
        build_windows_dynamic_lib(&out_dir)?;
    }
    Ok(())
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Os {
    Windows,
    Macos,
    Linux,
    Android,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Arch {
    Amd64,
    Arm64,
}

const fn host_os() -> Os {
    // this ugliness is a limitation of rust, where we can't directly
    // access the target triple of the build script.
    const HOST: Os = cfg_select! {
        target_os = "windows" => { Os::Windows }
        target_os = "linux"   => { Os::Linux }
        target_os = "macos"   => { Os::Macos }
    };
    HOST
}

fn target_os() -> anyhow::Result<Os> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").context("Missing 'CARGO_CFG_TARGET_OS")?;
    match target_os.as_str() {
        "windows" => Ok(Os::Windows),
        "linux" => Ok(Os::Linux),
        "macos" => Ok(Os::Macos),
        "android" => Ok(Os::Android),
        _ => bail!("Unsupported target os: {target_os}"),
    }
}

const fn host_arch() -> Arch {
    const ARCH: Arch = cfg_select! {
        target_arch = "x86_64"  => { Arch::Amd64 }
        target_arch = "aarch64" => { Arch::Arm64 }
    };
    ARCH
}

fn target_arch() -> anyhow::Result<Arch> {
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").context("Missing 'CARGO_CFG_TARGET_ARCH")?;
    match target_arch.as_str() {
        "x86_64" => Ok(Arch::Amd64),
        "aarch64" => Ok(Arch::Arm64),
        _ => bail!("Unsupported architecture: {target_arch}"),
    }
}

/// Compile libwg and maybenot and place them in the target dir relative to `OUT_DIR`.
///
/// The host has to run Windows.
fn build_windows_dynamic_lib(out_dir: &str) -> anyhow::Result<()> {
    let target_dir = Path::new(out_dir)
        .ancestors()
        .nth(3)
        .context("Failed to find target dir")?;
    build_shared_maybenot_lib(target_dir).context("Failed to build maybenot")?;

    let dll_path = target_dir.join("libwg.dll");
    let mut go_build = Command::new("go");
    go_build
        .env("CGO_ENABLED", "1")
        .current_dir("./libwg")
        .args(["build", "-v"])
        .arg("-o")
        .arg(&dll_path)
        .args(["--tags", "daita"])
        // Build DLL
        .args(["-buildmode", "c-shared"])
        // Needed for linking against maybenot-ffi
        .env("CGO_LDFLAGS", format!("-L{}", target_dir.to_str().unwrap()))
        .env("GOOS", "windows");

    let target_arch = target_arch()?;
    // We explicitly use zig for compiling libwg. Any MinGW-compatible toolchain should work.
    match target_arch {
        Arch::Amd64 => {
            go_build.env("CC", "zig cc -target x86_64-windows");
            go_build.env("GOARCH", "amd64");
        }
        Arch::Arm64 => {
            go_build.env("CC", "zig cc -target aarch64-windows");
            go_build.env("GOARCH", "arm64");
        }
    }

    generate_windows_lib(target_arch, target_dir)?;

    exec(go_build)?;

    println!("cargo::rustc-link-search={}", target_dir.to_str().unwrap());
    println!("cargo::rustc-link-lib=dylib=libwg");

    Ok(())
}

// Build dynamically library for maybenot
fn build_shared_maybenot_lib(out_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let target_triple = env::var("TARGET").context("Missing 'TARGET'")?;
    let profile_category = env::var("PROFILE").context("Missing 'PROFILE'")?;
    let profile = match profile_category.as_str() {
        "release" => "release",
        _ => "dev",
    };

    let mut build_command = Command::new("cargo");

    std::fs::create_dir_all("../build")?;

    let mut tmp_build_dir = Path::new("../build").canonicalize()?;

    // Strip \\?\ prefix. Note that doing this directly on Path/PathBuf fails
    let path_str = tmp_build_dir.to_str().unwrap();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        tmp_build_dir = PathBuf::from(stripped);
    }

    tmp_build_dir = tmp_build_dir.join("target");

    build_command
        .current_dir("./libwg/wireguard-go/maybenot-ffi")
        .env("RUSTFLAGS", "-C metadata=maybenot-ffi -Ctarget-feature=+crt-static")
        // Set temporary target dir to prevent deadlock, since we are invoking cargo from within
        // another cargo process.
        .env("CARGO_TARGET_DIR", &tmp_build_dir)
        .arg("rustc")
        // Build a shared library to consume from another language (go)
        .arg("--crate-type=cdylib")
        // Always respect lockfiles
        .args(["--locked"])
        .args(["--profile", profile])
        .args(["--target", &target_triple]);

    exec(build_command)?;

    let artifacts_dir = tmp_build_dir.join(target_triple).join(profile_category);

    // Copy library to desired target dir
    for (src_filename, dest_filename) in [
        ("maybenot_ffi.dll", "maybenot_ffi.dll"),
        ("maybenot_ffi.dll.lib", "maybenot.lib"),
    ] {
        let src = artifacts_dir.join(src_filename);
        let dest = out_dir.as_ref().join(dest_filename);
        fs::copy(&src, &dest).with_context(|| format!("Failed to copy {src_filename}",))?;
    }

    Ok(())
}

/// Generate a library for the exported functions. Required for load-time linking.
/// This requires `msbuild.exe` in the path.
fn generate_windows_lib(arch: Arch, out_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let exports_def_path = out_dir.as_ref().join("exports.def");
    generate_exports_def(&exports_def_path).context("Failed to generate exports.def")?;
    generate_lib_from_exports_def(arch, &exports_def_path)
        .context("Failed to generate lib from exports.def")
}

/// Find the correct `lib.exe` for this host and the target arch.
fn find_lib_exe() -> anyhow::Result<PathBuf> {
    let msbuild_exe = find_msbuild_exe()?;

    // Find lib.exe relative to msbuild.exe, in ../../../../ relative to msbuild
    let search_path = msbuild_exe
        .ancestors()
        .nth(4)
        .context("Unexpected msbuild.exe path")?;

    // This pattern can be found by browsing `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\<MSVC-version>\bin\<host>`
    let lib_exe_host = match host_arch() {
        Arch::Amd64 => "Hostx64",
        Arch::Arm64 => "Hostarm64",
    };

    // This pattern can be found by browsing `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\<MSVC-version>\bin\<host>\<arch>`
    let lib_exe_target = match target_arch()? {
        Arch::Amd64 => "x64",
        Arch::Arm64 => "arm64",
    };

    let lib_exe_pattern = format!("{lib_exe_host}/{lib_exe_target}/lib.exe",);
    let path_is_lib_exe = |file: &Path| file.ends_with(&lib_exe_pattern);

    find_file(search_path, &path_is_lib_exe)?.context("No lib.exe relative to msbuild.exe")
}

/// Recursively search for file until 'condition' returns true
fn find_file(
    dir: impl AsRef<Path>,
    condition: &impl Fn(&Path) -> bool,
) -> anyhow::Result<Option<PathBuf>> {
    for path in std::fs::read_dir(dir).context("Failed to read dir")? {
        let entry = path.context("Failed to read dir entry")?;
        let path = entry.path();
        if path.is_dir()
            && let Some(result) = find_file(&path, condition)?
        {
            return Ok(Some(result));
        }

        if condition(&path) {
            return Ok(Some(path.to_owned()));
        }
    }
    Ok(None)
}

/// Find msbuild.exe in PATH
fn find_msbuild_exe() -> anyhow::Result<PathBuf> {
    let path = std::env::var_os("PATH").context("Missing PATH var")?;
    std::env::split_paths(&path)
        .find(|path| path.join("msbuild.exe").exists())
        .context("msbuild.exe not found in PATH")
}

/// Generate lib from export
fn generate_lib_from_exports_def(arch: Arch, exports_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let lib_path = exports_path
        .as_ref()
        .parent()
        .context("Missing parent")?
        .join("libwg.lib");
    let path = exports_path.as_ref().to_str().context("Non-UTF8 path")?;

    let lib_exe = find_lib_exe()?;

    let mut lib_exe = Command::new(lib_exe);
    lib_exe.args([
        format!("/def:{path}"),
        format!("/out:{}", lib_path.to_str().context("Non-UTF8 lib path")?),
    ]);

    match arch {
        Arch::Amd64 => {
            lib_exe.arg("/machine:X64");
        }
        Arch::Arm64 => {
            lib_exe.arg("/machine:ARM64");
        }
    }

    exec(lib_exe)?;

    Ok(())
}

/// Generate exports.def from wireguard-go source
fn generate_exports_def(exports_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file = File::create(exports_path).context("Failed to create file")?;
    let mut file = BufWriter::new(file);

    writeln!(file, "LIBRARY libwg").context("Write LIBRARY statement")?;
    writeln!(file, "EXPORTS").context("Write EXPORTS statement")?;

    for path in &[
        "./libwg/libwg.go",
        "./libwg/libwg_windows.go",
        "./libwg/libwg_daita.go",
    ] {
        for export in gather_exports(path).context("Failed to find exports")? {
            writeln!(file, "\t{export}").context("Failed to output exported function")?;
        }
    }

    Ok(())
}

/// Return functions exported from .go file
fn gather_exports(go_src_path: impl AsRef<Path>) -> anyhow::Result<Vec<String>> {
    let go_src_path = go_src_path.as_ref();
    let mut exports = vec![];
    let file = File::open(go_src_path)
        .with_context(|| format!("Failed to open go source: {}", go_src_path.display()))?;

    for line in BufReader::new(file).lines() {
        let line = line.context("Failed to read line in go src")?;
        let mut words = line.split_whitespace();

        // Is this an export?
        let Some("//export") = words.next() else {
            continue;
        };

        let exported_func = words
            .next()
            .with_context(|| format!("Invalid export on line: {line}"))?;
        exports.push(exported_func.to_owned());
    }

    Ok(exports)
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
