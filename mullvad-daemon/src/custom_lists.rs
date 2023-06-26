use crate::{new_selector_config, settings, Daemon, EventListener};
use mullvad_types::{
    custom_list::{CustomList, CustomListLocationUpdate, Id},
    relay_constraints::{
        BridgeSettings, BridgeState, Constraint, LocationConstraint, RelaySettings,
    },
};
use talpid_types::net::TunnelType;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Custom list already exists
    #[error(display = "A list with that name already exists")]
    ListExists,
    /// Custom list does not exist
    #[error(display = "A list with that name does not exist")]
    ListNotFound,
    /// Can not add any to a custom list
    #[error(display = "Can not add or remove 'any' to or from a custom list")]
    CannotAddOrRemoveAny,
    /// Custom list settings error
    #[error(display = "Settings error")]
    Settings(#[error(source)] settings::Error),
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn delete_custom_list(&mut self, name: String) -> Result<(), Error> {
        let custom_list = self.settings.custom_lists.get_custom_list_with_name(&name);
        match &custom_list {
            None => Err(Error::ListNotFound),
            Some(custom_list) => {
                let id = custom_list.id.clone();

                let settings_changed = self
                    .settings
                    .update(|settings| {
                        settings.custom_lists.custom_lists.remove(&id);
                    })
                    .await
                    .map_err(Error::Settings);

                if let Ok(true) = settings_changed {
                    let need_to_reconnect = self.change_should_cause_reconnect(&id);

                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.relay_selector
                        .set_config(new_selector_config(&self.settings, &self.app_version_info));

                    if need_to_reconnect {
                        log::info!(
                            "Initiating tunnel restart because a selected custom list was deleted"
                        );
                        self.reconnect_tunnel();
                    }
                }

                settings_changed.map(|_| ())
            }
        }
    }

    pub async fn create_custom_list(&mut self, name: String) -> Result<(), Error> {
        if self
            .settings
            .custom_lists
            .get_custom_list_with_name(&name)
            .is_some()
        {
            return Err(Error::ListExists);
        }

        let settings_changed = self
            .settings
            .update(|settings| {
                let custom_list = CustomList::new(name);
                assert!(settings
                    .custom_lists
                    .custom_lists
                    .insert(custom_list.id.clone(), custom_list)
                    .is_none());
            })
            .await
            .map_err(Error::Settings);

        if let Ok(true) = settings_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());
            self.relay_selector
                .set_config(new_selector_config(&self.settings, &self.app_version_info));
        }

        settings_changed.map(|_| ())
    }

    pub async fn update_custom_list_location(
        &mut self,
        update: CustomListLocationUpdate,
    ) -> Result<(), Error> {
        match update {
            CustomListLocationUpdate::Add {
                name,
                location: new_location,
            } => {
                if new_location.is_any() {
                    return Err(Error::CannotAddOrRemoveAny);
                }

                if let Some(custom_list) =
                    self.settings.custom_lists.get_custom_list_with_name(&name)
                {
                    let id = custom_list.id.clone();
                    let new_location = new_location.unwrap();

                    let settings_changed = self
                        .settings
                        .update(|settings| {
                            let locations = &mut settings
                                .custom_lists
                                .custom_lists
                                .get_mut(&id)
                                .unwrap()
                                .locations;

                            if !locations.iter().any(|location| new_location == *location) {
                                locations.push(new_location);
                            }
                        })
                        .await
                        .map_err(Error::Settings);

                    if let Ok(true) = settings_changed {
                        let should_reconnect = self.change_should_cause_reconnect(&id);

                        self.event_listener
                            .notify_settings(self.settings.to_settings());
                        self.relay_selector.set_config(new_selector_config(
                            &self.settings,
                            &self.app_version_info,
                        ));

                        if should_reconnect {
                            log::info!(
                                "Initiating tunnel restart because a selected custom list changed"
                            );
                            self.reconnect_tunnel();
                        }
                    }

                    settings_changed.map(|_| ())
                } else {
                    Err(Error::ListNotFound)
                }
            }
            CustomListLocationUpdate::Remove {
                name,
                location: location_to_remove,
            } => {
                if location_to_remove.is_any() {
                    return Err(Error::CannotAddOrRemoveAny);
                }

                if let Some(custom_list) =
                    self.settings.custom_lists.get_custom_list_with_name(&name)
                {
                    let id = custom_list.id.clone();
                    let location_to_remove = location_to_remove.unwrap();

                    let settings_changed = self
                        .settings
                        .update(|settings| {
                            let locations = &mut settings
                                .custom_lists
                                .custom_lists
                                .get_mut(&id)
                                .unwrap()
                                .locations;
                            if let Some(index) = locations
                                .iter()
                                .position(|location| location == &location_to_remove)
                            {
                                locations.remove(index);
                            }
                        })
                        .await
                        .map_err(Error::Settings);

                    if let Ok(true) = settings_changed {
                        let should_reconnect = self.change_should_cause_reconnect(&id);

                        self.event_listener
                            .notify_settings(self.settings.to_settings());
                        self.relay_selector.set_config(new_selector_config(
                            &self.settings,
                            &self.app_version_info,
                        ));

                        if should_reconnect {
                            log::info!(
                                "Initiating tunnel restart because a selected custom list changed"
                            );
                            self.reconnect_tunnel();
                        }
                    }

                    settings_changed.map(|_| ())
                } else {
                    Err(Error::ListNotFound)
                }
            }
        }
    }

    pub async fn rename_custom_list(
        &mut self,
        name: String,
        new_name: String,
    ) -> Result<(), Error> {
        if self
            .settings
            .custom_lists
            .get_custom_list_with_name(&new_name)
            .is_some()
        {
            Err(Error::ListExists)
        } else {
            match self.settings.custom_lists.get_custom_list_with_name(&name) {
                Some(custom_list) => {
                    let id = custom_list.id.clone();

                    let settings_changed = self
                        .settings
                        .update(|settings| {
                            settings
                                .custom_lists
                                .custom_lists
                                .get_mut(&id)
                                .unwrap()
                                .name = new_name;
                        })
                        .await;

                    if let Ok(true) = settings_changed {
                        self.event_listener
                            .notify_settings(self.settings.to_settings());
                        self.relay_selector.set_config(new_selector_config(
                            &self.settings,
                            &self.app_version_info,
                        ));
                    }

                    Ok(())
                }
                None => Err(Error::ListNotFound),
            }
        }
    }

    fn change_should_cause_reconnect(&self, custom_list_id: &Id) -> bool {
        use mullvad_types::states::TunnelState;
        let mut need_to_reconnect = false;

        if let RelaySettings::Normal(relay_settings) = &self.settings.relay_settings {
            if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
                &relay_settings.location
            {
                need_to_reconnect |= list_id == custom_list_id;
            }

            if let TunnelState::Connecting { endpoint, location: _ } | TunnelState::Connected { endpoint, location: _ }  = &self.tunnel_state {
                match endpoint.tunnel_type {
                    TunnelType::Wireguard => {
                        if relay_settings.wireguard_constraints.use_multihop {
                            if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
                                &relay_settings.wireguard_constraints.entry_location
                            {
                                need_to_reconnect |= list_id == custom_list_id;
                            }
                        }
                    }

                    TunnelType::OpenVpn => {
                        if !matches!(self.settings.bridge_state, BridgeState::Off) {
                            if let BridgeSettings::Normal(bridge_settings) =
                                &self.settings.bridge_settings
                            {
                                if let Constraint::Only(LocationConstraint::CustomList {
                                    list_id,
                                }) = &bridge_settings.location
                                {
                                    need_to_reconnect |= list_id == custom_list_id;
                                }
                            }
                        }
                    }
                }
            }
        }

        need_to_reconnect
    }
}
