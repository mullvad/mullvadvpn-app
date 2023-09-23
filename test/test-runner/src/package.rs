#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::path::Path;
use std::{
    collections::HashMap,
    process::{Output, Stdio},
};
use test_rpc::package::{Error, Package, Result};
use tokio::process::Command;

#[cfg(target_os = "linux")]
pub async fn uninstall_app(env: HashMap<String, String>) -> Result<()> {
    match get_distribution()? {
        Distribution::Debian | Distribution::Ubuntu => {
            uninstall_dpkg("mullvad-vpn", env, true).await
        }
        Distribution::Fedora => uninstall_rpm("mullvad-vpn", env).await,
    }
}

#[cfg(target_os = "macos")]
pub async fn uninstall_app(env: HashMap<String, String>) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    // Uninstall uses sudo -- patch sudoers to not strip env vars
    let mut sudoers = tokio::fs::OpenOptions::new()
        .append(true)
        .open("/etc/sudoers")
        .await
        .map_err(|e| strip_error(Error::WriteFile, e))?;

    for k in env.keys() {
        sudoers
            .write_all(format!("\nDefaults env_keep += \"{k}\"").as_bytes())
            .await
            .map_err(|e| strip_error(Error::WriteFile, e))?;
    }
    drop(sudoers);

    // Run uninstall script, answer yes to everything
    let mut cmd = Command::new("zsh");
    cmd.arg("-c");
    cmd.arg(
        "\"/Applications/Mullvad VPN.app/Contents/Resources/uninstall.sh\" << EOF
y
y
y
EOF",
    );
    cmd.envs(env);
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("uninstall.sh", output))
}

#[cfg(target_os = "windows")]
pub async fn uninstall_app(env: HashMap<String, String>) -> Result<()> {
    // TODO: obtain from registry
    // TODO: can this mimic an actual uninstall more closely?

    let program_dir = Path::new(r"C:\Program Files\Mullvad VPN");
    let uninstall_path = program_dir.join("Uninstall Mullvad VPN.exe");

    // To wait for the uninstaller, we must copy it to a temporary directory and
    // supply it with the install path.

    let temp_uninstaller = std::env::temp_dir().join("mullvad_uninstall.exe");
    tokio::fs::copy(uninstall_path, &temp_uninstaller)
        .await
        .map_err(|e| strip_error(Error::CreateTempUninstaller, e))?;

    let mut cmd = Command::new(temp_uninstaller);

    cmd.kill_on_drop(true);
    cmd.arg("/allusers");
    // Silent mode
    cmd.arg("/S");
    // NSIS doesn't understand that it shouldn't fork itself unless
    // there's whitespace prepended to "_?=".
    cmd.arg(format!(" _?={}", program_dir.display()));
    cmd.envs(env);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("uninstall app", output))
}

#[cfg(target_os = "windows")]
pub async fn install_package(package: Package) -> Result<()> {
    install_nsis_exe(&package.path).await
}

#[cfg(target_os = "linux")]
pub async fn install_package(package: Package) -> Result<()> {
    match get_distribution()? {
        Distribution::Debian | Distribution::Ubuntu => install_dpkg(&package.path).await,
        Distribution::Fedora => install_rpm(&package.path).await,
    }
}

#[cfg(target_os = "macos")]
pub async fn install_package(package: Package) -> Result<()> {
    let mut cmd = Command::new("/usr/sbin/installer");
    cmd.arg("-pkg");
    cmd.arg(package.path);
    cmd.arg("-target");
    cmd.arg("/");
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("installer -pkg", output))
}

#[cfg(target_os = "linux")]
async fn install_dpkg(path: &Path) -> Result<()> {
    let mut cmd = Command::new("/usr/bin/dpkg");
    cmd.arg("-i");
    cmd.arg(path.as_os_str());
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("dpkg -i", output))
}

