//! Config definition.
//! TODO: Document struct and link to that documentation

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::VmConfig;
use crate::tests::config::DEFAULT_MULLVAD_HOST;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    pub mullvad_host: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct RuntimeOptions {
    pub display: Display,
    pub keep_changes: bool,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum Display {
    #[default]
    None,
    Local,
    Vnc,
}

impl Config {
    pub fn get_vm(&self, name: &str) -> Option<&VmConfig> {
        self.vms.get(name)
    }

    /// Get the Mullvad host to use.
    ///
    /// Defaults to [`DEFAULT_MULLVAD_HOST`] if the host was not provided in the [`ConfigFile`].
    pub fn get_host(&self) -> String {
        self.mullvad_host.clone().unwrap_or_else(|| {
            log::debug!("No Mullvad host has been set explicitly. Falling back to default host");
            DEFAULT_MULLVAD_HOST.to_owned()
        })
    }
}
