//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

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
    StateTransition(states::TunnelState),

    /// The daemon settings changed.
    Settings(settings::Settings),

    /// The daemon got an updated relay list.
    RelayList(relay_list::RelayList),

    /// Key event
    WireguardKey(wireguard::KeygenEvent),
}
