/// Re-launch self with rootlesskit if we're not root.
/// Allows for rootless and containerized networking.
/// The VNC port is published to localhost.
#[cfg(target_os = "linux")]
pub async fn relaunch_with_rootlesskit(vnc_port: Option<u16>) {
    use nix::unistd::geteuid;
    use tokio::process::Command;

    // check if user is root (`man getuid`).
    if geteuid().is_root() {
        return;
    }

    let mut cmd = Command::new("rootlesskit");
    cmd.args([
        "--net",
        "slirp4netns",
        "--ipv6",
        // A higher MTU breaks IPv6
        "--mtu",
        "1500",
        "--copy-up=/etc",
    ]);

    if let Some(port) = vnc_port {
        log::debug!("VNC port: {port} -> 5901/tcp");

        cmd.args([
            "--port-driver",
            "slirp4netns",
            "-p",
            &format!("127.0.0.1:{port}:5901/tcp"),
        ]);
    } else {
        cmd.arg("--disable-host-loopback");
    }

    cmd.args(std::env::args());

    let status = cmd.status().await.unwrap_or_else(|e| {
        panic!("failed to execute [{cmd:?}]: {e}");
    });

    std::process::exit(status.code().unwrap_or(1));
}

#[cfg(target_os = "macos")]
async fn relaunch_with_sudo() {
    // check if user is root (`man getuid`).
    if nix::unistd::geteuid().is_root() {
        return;
    }
    let mut cmd = tokio::process::Command::new("sudo");
    cmd.arg("--preserve-env");
    cmd.args(std::env::args());

    log::info!("Root privileges required. Re-launching with sudo.");
    eprintln!("{cmd:?}");

    let status = cmd.status().await.unwrap_or_else(|e| {
        panic!("failed to execute [{cmd:?}]: {e}");
    });

    std::process::exit(status.code().unwrap_or(1));
}
