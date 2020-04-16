#![deny(rust_2018_idioms)]

pub mod account;
pub mod auth_failed;
pub mod endpoint;
pub mod location;
pub mod relay_constraints;
pub mod relay_list;
pub mod settings;
pub mod states;
pub mod version;
pub mod wireguard;

mod custom_tunnel;
pub use crate::custom_tunnel::*;

/// An event sent out from the daemon to frontends.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonEvent {
    /// The daemon transitioned into a new state.
    TunnelState(states::TunnelState),

    /// The daemon settings changed.
    Settings(settings::Settings),

    /// The daemon got an updated relay list.
    RelayList(relay_list::RelayList),

    /// The daemon got update version info.
    AppVersionInfo(version::AppVersionInfo),

    /// Key event
    WireguardKey(wireguard::KeygenEvent),
}
