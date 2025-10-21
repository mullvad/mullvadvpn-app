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
    match target_os()? {
        Os::Windows if host_os() == Os::Windows => build_windows_dynamic_lib(&out_dir)?,
        Os::Linux => build_linux_static_lib(&out_dir)?,
        Os::Macos => build_macos_static_lib(&out_dir)?,
        Os::Android => build_android_dynamic_lib(&out_dir)?,
        _ => (),
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

#[derive(PartialEq, Eq, Clone, Copy)]
enum AndroidTarget {
    Aarch64, // "aarch64"
    X86,     // "x86_64"
    Armv7,   // "armv7"
    I686,    // "i686"
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Libc {
    /// glibc
    Gnu,
    /// musl libc
    Musl,
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

const fn host_os() -> Os {
    // this ugliness is a limitation of rust, where we can't directly
    // access the target triple of the build script.
    const HOST: Os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        panic!("Unsupported host OS")
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
    const ARCH: Arch = if cfg!(target_arch = "x86_64") {
        Arch::Amd64
    } else if cfg!(target_arch = "aarch64") {
        Arch::Arm64
    } else {
        panic!("Unsupported host architecture")
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

// https://doc.rust-lang.org/reference/conditional-compilation.html#target_env
fn target_libc() -> anyhow::Result<Libc> {
    let target_arch = env::var("CARGO_CFG_TARGET_ENV").context("Missing 'CARGO_CFG_TARGET_ENV")?;
    match target_arch.as_str() {
        "gnu" => Ok(Libc::Gnu),
        "musl" => Ok(Libc::Musl),
        _ => bail!("Unsupported target ABI/libc: {target_arch}"),
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

/// Compile libwg and place it in `OUT_DIR`.
fn build_linux_static_lib(out_dir: &str) -> anyhow::Result<()> {
    let out_file = format!("{out_dir}/libwg.a");
    let mut go_build = Command::new("go");
    go_build
        .env("CGO_ENABLED", "1")
        .current_dir("./libwg")
        .args(["build", "-v", "-o", &out_file])
        .args(["--tags", "daita"])
        // Build static lib
        .args(["-buildmode", "c-archive"])
        .env("GOOS", "linux");

    let target_arch = target_arch()?;
    match target_arch {
        Arch::Amd64 => go_build.env("GOARCH", "amd64"),
        Arch::Arm64 => go_build.env("GOARCH", "arm64"),
    };

    if is_cross_compiling()? {
        match (target_arch, target_libc()?) {
            (Arch::Arm64, Libc::Gnu) => go_build.env("CC", "aarch64-linux-gnu-gcc"),
            (Arch::Arm64, Libc::Musl) => go_build.env("CC", "aarch64-linux-musl-gcc"),
            (Arch::Amd64, _) => bail!("cross-compiling to linux x86_64 is not implemented"),
        };
    }

    exec(go_build)?;

    // make sure to link to the resulting binary
    println!("cargo::rustc-link-search={out_dir}");
    println!("cargo::rustc-link-lib=static=wg");

    Ok(())
}

/// Compile libwg and place it in `OUT_DIR`.
fn build_macos_static_lib(out_dir: &str) -> anyhow::Result<()> {
    let out_file = format!("{out_dir}/libwg.a");
    let mut go_build = Command::new("go");
    go_build
        .env("CGO_ENABLED", "1")
        .current_dir("./libwg")
        .args(["build", "-v", "-o", &out_file])
        .args(["--tags", "daita"])
        // Build static lib
        .args(["-buildmode", "c-archive"])
        .env("GOOS", "darwin");

    let target_arch = target_arch()?;
    match target_arch {
        Arch::Amd64 => go_build.env("GOARCH", "amd64"),
        Arch::Arm64 => go_build.env("GOARCH", "arm64"),
    };

    if is_cross_compiling()? {
        let sdkroot = env::var("SDKROOT").context("Missing 'SDKROOT'")?;

        let c_arch = match target_arch {
            Arch::Amd64 => "x86_64",
            Arch::Arm64 => "arm64",
        };

        let xcrun_output = exec(Command::new("xcrun").args(["-sdk", &sdkroot, "--find", "clang"]))?;
        go_build.env("CC", xcrun_output);

        let cflags = format!("-isysroot {sdkroot} -arch {c_arch} -I{sdkroot}/usr/include");
        go_build.env("CFLAGS", cflags);
        go_build.env("CGO_CFLAGS", format!("-isysroot {sdkroot} -arch {c_arch}"));
        go_build.env("CGO_LDFLAGS", format!("-isysroot {sdkroot} -arch {c_arch}"));
        go_build.env("LD_LIBRARY_PATH", format!("{sdkroot}/usr/lib"));
    }

    exec(go_build)?;

    println!("cargo::rustc-link-search={out_dir}");
    println!("cargo::rustc-link-lib=static=wg");

    Ok(())
}

/// Return whether compiling for an architecture or OS other than the host
fn is_cross_compiling() -> anyhow::Result<bool> {
    Ok(host_os() != target_os()? || host_arch() != target_arch()?)
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

/// Compile libwg as a dynamic library for android and place it in [`android_output_path`].
// NOTE: We use dynamic linking as Go cannot produce static binaries specifically for Android.
fn build_android_dynamic_lib(out_dir: &str) -> anyhow::Result<()> {
    let target_triple = env::var("TARGET").context("Missing 'TARGET'")?;
    let target = AndroidTarget::from_str(&target_triple)?;

    // This will either trigger a rebuild if any changes have been made to the libwg code
    // or if the libwg.so file has been changed. The latter is required since the
    // libwg.so file could be deleted. It however means that this build will need
    // to run two times before it is properly cached.
    // FIXME: Figure out a way to do this better. This is tracked in DROID-1697.
    println!(
        "cargo::rerun-if-changed={}",
        android_output_path(target)?.join("libwg.so").display()
    );
    println!("cargo::rerun-if-changed={}", libwg_path()?.display());

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
        // Note: -w -s results in a stripped binary.
        // Note: -Wl -z and max-page-size is added to support 16KB page size.
        // See the link below for more information.
        // https://developer.android.com/guide/practices/page-sizes#other-build-systems
        .env("LDFLAGS", format!("-L{out_dir} -w -s -Wl -z max-page-size=16384"))
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

    // Enable the DAITA (rust) feature flag
    println!(r#"cargo::rustc-cfg=daita"#);

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

    let mut copy_command = Command::new("cp");
    // -p command is required to preserve ownership and timestamp of the file to prevent a
    // rebuild of this module every time.
    copy_command
        .arg("-p")
        .arg(binary.to_str().unwrap())
        .arg(output.to_str().unwrap());

    exec(&mut copy_command)?;

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
    let relative_output_path =
        Path::new("../android/app/build/rustJniLibs/android").join(android_abi(target));
    std::fs::create_dir_all(relative_output_path.clone())?;
    let output_path = relative_output_path.canonicalize()?;
    Ok(output_path)
}

// Return the path of the libwg folder so that we can trigger rebuilds when any code is
fn libwg_path() -> anyhow::Result<PathBuf> {
    let relative_output_path = Path::new("libwg");
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
