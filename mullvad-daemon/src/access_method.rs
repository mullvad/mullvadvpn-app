use crate::{
    api::{self, AccessModeSelectorHandle},
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_api::rest::{self, MullvadRestHandle};
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

/// A tiny datastructure used for signaling whether the daemon should force a
/// rotation of the currently used [`AccessMethodSetting`] or not, and if so:
/// how it should do it.
pub enum Command {
    /// There is no need to force a rotation of [`AccessMethodSetting`]
    Nothing,
    /// Select the [`AccessMethodSetting`] with a certain [`access_method::Id`]
    Set(access_method::Id),
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
                self.connection_modes_handler.next().await?;
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
        //
        // This is not a call to `process_command` due to the restrictions on
        // recursively calling async functions.
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
        let mut command = Command::Nothing;
        let settings_update = |settings: &mut Settings| {
            if let Some(access_method) = settings
                .api_access_methods
                .find_mut(&access_method_update.get_id())
            {
                *access_method = access_method_update;
                if access_method.get_id() == current.get_id() {
                    command = Command::Set(access_method.get_id())
                }
            }
        };

        self.settings
            .update(settings_update)
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)?
            .process_command(command)
            .await
    }

    /// Return the [`AccessMethodSetting`] which is currently used to access the
    /// Mullvad API.
    pub async fn get_current_access_method(&self) -> Result<AccessMethodSetting, Error> {
        Ok(self.connection_modes_handler.get_access_method().await?)
    }

    /// Change which [`AccessMethodSetting`] which will be used as the Mullvad
    /// API endpoint.
    pub async fn force_api_endpoint_rotation(&self) -> Result<(), Error> {
        self.api_handle
            .service()
            .next_api_endpoint()
            .await
            .map_err(|error| {
                log::error!("Failed to rotate API endpoint: {}", error);
                Error::RotationError
            })
    }

    /// Update the known active access method and notify any listeners
    pub async fn set_active_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| settings.api_access_methods.active = Some(access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map(|_| ())
            .map_err(Error::Settings)
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

    /// The semantics of the [`Command`] datastructure.
    async fn process_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::Nothing => Ok(()),
            Command::Set(id) => self.set_api_access_method(id).await,
        }
    }
}

/// Try to reach the Mullvad API using a specific access method, returning
/// an [`Error`] in the case where the test fails to reach the API.
///
/// Ephemerally sets a new access method (associated with `access_method`)
/// to be used for subsequent API calls, before performing an API call and
/// switching back to the previously active access method. The previous
/// access method is *always* reset.
pub async fn test_access_method(
    new_access_method: AccessMethodSetting,
    access_mode_selector: AccessModeSelectorHandle,
    rest_handle: MullvadRestHandle,
) -> Result<bool, Error> {
    // Setup test
    let previous_access_method = access_mode_selector
        .get_access_method()
        .await
        .map_err(Error::ConnectionMode)?;

    let method_under_test = new_access_method.clone();
    access_mode_selector
        .set_access_method(new_access_method)
        .await
        .map_err(Error::ConnectionMode)?;

    // We need to perform a rotation of API endpoint after a set action
    let rotation_handle = rest_handle.clone();
    rotation_handle
        .service()
        .next_api_endpoint()
        .await
        .map_err(|err| {
            log::error!("Failed to rotate API endpoint: {err}");
            Error::RestError(err)
        })?;

    // Set up the reset
    //
    // In case the API call fails, the next API endpoint will
    // automatically be selected, which means that we need to set up
    // with the previous API endpoint beforehand.
    access_mode_selector
        .set_access_method(previous_access_method)
        .await
        .map_err(|err| {
            log::error!(
                "Could not reset to previous access
            method after API reachability test was carried out. This should only
            happen if the previous access method was removed in the meantime."
            );
            Error::ConnectionMode(err)
        })?;

    // Perform test
    //
    // Send a HEAD request to some Mullvad API endpoint. We issue a HEAD
    // request because we are *only* concerned with if we get a reply from
    // the API, and not with the actual data that the endpoint returns.
    let result = mullvad_api::ApiProxy::new(rest_handle)
        .api_addrs_available()
        .await
        .map_err(Error::RestError)?;

    // We need to perform a rotation of API endpoint after a set action
    // Note that this will be done automatically if the API call fails,
    // so it only has to be done if the call succeeded ..
    if result {
        rotation_handle
            .service()
            .next_api_endpoint()
            .await
            .map_err(|err| {
                log::error!("Failed to rotate API endpoint: {err}");
                Error::RestError(err)
            })?;
    }

    log::info!(
        "The result of testing {method:?} is {result}",
        method = method_under_test.access_method,
        result = if result {
            "success".to_string()
        } else {
            "failed".to_string()
        }
    );

    Ok(result)
}
