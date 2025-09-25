use super::receive_confirmation;
use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;

pub async fn handle_factory_reset(assume_yes: bool) -> Result<()> {
    if !assume_yes && !receive_confirmation("Are you sure you want to disconnect, log out, delete all settings, logs and cache files for the Mullvad VPN system service?", false).await {
        return Ok(());
    }
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.factory_reset().await?;
    #[cfg(target_os = "linux")]
    println!("If you're running systemd, to remove all logs, you must use journalctl");
    Ok(())
}

pub async fn handle_settings_reset(assume_yes: bool) -> Result<()> {
    if !assume_yes
        && !receive_confirmation(
            "Are you sure you want to reset all settings to the default?",
            false,
        )
        .await
    {
        return Ok(());
    }
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.reset_settings().await?;
    Ok(())
}
