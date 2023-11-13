use super::receive_confirmation;
use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;

pub async fn handle() -> Result<()> {
    if !receive_confirmation("Are you sure you want to disconnect, log out, delete all settings, logs and cache files for the Mullvad VPN system service?", false).await {
        return Ok(());
    }
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.factory_reset().await?;
    #[cfg(target_os = "linux")]
    println!("If you're running systemd, to remove all logs, you must use journalctl");
    Ok(())
}
