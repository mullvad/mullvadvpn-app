//! API Access Method datastructure
//!
//! Mirrors the protobuf definition

use serde::{Deserialize, Serialize};
use talpid_types::net::proxy::CustomProxy;

use crate::access_method::{AccessMethod, BuiltInAccessMethod, Id};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccessMethodSetting {
    /// Some unique id (distinct for each `AccessMethod`).
    id: Id,
    pub name: String,
    pub enabled: bool,
    pub access_method: AccessMethod,
}

impl AccessMethodSetting {
    pub fn new(name: String, enabled: bool, access_method: AccessMethod) -> Self {
        Self {
            id: Id::new(),
            name,
            enabled,
            access_method,
        }
    }

    /// Just like [`new`], [`with_id`] will create a new [`AccessMethodSetting`].
    /// But instead of automatically generating a new UUID, the id is instead
    /// passed as an argument.
    ///
    /// This is useful when converting to [`AccessMethodSetting`] from other data
    /// representations, such as protobuf.
    ///
    /// [`new`]: AccessMethodSetting::new
    /// [`with_id`]: AccessMethodSetting::with_id
    pub fn with_id(id: Id, name: String, enabled: bool, access_method: AccessMethod) -> Self {
        Self {
            id,
            name,
            enabled,
            access_method,
        }
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn disabled(&self) -> bool {
        !self.enabled
    }

    pub fn as_custom(&self) -> Option<&CustomProxy> {
        self.access_method.as_custom()
    }

    pub fn is_builtin(&self) -> bool {
        self.as_custom().is_none()
    }

    pub fn is_direct(&self) -> bool {
        matches!(
            self.access_method,
            AccessMethod::BuiltIn(BuiltInAccessMethod::Direct)
        )
    }

    /// Set an API access method to be enabled.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Set an API access method to be disabled.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}
