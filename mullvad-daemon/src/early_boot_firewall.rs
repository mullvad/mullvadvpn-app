use mullvad_daemon::settings::{self, SettingsPersister};
use talpid_core::firewall::{self, Firewall, FirewallPolicy};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize firewall")]
    Firewall(#[error(source)] firewall::Error),

    #[error(display = "Failed to get settings path")]
    Path(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to get settings")]
    Settings(#[error(source)] settings::Error),
}

pub async fn initialize_firewall() -> Result<(), Error> {
    let mut firewall = Firewall::new(mullvad_types::TUNNEL_FWMARK)?;
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
    let settings = SettingsPersister::load(&path).await;
    Ok(settings.allow_lan)
}
