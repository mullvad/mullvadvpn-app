use crate::{MullvadProxyClient, Result};

pub async fn print() -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;
    let current_version = rpc.get_current_version().await?;
    println!("{:21}: {}", "Current version", current_version);
    let version_info = rpc.get_version_info().await?;
    println!("{:21}: {}", "Is supported", version_info.supported);

    if let Some(suggested_upgrade) = version_info.suggested_upgrade {
        println!("{:21}: {}", "Suggested upgrade", suggested_upgrade);
    } else {
        println!("{:21}: none", "Suggested upgrade");
    }

    if !version_info.latest_stable.is_empty() {
        println!(
            "{:21}: {}",
            "Latest stable version", version_info.latest_stable
        );
    }

    let settings = rpc.get_settings().await?;
    if settings.show_beta_releases {
        println!("{:21}: {}", "Latest beta version", version_info.latest_beta);
    };

    Ok(())
}
