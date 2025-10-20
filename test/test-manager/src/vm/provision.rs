use crate::{
    config::{OsType, Provisioner, VmConfig},
    package,
    tests::config::BOOTSTRAP_SCRIPT,
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
use tokio::process::Command;

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
            .context("Failed to provision runner over SSH")
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
            let last_result = blocking_ssh(
                user.clone(),
                password.clone(),
                guest_ip,
                os_type,
                &local_runner_dir,
                local_app_manifest.clone(),
            );
            log::error!("SSH provisioning attempt result: {:?}", last_result);

            {
                let mut r = std::process::Command::new("netstat");
                r.arg("-rn");
                let out = r.output();
                log::error!("netstat output: {:?}", out);
            }
            {
                let mut r = std::process::Command::new("ping");
                r.args(&["-c".to_string(), "1".to_string(), guest_ip.to_string()]);
                let out = r.output();
                log::error!("ping output: {:?}", out);
            }

            if last_result.is_err() && started.elapsed() < SSH_TIMEOUT {
                log::warn!("Failed to provision over SSH, retrying...");
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            break last_result;
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
        // FIXME: There is a problem with the `ssh2` crate (both with scp and sftp) that
        // we can not create new directories, so instead we have to rely on pre-existing
        // directories if we want to create / upload files to the Windows guest. As a
        // workaround, use `C:` as a temporary directory.
        OsType::Windows => "c:",
        OsType::Macos | OsType::Linux => "/opt/testing",
    };

    // Directory that receives the payload. Any directory that the SSH user has access to.
    let remote_temp_dir = match os_type {
        OsType::Windows => r"c:\temp",
        OsType::Macos | OsType::Linux => r"/tmp/",
    };

    let stream = TcpStream::connect(SocketAddr::new(guest_ip, 22)).context("TCP connect failed")?;

    let mut session = Session::new().context("Failed to connect to SSH server")?;
    session.set_tcp_stream(stream);
    session.handshake()?;

    session
        .userauth_password(&user, &password)
        .context("SSH auth failed")?;

    let temp_dir = Path::new(remote_temp_dir);
    // Transfer a test runner
    let source = local_runner_dir.join("test-runner");
    ssh_send_file_with_opts(&session, &source, temp_dir, FileOpts { executable: true })
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    // Transfer connection-checker
    let source = local_runner_dir.join("connection-checker");
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

    // Transfer setup script
    if matches!(os_type, OsType::Linux | OsType::Macos) {
        // TODO: Move this name to a constant somewhere?
        let bootstrap_script_dest = temp_dir.join("ssh-setup.sh");
        ssh_write_with_opts(
            &session,
            &bootstrap_script_dest,
            BOOTSTRAP_SCRIPT,
            FileOpts { executable: true },
        )
        .context("failed to send bootstrap script to remote")?;

        // Run setup script
        let app_package_path = local_app_manifest
            .app_package_path
            .file_name()
            .unwrap()
            .to_string_lossy();
        let app_package_to_upgrade_from_path = local_app_manifest
            .app_package_to_upgrade_from_path
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .unwrap_or_default();
        let gui_package_path = local_app_manifest
            .gui_package_path
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .unwrap_or_default();

        // Run the setup script in the test runner
        let cmd = format!(
            r#"sudo {} {remote_dir} "{app_package_path}" "{app_package_to_upgrade_from_path}" "{gui_package_path}" "{UNPRIVILEGED_USER}""#,
            bootstrap_script_dest.display(),
        );
        log::debug!("Running setup script on remote, cmd: {cmd}");
        ssh_exec(&session, &cmd)
            .map(drop)
            .context("Failed to run setup script")?;
    }

    Ok(remote_dir.to_string())
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
