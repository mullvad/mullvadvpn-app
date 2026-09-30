use crate::{
    config::{OsType, Provisioner, VmConfig},
    package,
};
use anyhow::{Context, Result, bail};
use ssh2::{File, Session};
use std::{
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
    time::Instant,
};
use test_rpc::UNPRIVILEGED_USER;

const BOOTSTRAP_SCRIPT_NAME: &str = "ssh-setup.sh";
/// Script for bootstrapping the test-runner on Linux and macOS after the test-manager has
/// successfully logged in.
const BOOTSTRAP_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/",
    "ssh-setup.sh"
));

const WINDOWS_BOOTSTRAP_SCRIPT_NAME: &str = "ssh-setup.ps1";
/// Script for bootstrapping the test-runner on Windows after the test-manager has successfully
/// logged in.
const WINDOWS_BOOTSTRAP_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/",
    "ssh-setup.ps1"
));

/// Returns the directory in the test runner where the test-runner binary is installed.
pub async fn provision(
    config: &VmConfig,
    instance: &dyn super::VmInstance,
    app_manifest: &package::Manifest,
    runner_dir: PathBuf,
) -> Result<String> {
    match config.provisioner {
        Provisioner::Ssh => {
            log::debug!("SSH provisioning");

            let (user, password) = config.get_ssh_options().context("missing SSH config")?;
            provision_ssh(
                instance,
                config.os_type,
                &runner_dir,
                app_manifest,
                user,
                password,
            )
            .await
        }
        Provisioner::Noop => {
            let dir = config
                .artifacts_dir
                .as_ref()
                .context("'artifacts_dir' must be set to a mountpoint")?;
            Ok(dir.clone())
        }
    }
}

