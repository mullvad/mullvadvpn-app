#![cfg(target_os = "linux")]

use nix::unistd::geteuid;
use tokio::process::Command;

/// Re-launch self with rootlesskit if we're not root.
/// Allows for rootless and containerized networking.
/// The VNC port is published to localhost.
pub async fn relaunch_with_rootlesskit(vnc_port: Option<u16>) {
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
