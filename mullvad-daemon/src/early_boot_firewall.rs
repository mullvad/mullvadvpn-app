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
    let settings_dir = mullvad_paths::settings_dir()?;

    // Apply IP split-tunnel rules before blocking firewall policy.
    // This ensures that configured IP ranges can bypass the tunnel from system boot.
    #[cfg(target_os = "linux")]
    if let Err(error) = apply_ip_split_tunnel_early(&settings_dir).await {
        log::warn!(
            "Failed to apply IP split-tunnel rules during early boot: {}. Continuing without IP split-tunneling.",
            error
        );
    }

    let mut firewall = Firewall::new(mullvad_types::TUNNEL_FWMARK, None, None)?;
    let allow_lan = get_allow_lan(&settings_dir).await.unwrap_or_else(|err| {
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

#[cfg(target_os = "linux")]
async fn apply_ip_split_tunnel_early(
    settings_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use ipnetwork::Ipv4Network;
    use serde::Deserialize;
    use std::collections::BTreeSet;

    const STORE_VERSION: u32 = 1;
    const STORE_DIR: &str = "fork";
    const STORE_FILE: &str = "ip-split-tunnel.json";

    #[derive(Deserialize)]
    struct Store {
        version: u32,
        ipv4_ranges: Vec<String>,
    }

    let path = settings_dir.join(STORE_DIR).join(STORE_FILE);
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            log::debug!("No IP split-tunnel ranges configured for early boot");
            return Ok(());
        }
        Err(error) => return Err(Box::new(error)),
    };

    let store: Store = serde_json::from_str(&contents)?;
    if store.version != STORE_VERSION {
        return Err(format!("Unsupported IP split-tunnel settings version: {}", store.version).into());
    }

    let ranges: BTreeSet<Ipv4Network> = store
        .ipv4_ranges
        .iter()
        .filter_map(|range_str| {
            range_str.parse::<Ipv4Network>().ok().and_then(|network| {
                Ipv4Network::new(network.network(), network.prefix()).ok()
            })
        })
        .collect();

    if ranges.is_empty() {
        log::debug!("No valid IP split-tunnel ranges for early boot");
        return Ok(());
    }

    let ranges_vec: Vec<Ipv4Network> = ranges.into_iter().collect();
    log::info!(
        "Applying {} IP split-tunnel range(s) during early boot",
        ranges_vec.len()
    );

    talpid_core::firewall::linux::ip_split_tunnel::apply_ranges(
        mullvad_types::TUNNEL_FWMARK,
        &ranges_vec,
    )?;

    Ok(())
}

async fn get_allow_lan(settings_dir: &std::path::Path) -> Result<bool, Error> {
    // NOTE: This may fail if the daemon has not been restarted after an upgrade.
    //       This will cause `allow_lan` to be disabled during early boot. This
    //       is probably acceptable.
    let settings = SettingsPersister::read_only(settings_dir).await;
    Ok(settings.allow_lan)
}
