use crate::{
    api,
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
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
    /// Can not find *any* access method. This should never happen. If it does,
    /// the user should do a factory reset.
    #[error(display = "No access methods are configured")]
    NoMethodsExist,
    /// Access method could not be rotate
    #[error(display = "Access method could not be rotated")]
    RotationError,
    /// Daemon API error
    #[error(display = "Daemon API handling error")]
    Api(#[error(source)] api::Error),
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
    /// Select the next available [`AccessMethodSetting`], whichever that is
    Rotate,
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
        // Make sure that we are not trying to remove a built-in API access
        // method
        let command = match self.settings.api_access_methods.find(&access_method) {
            Some(api_access_method) => {
                if api_access_method.is_builtin() {
                    Err(Error::RemoveBuiltIn)
                } else if api_access_method.get_id()
                    == self.get_current_access_method().await?.get_id()
                {
                    Ok(Command::Rotate)
                } else {
                    Ok(Command::Nothing)
                }
            }
            None => Ok(Command::Nothing),
        }?;

        self.settings
            .update(|settings| settings.api_access_methods.remove(&access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)?
            .process_command(command)
            .await
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

    /// Change which [`AccessMethodSetting`] which will be used to figure out
    /// the Mullvad API endpoint.
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
            let new_access_methods = self
                .settings
                .api_access_methods
                .access_method_settings
                .iter()
                .filter(|api_access_method| api_access_method.enabled())
                .cloned()
                .collect();
            tokio::spawn(async move {
                let _ = handle.update_access_methods(new_access_methods).await;
            });
        };
        self
    }

    /// The semantics of the [`Command`] datastructure.
    async fn process_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::Nothing => Ok(()),
            Command::Rotate => self.force_api_endpoint_rotation().await,
            Command::Set(id) => self.set_api_access_method(id).await,
        }
    }
}
