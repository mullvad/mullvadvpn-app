use ipnetwork::{IpNetwork, Ipv4Network};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    str::FromStr,
};
use talpid_routing::RouteManagerHandle;
use talpid_types::ErrorExt;

const STORE_VERSION: u32 = 1;
const STORE_DIR: &str = "fork";
const STORE_FILE: &str = "ip-split-tunnel.json";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid IPv4 range: {0}")]
    InvalidRange(String),
    #[error("IP split-tunnel range must be IPv4")]
    NonIpv4Range,
    #[error("Unable to read IP split-tunnel settings")]
    ReadStore(#[source] std::io::Error),
    #[error("Unable to parse IP split-tunnel settings")]
    ParseStore(#[source] serde_json::Error),
    #[error("Unsupported IP split-tunnel settings version: {0}")]
    UnsupportedStoreVersion(u32),
    #[error("Unable to create IP split-tunnel settings directory")]
    CreateStoreDir(#[source] std::io::Error),
    #[error("Unable to write IP split-tunnel settings")]
    WriteStore(#[source] std::io::Error),
    #[error("Firewall error")]
    Firewall(#[source] talpid_core::firewall::Error),
    #[error("Route manager error")]
    RouteManager(#[source] talpid_routing::Error),
}

#[derive(Debug, Deserialize, Serialize)]
struct Store {
    version: u32,
    ipv4_ranges: Vec<String>,
}

pub struct IpSplitTunnel {
    path: PathBuf,
    ranges: BTreeSet<Ipv4Network>,
    route_manager: RouteManagerHandle,
    applied_routes: Vec<Ipv4Network>,
    enabled: bool,
}

pub enum Command {
    List(crate::ResponseTx<Vec<String>, crate::Error>),
    Add(crate::ResponseTx<(), crate::Error>, String),
    Remove(crate::ResponseTx<(), crate::Error>, String),
    Clear(crate::ResponseTx<(), crate::Error>),
    Check(crate::ResponseTx<Vec<CheckResult>, crate::Error>, Option<String>),
    Toggle(crate::ResponseTx<bool, crate::Error>),
}

impl Command {
    pub fn with_tunnel_interface(self, tunnel_interface: Option<String>) -> Self {
        match self {
            Command::Check(tx, _) => Command::Check(tx, tunnel_interface),
            command => command,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub range: String,
    pub sample_ip: Ipv4Addr,
    pub bypasses_tunnel: bool,
    pub interface: Option<String>,
    pub error: Option<String>,
}

impl IpSplitTunnel {
    pub async fn new(
        settings_dir: impl AsRef<Path>,
        route_manager: RouteManagerHandle,
    ) -> Result<Self, Error> {
        let path = settings_dir.as_ref().join(STORE_DIR).join(STORE_FILE);
        let ranges = load_ranges(&path).await?;

        let mut ip_split_tunnel = Self {
            path,
            ranges,
            route_manager,
            applied_routes: Vec::new(),
            enabled: true, // Enabled by default
        };
        
        // Try to load enabled state, but don't fail if it doesn't exist
        let _ = ip_split_tunnel.load_enabled_state().await;
        
        Ok(ip_split_tunnel)
    }

    pub fn list(&self) -> Vec<String> {
        self.ranges.iter().map(ToString::to_string).collect()
    }

    pub async fn add(&mut self, range: &str) -> Result<(), Error> {
        let range = parse_ipv4_range(range)?;
        self.ranges.insert(range);
        self.save_and_apply().await
    }

    pub async fn remove(&mut self, range: &str) -> Result<(), Error> {
        let range = parse_ipv4_range(range)?;
        self.ranges.remove(&range);
        self.save_and_apply().await
    }

    pub async fn clear(&mut self) -> Result<(), Error> {
        self.ranges.clear();
        self.save_and_apply().await
    }

    pub async fn apply(&mut self) -> Result<(), Error> {
        // If disabled, clear all rules and return
        if !self.enabled {
            self.clear_applied().await?;
            self.applied_routes.clear();
            return Ok(());
        }

        let ranges = self.ranges.iter().copied().collect::<Vec<_>>();
        
        // Apply firewall marks - this marks packets destined for whitelisted IPs
        // so they bypass the tunnel routing table
        talpid_core::firewall::linux::ip_split_tunnel::apply_ranges(
            mullvad_types::TUNNEL_FWMARK,
            &ranges,
        )
        .map_err(Error::Firewall)?;

        // The route manager should handle adding routes to the main table
        // for whitelisted IPs. This is handled by the daemon's tunnel state
        // transition which calls apply() when connected.
        
        self.applied_routes = ranges;
        Ok(())
    }

    pub async fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        
        // Save the enabled state
        if let Err(e) = self.save_enabled_state().await {
            log::error!("Failed to save IP split-tunnel enabled state: {}", e);
        }
        
        // Apply or clear based on new state
        if let Err(e) = self.apply().await {
            log::error!("Failed to apply IP split-tunnel after toggle: {}", e);
        }
        
        self.enabled
    }

    async fn save_enabled_state(&self) -> Result<(), Error> {
        // Save enabled state in a separate file
        let enabled_path = self.path.with_file_name("ip-split-tunnel-enabled.json");
        let state = serde_json::json!({ "enabled": self.enabled });
        let contents = serde_json::to_vec(&state).map_err(Error::ParseStore)?;
        tokio::fs::write(&enabled_path, contents)
            .await
            .map_err(Error::WriteStore)
    }

    async fn load_enabled_state(&mut self) -> Result<(), Error> {
        let enabled_path = self.path.with_file_name("ip-split-tunnel-enabled.json");
        let contents = match tokio::fs::read_to_string(&enabled_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.enabled = true; // Default to enabled
                return Ok(());
            }
            Err(e) => return Err(Error::ReadStore(e)),
        };
        
        let state: serde_json::Value = serde_json::from_str(&contents).map_err(Error::ParseStore)?;
        self.enabled = state.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        if let Err(error) = self.clear_applied().await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to clear IP split-tunnel rules")
            );
        }
        // Also clear routes
        if let Err(e) = self.route_manager.clear_routes() {
            log::warn!("Failed to clear IP split-tunnel routes on shutdown: {}", e);
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::List(tx) => {
                let _ = tx.send(Ok(self.list()));
            }
            Command::Add(tx, range) => {
                let result = self
                    .add(&range)
                    .await
                    .map_err(crate::Error::IpSplitTunnel)
                    .inspect_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Unable to add IP split-tunnel range")
                        );
                    });
                let _ = tx.send(result);
            }
            Command::Remove(tx, range) => {
                let result = self
                    .remove(&range)
                    .await
                    .map_err(crate::Error::IpSplitTunnel)
                    .inspect_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Unable to remove IP split-tunnel range")
                        );
                    });
                let _ = tx.send(result);
            }
            Command::Clear(tx) => {
                let result = self
                    .clear()
                    .await
                    .map_err(crate::Error::IpSplitTunnel)
                    .inspect_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Unable to clear IP split-tunnel ranges")
                        );
                    });
                let _ = tx.send(result);
            }
            Command::Check(tx, tunnel_interface) => {
                let result = self
                    .check(tunnel_interface.as_deref())
                    .await
                    .map_err(crate::Error::IpSplitTunnel)
                    .inspect_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Unable to check IP split-tunnel ranges")
                        );
                    });
                let _ = tx.send(result);
            }
            Command::Toggle(tx) => {
                let new_state = self.toggle().await;
                let _ = tx.send(Ok(new_state));
            }
        }
    }

    async fn check(&self, tunnel_interface: Option<&str>) -> Result<Vec<CheckResult>, Error> {
        let mut checks = Vec::with_capacity(self.ranges.len());

        for range in &self.ranges {
            let sample_ip = sample_ip(*range);
            let mut check = CheckResult {
                range: range.to_string(),
                sample_ip,
                bypasses_tunnel: false,
                interface: None,
                error: None,
            };

            let Some(tunnel_interface) = tunnel_interface else {
                check.error = Some("VPN tunnel interface unavailable; connect Mullvad first".into());
                checks.push(check);
                continue;
            };

            match self
                .route_manager
                .get_destination_route(IpAddr::V4(sample_ip), Some(mullvad_types::TUNNEL_FWMARK))
                .await
                .map_err(Error::RouteManager)?
            {
                Some(route) => {
                    let interface = route.get_node().get_device().map(ToOwned::to_owned);
                    check.bypasses_tunnel = interface.as_deref() != Some(tunnel_interface);
                    check.interface = interface;
                    if check.interface.is_none() {
                        check.error = Some("route has no output interface".into());
                    }
                }
                None => {
                    check.error = Some("no route for marked traffic".into());
                }
            }

            checks.push(check);
        }

        Ok(checks)
    }

    async fn save_and_apply(&mut self) -> Result<(), Error> {
        save_ranges(&self.path, &self.ranges).await?;
        self.apply().await
    }

    async fn clear_applied(&mut self) -> Result<(), Error> {
        talpid_core::firewall::linux::ip_split_tunnel::reset_ranges().map_err(Error::Firewall)
    }
}

