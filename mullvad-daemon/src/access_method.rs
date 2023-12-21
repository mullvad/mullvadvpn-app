use crate::{
    api,
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_api::rest;
use mullvad_types::{
    access_method::{self, AccessMethod, AccessMethodSetting},
    settings::Settings,
};

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
    /// Access method could not be rotate
    #[error(display = "Access method could not be rotated")]
    RotationError,
    /// Some error occured in the daemon's state of handling
    /// [`AccessMethodSetting`]s & [`ApiConnectionMode`]s.
    #[error(display = "Error occured when handling connection settings & details")]
    ConnectionMode(#[error(source)] api::Error),
    #[error(display = "API endpoint rotation failed")]
    RestError(#[error(source)] rest::Error),
    /// Access methods settings error
    #[error(display = "Settings error")]
    Settings(#[error(source)] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    /// Add a [`AccessMethod`] to the daemon's settings.
    ///
    /// If the daemon settings are successfully updated, the
    /// [`access_method::Id`] of the newly created [`AccessMethodSetting`]
    /// (which has been derived from the [`AccessMethod`]) is returned.
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

    /// Remove a [`AccessMethodSetting`] from the daemon's saved settings.
    ///
    /// If the [`AccessMethodSetting`] which is currently in use happens to be
    /// removed, the daemon should force a rotation of the active API endpoint.
    pub async fn remove_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        match self.settings.api_access_methods.find(&access_method) {
            // Make sure that we are not trying to remove a built-in API access
            // method
            Some(api_access_method) if api_access_method.is_builtin() => {
                return Err(Error::RemoveBuiltIn)
            }
            // If the currently active access method is removed, a new access
            // method should trigger
            Some(api_access_method)
                if api_access_method.get_id()
                    == self.get_current_access_method().await?.get_id() =>
            {
                self.force_api_endpoint_rotation().await?;
            }
            _ => (),
        }

        self.settings
            .update(|settings| settings.api_access_methods.remove(&access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map(|_| ())
            .map_err(Error::Settings)
    }

    /// Set a [`AccessMethodSetting`] as the current API access method.
    ///
    /// If successful, the daemon will force a rotation of the active API access
    /// method, which means that subsequent API calls will use the new
    /// [`AccessMethodSetting`] to figure out the API endpoint.
    pub async fn set_api_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        let access_method = self.get_api_access_method(access_method)?;
        self.connection_modes_handler
            .set_access_method(access_method)
            .await?;
        // Force a rotation of Access Methods.
        self.force_api_endpoint_rotation().await
    }

    pub fn get_api_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<AccessMethodSetting, Error> {
        self.settings
            .api_access_methods
            .find(&access_method)
            .ok_or(Error::NoSuchMethod(access_method))
            .cloned()
    }

    /// "Updates" an [`AccessMethodSetting`] by replacing the existing entry
    /// with the argument `access_method_update` if an existing entry with
    /// matching [`access_method::Id`] is found.
    ///
    /// If the currently active [`AccessMethodSetting`] is updated, the daemon
    /// will automatically use this updated [`AccessMethodSetting`] when
    /// performing subsequent API calls.
    pub async fn update_access_method(
        &mut self,
        access_method_update: AccessMethodSetting,
    ) -> Result<(), Error> {
        // We have to be a bit careful. If we are about to disable the last
        // remaining enabled access method, we would cause an inconsistent state
        // in the daemon's settings. Therefore, we have to safeguard against
        // this by explicitly checking for & disallow any update which would
        // cause the last enabled access method to become disabled.
        let current = self.get_current_access_method().await?;
        // If the currently active access method is updated, we need to re-set it.
        let mut refresh = None;
        let settings_update = |settings: &mut Settings| {
            if let Some(access_method) = settings
                .api_access_methods
                .find_mut(&access_method_update.get_id())
            {
                *access_method = access_method_update;
                if access_method.get_id() == current.get_id() {
                    refresh = Some(access_method.get_id())
                }
            }
        };

        self.settings
            .update(settings_update)
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)?;
        if let Some(id) = refresh {
            self.set_api_access_method(id).await?;
        }
        Ok(())
    }

    /// Return the [`AccessMethodSetting`] which is currently used to access the
    /// Mullvad API.
    pub async fn get_current_access_method(&self) -> Result<AccessMethodSetting, Error> {
        self.connection_modes_handler
            .get_current()
            .await
            .map(|current| current.setting)
            .map_err(Error::ConnectionMode)
    }

    /// Change which [`AccessMethodSetting`] which will be used as the Mullvad
    /// API endpoint.
    async fn force_api_endpoint_rotation(&self) -> Result<(), Error> {
        self.api_handle
            .service()
            .next_api_endpoint()
            .await
            .map_err(|error| {
                log::error!("Failed to rotate API endpoint: {}", error);
                Error::RotationError
            })
    }

    /// If settings were changed due to an update, notify all listeners.
    fn notify_on_change(&mut self, settings_changed: MadeChanges) -> &mut Self {
        if settings_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());

            let handle = self.connection_modes_handler.clone();
            let new_access_methods = self.settings.api_access_methods.collect_enabled();
            tokio::spawn(async move {
                match handle.update_access_methods(new_access_methods).await {
                    Ok(_) => (),
                    Err(api::Error::NoAccessMethods) | Err(_) => {
                        // `access_methods` was empty! This implies that the user
                        // disabled all access methods. If we ever get into this
                        // state, we should default to using the direct access
                        // method.
                        let default = access_method::Settings::direct();
                        handle.update_access_methods(vec![default]).await.expect("Failed to create the data structure responsible for managing access methods");
                    }
                }
            });
        };
        self
    }
}
