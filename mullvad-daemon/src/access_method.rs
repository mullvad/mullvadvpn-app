use crate::{
    settings::{self, MadeChanges},
    Daemon, EventListener,
};
use mullvad_types::access_method::{
    daemon::{ApiAccessMethodReplace, ApiAccessMethodToggle},
    AccessMethod, CustomAccessMethod,
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Can not add access method
    #[error(display = "Cannot add custom access method")]
    Add,
    /// Access methods settings error
    #[error(display = "Settings error")]
    Settings(#[error(source)] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn add_access_method(&mut self, access_method: AccessMethod) -> Result<(), Error> {
        self.settings
            .update(|settings| settings.api_access_methods.append(access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn toggle_api_access_method(
        &mut self,
        access_method_toggle: ApiAccessMethodToggle,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                if let Some(access_method) = settings
                    .api_access_methods
                    .find_mut(&access_method_toggle.access_method)
                {
                    access_method.toggle(access_method_toggle.enable);
                }
            })
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn remove_access_method(
        &mut self,
        access_method: CustomAccessMethod,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| settings.api_access_methods.remove(&access_method))
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn replace_access_method(
        &mut self,
        access_method_replace: ApiAccessMethodReplace,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                let access_methods = &mut settings.api_access_methods;
                access_methods.append(access_method_replace.access_method);
                access_methods.swap_remove(access_method_replace.index);
            })
            .await
            .map(|did_change| self.notify_on_change(did_change))
            .map_err(Error::Settings)
    }

    pub async fn set_api_access_method(
        &mut self,
        access_method: AccessMethod,
    ) -> Result<(), Error> {
        {
            let mut connection_modes = self.connection_modes.lock().unwrap();
            connection_modes.set_access_method(access_method);
        }
        // Force a rotation of Access Methods.
        let _ = self.api_handle.service().next_api_endpoint();
        Ok(())
    }

    /// If settings were changed due to an update, notify all listeners.
    fn notify_on_change(&mut self, settings_changed: MadeChanges) {
        if settings_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());

            let mut connection_modes = self.connection_modes.lock().unwrap();
            connection_modes.update_access_methods(self.settings.api_access_methods.cloned())
        };
    }
}
