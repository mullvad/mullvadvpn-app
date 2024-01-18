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
    RotationFailed,
    /// Some error occured in the daemon's state of handling
    /// [`AccessMethodSetting`]s & [`ApiConnectionMode`]s.
    #[error(display = "Error occured when handling connection settings & details")]
    ConnectionMode(#[error(source)] api::Error),
    #[error(display = "API endpoint rotation failed")]
    Rest(#[error(source)] rest::Error),
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
        match self.settings.api_access_methods.find_by_id(&access_method) {
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

    /// Select an [`AccessMethodSetting`] as the current API access method.
    ///
    /// If successful, the daemon will force a rotation of the active API access
    /// method, which means that subsequent API calls will use the new
    /// [`AccessMethodSetting`] as to reach the API endpoint.
    ///
    /// # Note
    ///
    /// If the selected [`AccessMethodSetting`] is disabled, it will be enabled
    /// and the Daemon's settings will be updated accordingly. If an
    /// [`AccessMethodSetting`] is enabled, it is eligible to be part of the
    /// automatic selection of access methods that the Daemon will perform at
    /// start up or if the current access method starts failing.
    pub async fn select_api_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        let mut access_method = self.get_api_access_method(access_method)?;
        // Toggle the enabled status if needed
        if !access_method.enabled() {
            access_method.enable();
            self.update_access_method_inner(&access_method).await?
        }
        // Set `access_method` as the next access method to use
        self.connection_modes_handler
            .set_access_method(access_method)
            .await?;
        // Force a rotation of Access Methods
        self.force_api_endpoint_rotation().await
    }

    pub fn get_api_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<AccessMethodSetting, Error> {
        self.settings
            .api_access_methods
            .find_by_id(&access_method)
            .ok_or(Error::NoSuchMethod(access_method))
            .cloned()
    }

    /// Updates a [`AccessMethodSetting`] by replacing the existing entry with
    /// the argument `access_method_update`.  if an entry with a matching
    /// [`access_method::Id`] is found.
    ///
    /// If the currently active [`AccessMethodSetting`] is updated, the daemon
    /// will automatically use this updated [`AccessMethodSetting`] when
    /// performing subsequent API calls.
    pub async fn update_access_method(
        &mut self,
        access_method_update: AccessMethodSetting,
    ) -> Result<(), Error> {
        self.update_access_method_inner(&access_method_update)
            .await?;

        // If the currently active access method is updated, we need to re-set
        // it after updating the settings.
        if access_method_update.get_id() == self.get_current_access_method().await?.get_id() {
            self.select_api_access_method(access_method_update.get_id())
                .await?;
        }

        Ok(())
    }

    /// Updates a [`AccessMethodSetting`] by replacing the existing entry with
    /// the argument `access_method_update`.  if an entry with a matching
    /// [`access_method::Id`] is found.
    ///
    /// This inner function does not perform any kind of check to see if the
    /// existing, in-use setting needs to be re-set.
    async fn update_access_method_inner(
        &mut self,
        access_method_update: &AccessMethodSetting,
    ) -> Result<(), Error> {
        let access_method_update_moved = access_method_update.clone();
        let settings_update = |settings: &mut Settings| {
            if let Some(access_method) = settings
                .api_access_methods
                .find_by_id_mut(&access_method_update_moved.get_id())
            {
                *access_method = access_method_update_moved;
                // We have to be a bit careful. If the update is about to
                // disable the last remaining enabled access method, we would
                // cause an inconsistent state in the daemon's settings.
                // Therefore, we have to explicitly safeguard against this by.
                // In that case, we should re-enable the `Direct` access method.
                if settings.api_access_methods.collect_enabled().is_empty() {
                    if let Some(direct) = settings.api_access_methods.get_direct() {
                        direct.enabled = true;
                    } else {
                        // If the `Direct` access method does not exist within the
                        // settings for some reason, the settings are in an
                        // inconsistent state. We don't have much choice but to
                        // reset these settings to their default value.
                        log::warn!("The built-in access methods can not be found. This might be due to a corrupt settings file");
                        settings.api_access_methods = access_method::Settings::default();
                    }
                }
            }
        };

        self.settings
            .update(settings_update)
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)?;

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
                Error::RotationFailed
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
