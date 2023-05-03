use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;
use std::io::stdin;

pub async fn handle() -> Result<()> {
    if receive_confirmation().await {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.factory_reset().await?;
        #[cfg(target_os = "linux")]
        println!("If you're running systemd, to remove all logs, you must use journalctl");
    }
    Ok(())
}

async fn receive_confirmation() -> bool {
    println!("Are you sure you want to disconnect, log out, delete all settings, logs and cache files for the Mullvad VPN system service? [Yes/No (default)]");

    tokio::task::spawn_blocking(|| loop {
        let mut buf = String::new();
        if let Err(e) = stdin().read_line(&mut buf) {
            eprintln!("Couldn't read from STDIN: {e}");
            return false;
        }
        match buf.trim() {
            "Yes" => return true,
            "No" | "no" | "" => return false,
            _ => eprintln!("Unexpected response. Please enter \"Yes\" or \"No\""),
        }
    })
    .await
    .unwrap()
}
