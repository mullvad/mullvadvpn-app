use crate::{new_selector_config, settings, Daemon, EventListener};
use mullvad_types::api_access_method::{daemon::ApiAccessMethodReplace, AccessMethod};

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
            .update(|settings| {
                settings
                    .api_access_methods
                    .api_access_methods
                    .push(access_method);
            })
            .await
            .map(|changed| {
                if changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.relay_selector
                        .set_config(new_selector_config(&self.settings));
                };
            })
            .map_err(Error::Settings)
    }

    pub async fn remove_access_method(&mut self, access_method: AccessMethod) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                settings
                    .api_access_methods
                    .api_access_methods
                    .retain(|x| *x != access_method);
            })
            .await
            .map(|changed| {
                if changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.relay_selector
                        .set_config(new_selector_config(&self.settings));
                };
            })
            .map_err(Error::Settings)
    }

    pub async fn replace_access_method(
        &mut self,
        access_method_replace: ApiAccessMethodReplace,
    ) -> Result<(), Error> {
        self.settings
            .update(|settings| {
                let access_methods = &mut settings.api_access_methods.api_access_methods;
                access_methods.push(access_method_replace.access_method);
                access_methods.swap_remove(access_method_replace.index);
            })
            .await
            .map(|changed| {
                if changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.relay_selector
                        .set_config(new_selector_config(&self.settings));
                };
            })
            .map_err(Error::Settings)
    }
}
