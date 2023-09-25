use crate::{
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_management_interface::types::rpc::api_access_method_update::ApiAccessMethodUpdate;
use mullvad_types::access_method::{ApiAccessMethod, ApiAccessMethodId};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Can not add access method
    #[error(display = "Cannot add custom access method")]
    Add,
    /// Can not find access method
    #[error(display = "Cannot find custom access method {}", _0)]
    NoSuchMethod(ApiAccessMethodId),
    /// Access methods settings error
    #[error(display = "Settings error")]
    Settings(#[error(source)] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn add_access_method(&mut self, access_method: ApiAccessMethod) -> Result<(), Error> {
        self.settings
            .update(|settings| settings.api_access_methods.append(access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn remove_access_method(
        &mut self,
        access_method: ApiAccessMethodId,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| settings.api_access_methods.remove(&access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn update_access_method(
        &mut self,
        access_method_update: ApiAccessMethodUpdate,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                let access_methods = &mut settings.api_access_methods;
                if let Some(access_method) =
                    // TODO: This will not work, has to be based on ID!
                    access_methods.find_mut(&access_method_update.id)
                {
                    *access_method = access_method_update.access_method
                }
            })
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub fn set_api_access_method(&mut self, access_method: ApiAccessMethodId) -> Result<(), Error> {
        if let Some(access_method) = self.settings.api_access_methods.find(&access_method) {
            {
                let mut connection_modes = self.connection_modes.lock().unwrap();
                connection_modes.set_access_method(access_method.access_method.clone());
            }
            // Force a rotation of Access Methods.
            let _ = self.api_handle.service().next_api_endpoint();
            Ok(())
        } else {
            Err(Error::NoSuchMethod(access_method))
        }
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
                    .api_access_methods
                    .iter()
                    .map(|x| x.access_method.clone())
                    .collect(),
            )
        };
    }
}