pub fn parse_ipv4_range(input: &str) -> Result<Ipv4Network, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidRange(input.to_owned()));
    }

    if trimmed.contains('/') {
        let network = IpNetwork::from_str(trimmed)
            .map_err(|_| Error::InvalidRange(input.to_owned()))?;
        let IpNetwork::V4(network) = network else {
            return Err(Error::NonIpv4Range);
        };
        return canonicalize(network);
    }

    let address =
        Ipv4Addr::from_str(trimmed).map_err(|_| Error::InvalidRange(input.to_owned()))?;
    Ipv4Network::new(address, 32).map_err(|_| Error::InvalidRange(input.to_owned()))
}

fn canonicalize(network: Ipv4Network) -> Result<Ipv4Network, Error> {
    Ipv4Network::new(network.network(), network.prefix())
        .map_err(|_| Error::InvalidRange(network.to_string()))
}

fn sample_ip(range: Ipv4Network) -> Ipv4Addr {
    match range.prefix() {
        0 => Ipv4Addr::new(1, 1, 1, 1),
        32 => range.ip(),
        _ => Ipv4Addr::from(u32::from(range.network()) + 1),
    }
}

async fn load_ranges(path: &Path) -> Result<BTreeSet<Ipv4Network>, Error> {
    let contents = match tokio::fs::read_to_string(path).await {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeSet::new());
        }
        Err(error) => return Err(Error::ReadStore(error)),
    };

    let store: Store = serde_json::from_str(&contents).map_err(Error::ParseStore)?;
    if store.version != STORE_VERSION {
        return Err(Error::UnsupportedStoreVersion(store.version));
    }

    store
        .ipv4_ranges
        .iter()
        .map(|range| parse_ipv4_range(range))
        .collect()
}

