use crate::{api, settings, Daemon, EventListener};
use mullvad_api::{proxy::ApiConnectionMode, rest, ApiProxy};
use mullvad_types::{
    access_method::{self, AccessMethod, AccessMethodSetting},
    settings::Settings,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Can not add access method
    #[error("Cannot add custom access method")]
    Add,
    /// Can not find access method
    #[error("Cannot find custom access method {0}")]
    NoSuchMethod(access_method::Id),
    /// Some error occured in the daemon's state of handling
    /// [`AccessMethodSetting`]s & [`ApiConnectionMode`]s
    #[error("Error occured when handling connection settings & details")]
    ApiService(#[from] api::Error),
    /// A REST request failed
    #[error("Reset request failed")]
    Rest(#[from] rest::Error),
    /// Something went wrong in the [`access_method`](mod@access_method) module.
    #[error("Access method error")]
    AccessMethod(#[from] access_method::Error),
    /// Access methods settings error
    #[error("Settings error")]
    Settings(#[from] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener,
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
            .await?;
        Ok(id)
    }

    /// Remove a [`AccessMethodSetting`] from the daemon's saved settings.
    pub async fn remove_access_method(
        &mut self,
        access_method: access_method::Id,
    ) -> Result<(), Error> {
        self.settings
            .try_update(|settings| -> Result<(), Error> {
                settings.api_access_methods.remove(&access_method)?;
                Ok(())
            })
            .await?;
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
        self.settings
            .update(|settings| {
                settings.api_access_methods.update(
                    |setting| setting.get_id() == access_method,
                    |setting| setting.enable(),
                );
            })
            .await?;
        self.access_mode_handler
            .use_access_method(access_method)
            .await?;
        Ok(())
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
    pub async fn update_access_method(
        &mut self,
        access_method_update: AccessMethodSetting,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings: &mut Settings| {
                let target = access_method_update.get_id();
                settings.api_access_methods.update(
                    |access_method| access_method.get_id() == target,
                    |method| *method = access_method_update,
                );
            })
            .await?;

        Ok(())
    }

    /// Remove all custom [`AccessMethodSetting`].
    pub async fn clear_custom_api_access_methods(&mut self) -> Result<(), Error> {
        self.settings
            .update(|settings: &mut Settings| {
                settings.api_access_methods.clear_custom();
            })
            .await?;

        Ok(())
    }

    /// Return the [`AccessMethodSetting`] which is currently used to access the
    /// Mullvad API.
    pub async fn get_current_access_method(&self) -> Result<AccessMethodSetting, Error> {
        let current = self.access_mode_handler.get_current().await?;
        Ok(current.setting)
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
    /// specific endpoint `connection_mode`.
    pub fn create_limited_api_proxy(&mut self, connection_mode: ApiConnectionMode) -> ApiProxy {
        let rest_handle = self
            .api_runtime
            .mullvad_rest_handle(connection_mode.into_provider());
        ApiProxy::new(rest_handle)
    }

    /// Perform some REST request against the Mullvad API.
    ///
    /// * Returns `Ok(true)` if the API returned the expected result
    /// * Returns `Ok(false)` if the API returned an unexpected result
    /// * Returns `Err(..)` if the API could not be reached
    async fn perform_api_request(api_proxy: ApiProxy) -> Result<bool, Error> {
        Ok(api_proxy.api_addrs_available().await?)
    }
}
