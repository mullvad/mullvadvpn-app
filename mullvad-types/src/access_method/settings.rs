//! Settings for API access methods.

use serde::{Deserialize, Serialize};

use crate::access_method::{AccessMethod, BuiltInAccessMethod, Id, protobuf::AccessMethodSetting};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Access method with name already exists")]
    DuplicateName,
    /// Built-in access methods can not be removed
    #[error("Cannot remove built-in access method {}", attempted)]
    RemoveBuiltin { attempted: BuiltInAccessMethod },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default = "Settings::create_direct")]
    direct: AccessMethodSetting,
    #[serde(default = "Settings::create_mullvad_bridges")]
    mullvad_bridges: AccessMethodSetting,
    #[serde(default = "Settings::create_encrypted_dns_proxy")]
    encrypted_dns_proxy: AccessMethodSetting,
    #[serde(default = "Settings::create_domain_fronting")]
    domain_fronting: AccessMethodSetting,
    /// Custom API access methods.
    custom: Vec<AccessMethodSetting>,
}

impl Settings {
    pub fn new(
        direct: AccessMethodSetting,
        mullvad_bridges: AccessMethodSetting,
        encrypted_dns_proxy: AccessMethodSetting,
        domain_fronting: AccessMethodSetting,
        custom: Vec<AccessMethodSetting>,
    ) -> Settings {
        Settings {
            direct,
            mullvad_bridges,
            encrypted_dns_proxy,
            domain_fronting,
            custom,
        }
    }

    /// Append an [`AccessMethod`] to the end of `api_access_methods`.
    pub fn append(&mut self, api_access_method: AccessMethodSetting) -> Result<(), Error> {
        self.check_custom_access_method_name_is_unique(&api_access_method)?;
        self.custom.push(api_access_method);
        Ok(())
    }

    /// Remove an [`AccessMethod`] from `api_access_methods`.
    ///
    /// This function will return an error if a built-in API access is about to
    /// be removed.
    pub fn remove(&mut self, api_access_method: &Id) -> Result<(), Error> {
        let Some(setting) = self
            .custom
            .iter()
            .find(|setting| setting.get_id() == *api_access_method)
        else {
            return Ok(());
        };

        match setting.access_method {
            AccessMethod::BuiltIn(ref built_in) => Err(Error::RemoveBuiltin {
                attempted: built_in.clone(),
            }),
            AccessMethod::Custom(_) => {
                self.custom
                    .retain(|method| method.get_id() != *api_access_method);
                self.ensure_consistent_state();
                Ok(())
            }
        }
    }

    /// Update an existing [`AccessMethodSetting`] chosen by `predicate`, in a
    /// closure `f`, saving the result to `self`.
    ///
    /// Returns a bool to indicate whether some [`AccessMethodSetting`] was
    /// updated.
    pub fn update(
        &mut self,
        predicate: impl Fn(&AccessMethodSetting) -> bool,
        f: impl FnOnce(&mut AccessMethodSetting),
    ) -> Result<bool, Error> {
        let mut updated = false;

        let handle = self.clone();
        let update_check = |new_access_method| {
            handle.check_custom_access_method_name_is_unique(new_access_method)?;
            Ok(())
        };

        if let Some(access_method) = self.iter_mut().find(|setting| predicate(setting)) {
            let mut new_access_method = access_method.clone();
            f(&mut new_access_method);
            update_check(&new_access_method)?;
            *access_method = new_access_method;

            updated = true;
        }
        self.ensure_consistent_state();

        Ok(updated)
    }

    /// Update an existing builtin [`AccessMethodSetting`] chosen by
    /// `predicate`, in a closure `f`, saving the result to `self`.
    ///
    /// Returns a bool to indicate whether some [`AccessMethodSetting`] was
    /// updated.
    pub fn update_builtin(
        &mut self,
        predicate: impl Fn(&AccessMethodSetting) -> bool,
        f: impl FnOnce(&mut AccessMethodSetting),
    ) -> bool {
        let mut updated = false;

        if let Some(access_method) = self
            .iter_mut()
            .find(|setting| setting.is_builtin() && predicate(setting))
        {
            f(access_method);

            updated = true;
        }

        updated
    }

    /// Remove all custom access methods.
    pub fn clear_custom(&mut self) {
        self.custom.clear();
        self.ensure_consistent_state();
    }

    /// Check that `self` contains atleast one enabled access methods. If not,
    /// the `Direct` access method is re-enabled.
    fn ensure_consistent_state(&mut self) {
        if self.iter().all(AccessMethodSetting::disabled) {
            self.direct.enable();
        }
    }

    /// This function will return an error if a custom access method with
    /// the same name already exists.
    fn check_custom_access_method_name_is_unique(
        &self,
        new_api_access_method: &AccessMethodSetting,
    ) -> Result<(), Error> {
        if self.custom.iter().any(|api_access_method| {
            api_access_method.get_id() != new_api_access_method.get_id()
                && api_access_method.name == new_api_access_method.name
        }) {
            return Err(Error::DuplicateName);
        }
        Ok(())
    }

    /// Iterate over references of built-in & custom access methods.
    pub fn iter(&self) -> impl Iterator<Item = &AccessMethodSetting> + Clone {
        use std::iter::once;
        once(&self.direct)
            .chain(once(&self.mullvad_bridges))
            .chain(once(&self.encrypted_dns_proxy))
            .chain(once(&self.domain_fronting))
            .chain(&self.custom)
    }

    /// Iterate over mutable references of built-in & custom access methods.
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut AccessMethodSetting> {
        use std::iter::once;
        once(&mut self.direct)
            .chain(once(&mut self.mullvad_bridges))
            .chain(once(&mut self.encrypted_dns_proxy))
            .chain(once(&mut self.domain_fronting))
            .chain(&mut self.custom)
    }

    /// Iterate over references of custom access methods.
    pub fn iter_custom(&self) -> impl Iterator<Item = &AccessMethodSetting> {
        self.custom.iter()
    }

    /// Return the total number of access methods.
    /// This counts both enabled and disabled [`AccessMethodSetting`]s.
    pub fn cardinality(&self) -> usize {
        self.iter().count()
    }

    pub fn direct(&self) -> &AccessMethodSetting {
        &self.direct
    }

    pub fn mullvad_bridges(&self) -> &AccessMethodSetting {
        &self.mullvad_bridges
    }

    pub fn encrypted_dns_proxy(&self) -> &AccessMethodSetting {
        &self.encrypted_dns_proxy
    }

    pub fn domain_fronting(&self) -> &AccessMethodSetting {
        &self.domain_fronting
    }

    fn create_direct() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Direct;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    fn create_mullvad_bridges() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Bridge;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    fn create_encrypted_dns_proxy() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::EncryptedDnsProxy;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    fn create_domain_fronting() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::DomainFronting;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            direct: Settings::create_direct(),
            mullvad_bridges: Settings::create_mullvad_bridges(),
            encrypted_dns_proxy: Settings::create_encrypted_dns_proxy(),
            domain_fronting: Settings::create_domain_fronting(),
            custom: vec![],
        }
    }
}