/// Returns the directory in the test runner where the test-runner binary is installed.
async fn provision_ssh(
    instance: &dyn super::VmInstance,
    os_type: OsType,
    local_runner_dir: &Path,
    local_app_manifest: &package::Manifest,
    user: &str,
    password: &str,
) -> Result<String> {
    let guest_ip = *instance.get_ip();

    let user = user.to_owned();
    let password = password.to_owned();

    let local_runner_dir = local_runner_dir.to_owned();
    let local_app_manifest = local_app_manifest.to_owned();

    let remote_dir = tokio::task::spawn_blocking(move || {
        const SSH_TIMEOUT: Duration = Duration::from_secs(120);
        let started = Instant::now();
        loop {
            match blocking_ssh(
                user.clone(),
                password.clone(),
                guest_ip,
                os_type,
                &local_runner_dir,
                local_app_manifest.clone(),
            ) {
                Err(err) if started.elapsed() < SSH_TIMEOUT => {
                    log::warn!("{:#}", err.context("Failed to provision over SSH"));
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                res => break res,
            }
        }
    })
    .await
    .context("Failed to join SSH task")??;

    Ok(remote_dir)
}

/// Returns the remote runner directory
fn blocking_ssh(
    user: String,
    password: String,
    guest_ip: IpAddr,
    os_type: OsType,
    local_runner_dir: &Path,
    local_app_manifest: package::Manifest,
) -> Result<String> {
    let remote_dir = match os_type {
        OsType::Windows => r"C:\testing",
        OsType::Macos | OsType::Linux => "/opt/testing",
    };

    let exe_suffix = match os_type {
        OsType::Windows => ".exe",
        OsType::Macos | OsType::Linux => "",
    };

    // Directory that receives the payload. Any directory that the SSH user has access to.
    let remote_temp_dir = match os_type {
        OsType::Windows => r"c:\temp",
        OsType::Macos | OsType::Linux => r"/tmp/",
    };

    let stream = {
        let ssh = SocketAddr::new(guest_ip, 22);
        log::debug!("Connecting to {user}@{ssh} over ssh");
        TcpStream::connect(ssh).context("TCP connect failed")?
    };

    let mut session = Session::new().context("Failed to connect to SSH server")?;
    session.set_tcp_stream(stream);
    session.handshake()?;

    session
        .userauth_password(&user, &password)
        .context("SSH auth failed")?;

    if os_type == OsType::Windows {
        // There is a problem with the `ssh2` crate (both with scp and sftp) that we can not create
        // new directories on Windows, so create the directory using a command instead.
        let cmd = format!(
            r#"powershell -NoProfile -NonInteractive -Command "New-Item -ItemType Directory -Force -Path '{remote_temp_dir}' | Out-Null""#
        );
        ssh_exec(&session, &cmd)
            .map(drop)
            .with_context(|| format!("Failed to create '{remote_temp_dir}' on remote"))?;
    }

    let temp_dir = Path::new(remote_temp_dir);
    // Transfer a test runner
    let source = local_runner_dir.join(format!("test-runner{exe_suffix}"));
    ssh_send_file_with_opts(&session, &source, temp_dir, FileOpts { executable: true })
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    // Transfer connection-checker
    let source = local_runner_dir.join(format!("connection-checker{exe_suffix}"));
    ssh_send_file_with_opts(&session, &source, temp_dir, FileOpts { executable: true })
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    // Transfer app packages
    let source = &local_app_manifest.app_package_path;
    ssh_send_file_with_opts(&session, source, temp_dir, FileOpts { executable: true })
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    if let Some(source) = &local_app_manifest.app_package_to_upgrade_from_path {
        ssh_send_file_with_opts(&session, source, temp_dir, FileOpts { executable: true })
            .with_context(|| format!("Failed to send '{source:?}' to remote"))?;
    } else {
        log::warn!("No previous app package to upgrade from to send to remote")
    }
    if let Some(source) = &local_app_manifest.gui_package_path {
        ssh_send_file_with_opts(&session, source, temp_dir, FileOpts { executable: true })
            .with_context(|| format!("Failed to send '{source:?}' to remote"))?;
    } else {
        log::warn!("No UI e2e test to send to remote")
    }

    let app_package_path = file_name(&local_app_manifest.app_package_path);
    let app_package_to_upgrade_from_path = local_app_manifest
        .app_package_to_upgrade_from_path
        .as_deref()
        .map(file_name);
    let gui_package_path = local_app_manifest
        .gui_package_path
        .as_deref()
        .map(file_name);

    // Transfer setup script
    let cmd = match os_type {
        OsType::Linux | OsType::Macos => {
            let bootstrap_script_dest = temp_dir.join(BOOTSTRAP_SCRIPT_NAME);
            ssh_write_with_opts(
                &session,
                &bootstrap_script_dest,
                BOOTSTRAP_SCRIPT,
                FileOpts { executable: true },
            )
            .context("failed to send bootstrap script to remote")?;

            format!(
                r#"sudo {} {remote_dir} "{app_package_path}" "{}" "{}" "{UNPRIVILEGED_USER}""#,
                bootstrap_script_dest.display(),
                app_package_to_upgrade_from_path.unwrap_or_default(),
                gui_package_path.unwrap_or_default(),
            )
        }
        OsType::Windows => {
            let bootstrap_script_dest = temp_dir.join(WINDOWS_BOOTSTRAP_SCRIPT_NAME);
            ssh_write_with_opts(
                &session,
                &bootstrap_script_dest,
                WINDOWS_BOOTSTRAP_SCRIPT,
                FileOpts::default(),
            )
            .context("failed to send bootstrap script to remote")?;

            // Optional arguments are omitted rather than passed as empty strings, since
            // `powershell -File` does not reliably forward empty arguments.
            let mut cmd = format!(
                r#"powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{}" -RunnerDir "{remote_dir}" -AppPackage "{app_package_path}""#,
                bootstrap_script_dest.display(),
            );
            if let Some(path) = app_package_to_upgrade_from_path {
                cmd.push_str(&format!(r#" -PreviousApp "{path}""#));
            }
            if let Some(path) = gui_package_path {
                cmd.push_str(&format!(r#" -UiRunner "{path}""#));
            }
            cmd
        }
    };

    // Run the setup script in the test runner
    log::debug!("Running setup script on remote, cmd: {cmd}");
    ssh_exec(&session, &cmd)
        .map(drop)
        .context("Failed to run setup script")?;

    Ok(remote_dir.to_string())
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

/// Copy a `source` file to `dest_dir` in the test runner with opts.
///
/// Returns the absolute path in the test runner where the file is stored.
fn ssh_send_file_with_opts<P: AsRef<Path> + Copy>(
    session: &Session,
    source: P,
    dest_dir: &Path,
    opts: FileOpts,
) -> Result<PathBuf> {
    let dest = dest_dir.join(
        source
            .as_ref()
            .file_name()
            .context("Missing source file name")?,
    );

    log::debug!(
        "Copying file to remote: {} -> {}",
        source.as_ref().display(),
        dest.display(),
    );

    let source = std::fs::read(source)
        .with_context(|| format!("Failed to open file at {}", source.as_ref().display()))?;

    ssh_write_with_opts(session, &dest, &source[..], opts)?;

    Ok(dest)
}

/// Create a new file with opts at location `dest` and write the content of `source` into it.
/// Returns a handle to the newly created file.
fn ssh_write_with_opts<P: AsRef<Path>>(
    session: &Session,
    dest: P,
    mut source: impl Read,
    opts: FileOpts,
) -> Result<File> {
    let sftp = session.sftp()?;
    let mut remote_file = sftp.create(dest.as_ref())?;

    io::copy(&mut source, &mut remote_file).context("failed to write file")?;

    if opts.executable {
        make_executable(&mut remote_file)?;
    };

    Ok(remote_file)
}

/// Extra options that may be necessary to configure for files written to the test runner VM.
/// Used in conjunction with the `ssh_*_with_opts` functions.
#[derive(Clone, Copy, Debug, Default)]
struct FileOpts {
    /// If file should be executable.
    executable: bool,
}

fn make_executable(file: &mut File) -> Result<()> {
    // Make sure that the script is executable!
    let mut file_stat = file.stat()?;
    // 0x111 is the executable bit for Owner/Group/Public
    let perm = file_stat.perm.map(|perm| perm | 0x111).unwrap_or(0x111);
    file_stat.perm = Some(perm);
    file.setstat(file_stat)?;
    Ok(())
}

/// Execute an arbitrary string of commands via ssh.
fn ssh_exec(session: &Session, command: &str) -> Result<String> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;
    let mut stderr_handle = channel.stderr();
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.send_eof()?;
    channel.wait_eof()?;
    channel.wait_close()?;

    let exit_status = channel
        .exit_status()
        .context("Failed to obtain exit status")?;
    if exit_status != 0 {
        let mut stderr = String::new();
        stderr_handle.read_to_string(&mut stderr).unwrap();
        log::error!("Command failed: command: {command}\n\noutput:\n{output}\n\nstderr: {stderr}");
        bail!("command failed: {exit_status}");
    }

    Ok(output)
}
