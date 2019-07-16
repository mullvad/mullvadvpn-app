use super::{Error, Result, VersionedSettings};
use crate::{
    custom_tunnel::CustomTunnelEndpoint,
    relay_constraints::{
        BridgeSettings, BridgeState, Constraint, LocationConstraint, OpenVpnConstraints,
        RelaySettings as NewRelaySettings, TunnelProtocol, WireguardConstraints,
    },
    settings::TunnelOptions,
};
use serde::{Deserialize, Serialize};
use std::io::Read;


/// Mullvad daemon settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    account_token: Option<String>,
    relay_settings: RelaySettings,
    bridge_settings: BridgeSettings,
    bridge_state: BridgeState,
    /// If the daemon should allow communication with private (LAN) networks.
    allow_lan: bool,
    /// Extra level of kill switch. When this setting is on, the disconnected state will block
    /// the firewall to not allow any traffic in or out.
    block_when_disconnected: bool,
    /// If the daemon should connect the VPN tunnel directly on start or not.
    auto_connect: bool,
    /// Options that should be applied to tunnels of a specific type regardless of where the relays
    /// might be located.
    tunnel_options: TunnelOptions,
}

pub(super) struct Migration;
impl super::SettingsMigration for Migration {
    fn read(&self, mut reader: &mut dyn Read) -> Result<VersionedSettings> {
        serde_json::from_reader(&mut reader)
            .map(VersionedSettings::V1)
            .map_err(Error::ParseError)
    }
    fn migrate(&self, old: VersionedSettings) -> VersionedSettings {
        match old {
            VersionedSettings::V1(old) => VersionedSettings::V2(crate::settings::Settings {
                account_token: old.account_token,
                relay_settings: migrate_relay_settings(old.relay_settings),
                bridge_settings: old.bridge_settings,
                bridge_state: old.bridge_state,
                allow_lan: old.allow_lan,
                block_when_disconnected: old.block_when_disconnected,
                auto_connect: old.auto_connect,
                tunnel_options: old.tunnel_options,
                settings_version: super::SettingsVersion::V2,
            }),
            VersionedSettings::V2(new) => VersionedSettings::V2(new),
        }
    }
}

fn migrate_relay_settings(relay_settings: RelaySettings) -> NewRelaySettings {
    match relay_settings {
        RelaySettings::CustomTunnelEndpoint(endpoint) => {
            crate::relay_constraints::RelaySettings::CustomTunnelEndpoint(endpoint)
        }
        RelaySettings::Normal(old_constraints) => {
            let mut new_constraints = crate::relay_constraints::RelayConstraints {
                location: old_constraints.location,
                ..Default::default()
            };
            match old_constraints.tunnel {
                Constraint::Any => (),
                Constraint::Only(TunnelConstraints::OpenVpn(constraints)) => {
                    new_constraints.openvpn_constraints = constraints;
                }
                Constraint::Only(TunnelConstraints::Wireguard(constraints)) => {
                    new_constraints.wireguard_constraints = constraints;
                    new_constraints.tunnel_protocol = Constraint::Only(TunnelProtocol::Wireguard);
                }
            };
            crate::relay_constraints::RelaySettings::Normal(new_constraints)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaySettings {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraints),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    pub tunnel: Constraint<TunnelConstraints>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnConstraints),
    #[serde(rename = "wireguard")]
    Wireguard(WireguardConstraints),
}
