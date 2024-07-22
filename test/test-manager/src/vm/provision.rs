use crate::{
    config::{OsType, Provisioner, VmConfig},
    package,
};
use anyhow::{bail, Context, Result};
use ssh2::Session;
use std::{
    fs::File,
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpStream},
    path::Path,
};

pub async fn provision(
    config: &VmConfig,
    instance: &dyn super::VmInstance,
    app_manifest: &package::Manifest,
) -> Result<String> {
    match config.provisioner {
        Provisioner::Ssh => {
            log::debug!("SSH provisioning");

            let (user, password) = config.get_ssh_options().context("missing SSH config")?;
            ssh(
                instance,
                config.os_type,
                &config.get_runner_dir(),
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

async fn ssh(
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
    const SCRIPT_PAYLOAD: &[u8] = include_bytes!("../../../scripts/ssh-setup.sh");
    const OPENVPN_CERT: &[u8] = include_bytes!("../../../openvpn.ca.crt");

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
    ssh_send_file_path(&session, &source, temp_dir)
        .context("Failed to send test runner to remote")?;

    // Transfer connection-checker
    let source = local_runner_dir.join("connection-checker");
    ssh_send_file_path(&session, &source, temp_dir)
        .context("Failed to send connection-checker to remote")?;

    // Transfer app packages
    ssh_send_file_path(&session, &local_app_manifest.app_package_path, temp_dir)
        .context("Failed to send current app package to remote")?;
    if let Some(app_package_to_upgrade_from_path) =
        &local_app_manifest.app_package_to_upgrade_from_path
    {
        ssh_send_file_path(&session, app_package_to_upgrade_from_path, temp_dir)
            .context("Failed to send previous app package to remote")?;
    } else {
        log::warn!("No previous app to send to remote")
    }
    if let Some(ui_e2e_tests_path) = &local_app_manifest.ui_e2e_tests_path {
        ssh_send_file_path(&session, ui_e2e_tests_path, temp_dir)
            .context("Failed to send ui_e2e_tests_path to remote")?;
    } else {
        log::warn!("No UI e2e test to send to remote")
    }

    // Transfer openvpn cert
    let dest: std::path::PathBuf = temp_dir.join("openvpn.ca.crt");
    log::debug!("Copying remote openvpn.ca.crt -> {}", dest.display());
    #[allow(const_item_mutation)]
    ssh_send_file(
        &session,
        &mut OPENVPN_CERT,
        u64::try_from(OPENVPN_CERT.len()).expect("cert too long"),
        &dest,
    )
    .context("failed to send openvpn crt to remote")?;

    // Transfer setup script
    let dest = temp_dir.join("ssh-setup.sh");
    log::debug!("Copying remote setup script -> {}", dest.display());
    #[allow(const_item_mutation)]
    ssh_send_file(
        &session,
        &mut SCRIPT_PAYLOAD,
        u64::try_from(SCRIPT_PAYLOAD.len()).expect("script too long"),
        &dest,
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
    let ui_e2e_tests_path = local_app_manifest
        .ui_e2e_tests_path
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .unwrap_or_default();

    let cmd = format!(
        "sudo {} {remote_dir} \"{app_package_path}\" \"{app_package_to_upgrade_from_path}\" \"{ui_e2e_tests_path}\"",
        dest.display()
    );
    log::debug!("Running setup script on remote, cmd: {cmd}");
    ssh_exec(&session, &cmd)
        .map(drop)
        .context("Failed to run setup script")
}

fn ssh_send_file_path(session: &Session, source: &Path, dest_dir: &Path) -> Result<()> {
    let dest = dest_dir.join(source.file_name().context("Missing source file name")?);

    log::debug!(
        "Copying file to remote: {} -> {}",
        source.display(),
        dest.display(),
    );

    let mut file =
        File::open(source).with_context(|| format!("Failed to open file at {source:?}"))?;
    let file_len = file
        .metadata()
        .with_context(|| format!("Failed to get file size of {source:?}"))?
        .len();
    ssh_send_file(session, &mut file, file_len, &dest)
}

fn ssh_send_file<R: Read>(
    session: &Session,
    source: &mut R,
    source_len: u64,
    dest: &Path,
) -> Result<()> {
    let mut remote_file = session.scp_send(dest, 0o744, source_len, None)?;
    io::copy(source, &mut remote_file).context("failed to write file")?;
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;
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
