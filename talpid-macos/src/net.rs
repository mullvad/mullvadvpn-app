use anyhow::{Context, anyhow, bail};
use std::net::IpAddr;
use tokio::process::Command;

/// Adds an alias to a network interface.
pub async fn add_alias(interface: &str, addr: IpAddr) -> anyhow::Result<()> {
    let context = || anyhow!("Failed to add interface {interface} alias {addr}");
    let output = Command::new("ifconfig")
        .args([interface, "alias", &format!("{addr}"), "up"])
        .output()
        .await
        .context("Failed to spawn ifconfig")
        .with_context(context)?;

    if !output.status.success() {
        bail!(
            "{}: Non-zero exit code from ifconfig: {}",
            context(),
            output.status
        );
    }

    Ok(())
}

/// Removes an alias from a network interface.
pub async fn remove_alias(interface: &str, addr: IpAddr) -> anyhow::Result<()> {
    let context = || anyhow!("Failed to remove interface {interface} alias {addr}");
    let output = Command::new("ifconfig")
        .args([interface, "delete", &format!("{addr}")])
        .output()
        .await
        .context("Failed to spawn ifconfig")
        .with_context(context)?;

    if !output.status.success() {
        bail!(
            "{}: Non-zero exit code from ifconfig: {}",
            context(),
            output.status
        );
    }

    Ok(())
}
