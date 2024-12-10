use crate::{
    config::{OsType, Provisioner, VmConfig},
    package,
    tests::config::BOOTSTRAP_SCRIPT,
};
use anyhow::{bail, Context, Result};
use ssh2::Session;
use std::{
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
};
use test_rpc::UNPRIVILEGED_USER;

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

    let remote_dir = match os_type {
        OsType::Windows => r"C:\testing",
        OsType::Macos | OsType::Linux => r"/opt/testing",
    };

    let local_runner_dir = local_runner_dir.to_owned();
    let local_app_manifest = local_app_manifest.to_owned();

    tokio::task::spawn_blocking(move || {
        blocking_ssh(
            user,
            password,
            guest_ip,
            &local_runner_dir,
            local_app_manifest,
            remote_dir,
        )
    })
    .await
    .context("Failed to join SSH task")??;

    Ok(remote_dir.to_string())
}

fn blocking_ssh(
    user: String,
    password: String,
    guest_ip: IpAddr,
    local_runner_dir: &Path,
    local_app_manifest: package::Manifest,
    remote_dir: &str,
) -> Result<()> {
    // Directory that receives the payload. Any directory that the SSH user has access to.
    const REMOTE_TEMP_DIR: &str = "/tmp/";

    let temp_dir = Path::new(REMOTE_TEMP_DIR);

    let stream = TcpStream::connect(SocketAddr::new(guest_ip, 22)).context("TCP connect failed")?;

    let mut session = Session::new().context("Failed to connect to SSH server")?;
    session.set_tcp_stream(stream);
    session.handshake()?;

    session
        .userauth_password(&user, &password)
        .context("SSH auth failed")?;

    // Transfer a test runner
    let source = local_runner_dir.join("test-runner");
    ssh_send_file(&session, &source, temp_dir)
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    // Transfer connection-checker
    let source = local_runner_dir.join("connection-checker");
    ssh_send_file(&session, &source, temp_dir)
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    // Transfer app packages
    let source = &local_app_manifest.app_package_path;
    ssh_send_file(&session, source, temp_dir)
        .with_context(|| format!("Failed to send '{source:?}' to remote"))?;

    if let Some(source) = &local_app_manifest.app_package_to_upgrade_from_path {
        ssh_send_file(&session, source, temp_dir)
            .with_context(|| format!("Failed to send '{source:?}' to remote"))?;
    } else {
        log::warn!("No previous app package to upgrade from to send to remote")
    }
    if let Some(source) = &local_app_manifest.gui_package_path {
        ssh_send_file(&session, source, temp_dir)
            .with_context(|| format!("Failed to send '{source:?}' to remote"))?;
    } else {
        log::warn!("No UI e2e test to send to remote")
    }

    // Transfer setup script
    // TODO: Move this name to a constant somewhere?
    let bootstrap_script_dest = temp_dir.join("ssh-setup.sh");
    ssh_write(&session, &bootstrap_script_dest, BOOTSTRAP_SCRIPT)
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
        .context("Failed to run setup script")
}

/// Copy a `source` file to `dest_dir` in the test runner.
///
/// Returns the aboslute path in the test runner where the file is stored.
fn ssh_send_file<P: AsRef<Path> + Copy>(
    session: &Session,
    source: P,
    dest_dir: &Path,
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

    ssh_write(session, &dest, &source[..])?;

    Ok(dest)
}

/// Analogues to [`std::fs::write`], but over ssh!
fn ssh_write<P: AsRef<Path>>(session: &Session, dest: P, mut source: impl Read) -> Result<()> {
    let sftp = session.sftp()?;
    let mut remote_file = sftp.create(dest.as_ref())?;

    io::copy(&mut source, &mut remote_file).context("failed to write file")?;

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