#[cfg(target_os = "linux")]
async fn uninstall_dpkg(name: &str, env: HashMap<String, String>, purge: bool) -> Result<()> {
    let action;
    let mut cmd = Command::new("/usr/bin/dpkg");
    if purge {
        action = "dpkg --purge";
        cmd.args(["--purge", name]);
    } else {
        action = "dpkg -r";
        cmd.args(["-r", name]);
    }
    cmd.envs(env);
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output(action, output))
}

#[cfg(target_os = "linux")]
async fn install_rpm(path: &Path) -> Result<()> {
    use std::time::Duration;

    const MAX_INSTALL_ATTEMPTS: usize = 5;
    const RETRY_SUBSTRING: &[u8] = b"Failed to download";
    const RETRY_WAIT_INTERVAL: Duration = Duration::from_secs(3);

    let mut cmd = Command::new("/usr/bin/dnf");
    cmd.args(["install", "-y"]);
    cmd.arg(path.as_os_str());
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut attempt = 0;
    let mut output;

    loop {
        output = cmd
            .spawn()
            .map_err(|e| strip_error(Error::RunApp, e))?
            .wait_with_output()
            .await
            .map_err(|e| strip_error(Error::RunApp, e))?;

        let should_retry = !output.status.success()
            && output
                .stderr
                .windows(RETRY_SUBSTRING.len())
                .any(|slice| slice == RETRY_SUBSTRING);
        attempt += 1;
        if should_retry && attempt < MAX_INSTALL_ATTEMPTS {
            log::debug!("Retrying package install: retry attempt {}", attempt);
            tokio::time::sleep(RETRY_WAIT_INTERVAL).await;
            continue;
        }

        return result_from_output("dnf install", output);
    }
}

#[cfg(target_os = "linux")]
async fn uninstall_rpm(name: &str, env: HashMap<String, String>) -> Result<()> {
    let mut cmd = Command::new("/usr/bin/dnf");
    cmd.args(["remove", "-y", name]);
    cmd.envs(env);
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("dnf remove", output))
}

#[cfg(target_os = "windows")]
async fn install_nsis_exe(path: &Path) -> Result<()> {
    log::info!("Installing {}", path.display());
    let mut cmd = Command::new(path);

    cmd.kill_on_drop(true);

    // Run the installer in silent mode
    cmd.arg("/S");

    cmd.spawn()
        .map_err(|e| strip_error(Error::RunApp, e))?
        .wait_with_output()
        .await
        .map_err(|e| strip_error(Error::RunApp, e))
        .and_then(|output| result_from_output("install app", output))
}

#[cfg(target_os = "linux")]
enum Distribution {
    Debian,
    Ubuntu,
    Fedora,
}

#[cfg(target_os = "linux")]
fn get_distribution() -> Result<Distribution> {
    let os_release =
        rs_release::get_os_release().map_err(|_error| Error::UnknownOs("unknown".to_string()))?;
    match os_release
        .get("id")
        .or(os_release.get("ID"))
        .ok_or(Error::UnknownOs("unknown".to_string()))?
        .as_str()
    {
        "debian" => Ok(Distribution::Debian),
        "ubuntu" => Ok(Distribution::Ubuntu),
        "fedora" => Ok(Distribution::Fedora),
        os => Err(Error::UnknownOs(os.to_string())),
    }
}

fn strip_error<T: std::error::Error>(error: Error, source: T) -> Error {
    log::error!("Error: {error}\ncause: {source}");
    error
}

fn result_from_output(action: &'static str, output: Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stdout_str = std::str::from_utf8(&output.stdout).unwrap_or("non-utf8 string");
    let stderr_str = std::str::from_utf8(&output.stderr).unwrap_or("non-utf8 string");

    log::error!(
        "{action} failed:\n\nstdout:\n\n{}\n\nstderr:\n\n{}",
        stdout_str,
        stderr_str
    );

    Err(output
        .status
        .code()
        .map(Error::InstallerFailed)
        .unwrap_or(Error::InstallerFailedSignal))
}
