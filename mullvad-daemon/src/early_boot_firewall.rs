use mullvad_daemon::settings::{self, SettingsPersister};
use talpid_core::firewall::{self, Firewall, FirewallPolicy};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize firewall")]
    Firewall(#[from] firewall::Error),

    #[error("Failed to get settings path")]
    Path(#[from] mullvad_paths::Error),

    #[error("Failed to get settings")]
    Settings(#[from] settings::Error),
}

pub async fn initialize_firewall() -> Result<(), Error> {
    // TODO split-tunneling?
    let mut firewall = Firewall::new(mullvad_types::TUNNEL_FWMARK, None, None)?;
    let allow_lan = get_allow_lan().await.unwrap_or_else(|err| {
        log::info!(
            "Not allowing LAN traffic due to failing to read settings: {}",
            err
        );
        false
    });
    let policy = FirewallPolicy::Blocked {
        allow_lan,
        allowed_endpoint: None,
    };
    log::info!("Applying firewall policy {policy}");
    firewall.apply_policy(policy)?;
    Ok(())
}

async fn get_allow_lan() -> Result<bool, Error> {
    let path = mullvad_paths::settings_dir()?;
    // NOTE: This may fail if the daemon has not been restarted after an upgrade.
    //       This will cause `allow_lan` to be disabled during early boot. This
    //       is probably acceptable.
    let settings = SettingsPersister::read_only(&path).await;
    Ok(settings.allow_lan)
}
