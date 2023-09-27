use crate::{
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_types::access_method::{self, AccessMethod, AccessMethodSetting};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Can not add access method
    #[error(display = "Cannot add custom access method")]
    Add,
    /// Can not remove built-in access method
    #[error(display = "Cannot remove built-in access method")]
    RemoveBuiltIn,
    /// Can not find access method
    #[error(display = "Cannot find custom access method {}", _0)]
    NoSuchMethod(access_method::Id),
    /// Can not find *any* access method. This should never happen. If it does,
    /// the user should do a factory reset.
    #[error(display = "No access methods are configured")]
    NoMethodsExist,
    /// Access methods settings error
    #[error(display = "Settings error")]
    Settings(#[error(source)] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn add_access_method(
        &mut self,
        name: String,
        enabled: bool,
        access_method: AccessMethod,
    ) -> Result<access_method::Id, Error> {
        let access_method_setting = AccessMethodSetting::new(name, enabled, access_method);
        let id = access_method_setting.get_id();
        self.settings
            .update(|settings| settings.api_access_methods.append(access_method_setting))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map(|_| id)
            .map_err(Error::Settings)
    }

    pub async fn remove_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        // Make sure that we are not trying to remove a built-in API access
        // method
        match self.settings.api_access_methods.find(&access_method) {
            None => return Ok(()),
            Some(api_access_method) => {
                if api_access_method.is_builtin() {
                    return Err(Error::RemoveBuiltIn);
                }
            }
        };

        self.settings
            .update(|settings| settings.api_access_methods.remove(&access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub fn set_api_access_method(&mut self, access_method: access_method::Id) -> Result<(), Error> {
        if let Some(access_method) = self.settings.api_access_methods.find(&access_method) {
            {
                let mut connection_modes = self.connection_modes.lock().unwrap();
                connection_modes.set_access_method(access_method.clone());
            }
            // Force a rotation of Access Methods.
            let _ = self.api_handle.service().next_api_endpoint();
            Ok(())
        } else {
            Err(Error::NoSuchMethod(access_method))
        }
    }

    /// "Updates" an [`AccessMethodSetting`] by replacing the existing entry
    /// with the argument `access_method_update` if an existing entry with
    /// matching UUID is found.
    pub async fn update_access_method(
        &mut self,
        access_method_update: AccessMethodSetting,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                let access_methods = &mut settings.api_access_methods;
                if let Some(access_method) = access_methods.find_mut(&access_method_update.get_id())
                {
                    *access_method = access_method_update
                }
            })
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    /// Return the [`AccessMethodSetting`] which is currently used to access the
    /// Mullvad API.
    pub fn get_current_access_method(&mut self) -> Result<AccessMethodSetting, Error> {
        let connections_modes = self.connection_modes.lock().unwrap();
        Ok(connections_modes.peek())
    }

    /// If settings were changed due to an update, notify all listeners.
    fn notify_on_change(&mut self, settings_changed: MadeChanges) {
        if settings_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());

            let mut connection_modes = self.connection_modes.lock().unwrap();
            connection_modes.update_access_methods(
                self.settings
                    .api_access_methods
                    .access_method_settings
                    .iter()
                    .filter(|api_access_method| api_access_method.enabled())
                    .cloned()
                    .collect(),
            )
        };
    }
}