async fn save_ranges(path: &Path, ranges: &BTreeSet<Ipv4Network>) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(Error::CreateStoreDir)?;
    }

    let store = Store {
        version: STORE_VERSION,
        ipv4_ranges: ranges.iter().map(ToString::to_string).collect(),
    };
    let contents = serde_json::to_vec_pretty(&store).map_err(Error::ParseStore)?;
    tokio::fs::write(path, contents)
        .await
        .map_err(Error::WriteStore)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_store_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mullvad-ip-split-tunnel-{name}-{}-{nonce}.json",
            std::process::id()
        ))
    }

    #[test]
    fn single_ip_is_normalized_to_host_range() {
        let range = parse_ipv4_range("100.64.0.1").unwrap();
        assert_eq!(range.to_string(), "100.64.0.1/32");
    }

    #[test]
    fn cidr_is_canonicalized() {
        let range = parse_ipv4_range("100.64.12.34/10").unwrap();
        assert_eq!(range.to_string(), "100.64.0.0/10");
    }

    #[test]
    fn netbird_tenant_cidr_is_canonicalized() {
        let range = parse_ipv4_range("100.114.4.17/16").unwrap();
        assert_eq!(range.to_string(), "100.114.0.0/16");
    }

    #[test]
    fn rejects_ipv6() {
        assert!(matches!(
            parse_ipv4_range("fd7a:115c:a1e0::/48"),
            Err(Error::NonIpv4Range)
        ));
    }

    #[test]
    fn rejects_invalid_range() {
        assert!(matches!(
            parse_ipv4_range("not-a-range"),
            Err(Error::InvalidRange(_))
        ));
    }

    #[test]
    fn accepts_default_route() {
        let range = parse_ipv4_range("0.0.0.0/0").unwrap();
        assert_eq!(range.to_string(), "0.0.0.0/0");
    }

    #[test]
    fn samples_default_route_with_public_ipv4() {
        let range = parse_ipv4_range("0.0.0.0/0").unwrap();
        assert_eq!(sample_ip(range).to_string(), "1.1.1.1");
    }

    #[test]
    fn samples_cidr_with_first_host_address() {
        let range = parse_ipv4_range("100.64.0.0/10").unwrap();
        assert_eq!(sample_ip(range).to_string(), "100.64.0.1");
    }

    #[test]
    fn samples_host_route_with_exact_address() {
        let range = parse_ipv4_range("100.64.0.42").unwrap();
        assert_eq!(sample_ip(range).to_string(), "100.64.0.42");
    }

    #[test]
    fn missing_store_loads_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = test_store_path("missing");

        let ranges = runtime.block_on(load_ranges(&path)).unwrap();

        assert!(ranges.is_empty());
    }

    #[test]
    fn malformed_store_returns_error() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = test_store_path("malformed");
        std::fs::write(&path, b"{not-json").unwrap();

        let error = runtime.block_on(load_ranges(&path)).unwrap_err();

        assert!(matches!(error, Error::ParseStore(_)));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn saved_store_deduplicates_and_loads_canonical_ranges() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = test_store_path("roundtrip");
        let ranges = BTreeSet::from([
            parse_ipv4_range("100.64.0.1").unwrap(),
            parse_ipv4_range("100.64.0.1/32").unwrap(),
            parse_ipv4_range("100.64.12.34/10").unwrap(),
        ]);

        runtime.block_on(save_ranges(&path, &ranges)).unwrap();
        let loaded = runtime.block_on(load_ranges(&path)).unwrap();

        assert_eq!(
            loaded.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["100.64.0.0/10", "100.64.0.1/32"]
        );
        let _ = std::fs::remove_file(path);
    }
}
