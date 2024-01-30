use crate::{
    api,
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_api::{proxy::ApiConnectionMode, rest, ApiProxy};
use mullvad_types::{
    access_method::{self, AccessMethod, AccessMethodSetting},
    settings::Settings,
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Can not add access method
    #[error(display = "Cannot add custom access method")]
    Add,
    /// Can not find access method
    #[error(display = "Cannot find custom access method {}", _0)]
    NoSuchMethod(access_method::Id),
    /// Access method could not be rotate
    #[error(display = "Access method could not be rotated")]
    RotationFailed,
    /// Some error occured in the daemon's state of handling
    /// [`AccessMethodSetting`]s & [`ApiConnectionMode`]s
    #[error(display = "Error occured when handling connection settings & details")]
    ApiService(#[error(source)] api::Error),
    /// A REST request failed
    #[error(display = "Reset request failed")]
    Rest(#[error(source)] rest::Error),
    /// Something went wrong in the [`access_method`](mod@access_method) module.
    #[error(display = "Access method error")]
    AccessMethod(#[error(source)] access_method::Error),
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
        let did_change = self
            .settings
            .try_update(|settings| -> Result<(), Error> {
                settings.api_access_methods.remove(&access_method)?;
                Ok(())
            })
            .await
            .map_err(Error::Settings)?;

        self.notify_on_change(did_change);
        // If the currently active access method is removed, a new access
        // method should be selected.
        //
        // Notice the ordering here: It is important that the current method is
        // removed before we pick a new access method. The `remove` function
        // will ensure that atleast one access method is enabled after the
        // removal. If the currently active access method is removed, some other
        // method is enabled before we pick the next access method to use.
        if self.is_in_use(access_method.clone()).await? {
            self.force_api_endpoint_rotation().await?;
        }

        Ok(())
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
    pub async fn use_api_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        let mut access_method = self.get_api_access_method(access_method)?;
        // Toggle the enabled status if needed
        if !access_method.enabled() {
            access_method.enable();
            self.update_access_method_inner(access_method.clone())
                .await?
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
            .iter()
            .find(|setting| setting.get_id() == access_method)
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
        self.update_access_method_inner(access_method_update.clone())
            .await?;

        if self.is_in_use(access_method_update.get_id()).await? {
            if access_method_update.disabled() {
                // If the currently active access method is updated & disabled
                // we should select the next access method
                self.force_api_endpoint_rotation().await?;
            } else {
                // If the currently active access method is just updated, we
                // need to re-set it after updating the settings
                self.use_api_access_method(access_method_update.get_id())
                    .await?;
            }
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
        access_method_update: AccessMethodSetting,
    ) -> Result<(), Error> {
        let settings_update = |settings: &mut Settings| {
            let target = access_method_update.get_id();
            settings.api_access_methods.update(
                |access_method| access_method.get_id() == target,
                |_| access_method_update,
            );
        };

        self.settings
            .update(settings_update)
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)?;

        Ok(())
    }

    /// Check if some access method is the same as the currently active one.
    ///
    /// This can be useful for invalidating stale states.
    async fn is_in_use(&self, access_method: access_method::Id) -> Result<bool, Error> {
        Ok(access_method == self.get_current_access_method().await?.get_id())
    }

    /// Return the [`AccessMethodSetting`] which is currently used to access the
    /// Mullvad API.
    pub async fn get_current_access_method(&self) -> Result<AccessMethodSetting, Error> {
        self.connection_modes_handler
            .get_current()
            .await
            .map(|current| current.setting)
            .map_err(Error::ApiService)
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

    /// Test if the API is reachable via `proxy`.
    ///
    /// This function tests if [`AccessMethod`] can be used to reach the API.
    /// Its parameters are as low-level as possible to promot re-use between
    /// different kinds of testing contexts, such as testing
    /// [`AccessMethodSetting`]s or on the fly testing of
    /// [`talpid_types::net::proxy::CustomProxy`]s.
    pub(crate) async fn test_access_method(
        proxy: talpid_types::net::AllowedEndpoint,
        access_method_selector: api::AccessModeSelectorHandle,
        daemon_event_sender: crate::DaemonEventSender<(
            api::AccessMethodEvent,
            futures::channel::oneshot::Sender<()>,
        )>,
        api_proxy: ApiProxy,
    ) -> Result<bool, Error> {
        let reset = access_method_selector
            .get_current()
            .await
            .map(|connection_mode| connection_mode.endpoint)?;

        api::AccessMethodEvent::Allow { endpoint: proxy }
            .send(daemon_event_sender.clone())
            .await?;

        let result = Self::perform_api_request(api_proxy).await;

        api::AccessMethodEvent::Allow { endpoint: reset }
            .send(daemon_event_sender)
            .await?;

        result
    }

    /// Create an [`ApiProxy`] which will perform all REST requests against one
    /// specific endpoint `proxy_provider`.
    pub async fn create_limited_api_proxy(
        &mut self,
        proxy_provider: ApiConnectionMode,
    ) -> ApiProxy {
        let rest_handle = self
            .api_runtime
            .mullvad_rest_handle(proxy_provider.into_repeat())
            .await;
        ApiProxy::new(rest_handle)
    }

    /// Perform some REST request against the Mullvad API.
    ///
    /// * Returns `Ok(true)` if the API returned the expected result
    /// * Returns `Ok(false)` if the API returned an unexpected result
    /// * Returns `Err(..)` if the API could not be reached
    async fn perform_api_request(api_proxy: ApiProxy) -> Result<bool, Error> {
        api_proxy.api_addrs_available().await.map_err(Error::Rest)
    }

    /// If settings were changed due to an update, notify all listeners.
    fn notify_on_change(&mut self, settings_changed: MadeChanges) -> &mut Self {
        if settings_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());

            let handle = self.connection_modes_handler.clone();
            let new_access_methods = self.settings.api_access_methods.clone();
            tokio::spawn(async move {
                let _ = handle.update_access_methods(new_access_methods).await;
            });
        };
        self
    }
}
