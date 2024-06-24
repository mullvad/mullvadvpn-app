use crate::{new_selector_config, Daemon, Error, EventListener};
use mullvad_types::{
    constraints::Constraint,
    custom_list::{CustomList, Id},
    relay_constraints::{BridgeState, LocationConstraint, RelaySettings, ResolvedBridgeSettings},
};
use talpid_types::net::TunnelType;

impl<L> Daemon<L>
where
    L: EventListener,
{
    /// Create a new custom list.
    ///
    /// Returns an error if the name is not unique.
    pub async fn create_custom_list(&mut self, name: String) -> Result<Id, crate::Error> {
        let new_list = CustomList::new(name).map_err(crate::Error::CustomListError)?;
        let id = new_list.id;

        self.settings
            .try_update(|settings| settings.custom_lists.add(new_list))
            .await
            .map_err(Error::SettingsError)?;

        Ok(id)
    }

    /// Update a custom list.
    ///
    /// Returns an error if the list doesn't exist.
    pub async fn delete_custom_list(&mut self, id: Id) -> Result<(), Error> {
        let settings_changed = self
            .settings
            .try_update(|settings| {
                // NOTE: Not using swap remove because it would make user output slightly
                // more confusing and the cost is so small.
                settings.custom_lists.remove(&id)
            })
            .await
            .map_err(Error::SettingsError);

        if let Ok(true) = settings_changed {
            self.relay_selector
                .set_config(new_selector_config(&self.settings));

            if self.change_should_cause_reconnect(Some(id)) {
                log::info!("Initiating tunnel restart because a selected custom list was deleted");
                self.reconnect_tunnel();
            }
        }

        settings_changed?;
        Ok(())
    }

    /// Update a custom list.
    ///
    /// Returns an error if...
    /// - there is no existing list with the same ID,
    /// - or the existing list has a different name.
    pub async fn update_custom_list(&mut self, new_list: CustomList) -> Result<(), Error> {
        let list_id = new_list.id;
        let settings_changed = self
            .settings
            .try_update(|settings| settings.custom_lists.update(new_list))
            .await
            .map_err(Error::SettingsError);

        if let Ok(true) = settings_changed {
            self.relay_selector
                .set_config(new_selector_config(&self.settings));

            if self.change_should_cause_reconnect(Some(list_id)) {
                log::info!("Initiating tunnel restart because a selected custom list changed");
                self.reconnect_tunnel();
            }
        }

        settings_changed?;
        Ok(())
    }

    /// Remove all custom lists.
    pub async fn clear_custom_lists(&mut self) -> Result<(), Error> {
        let settings_changed = self
            .settings
            .update(|settings| {
                settings.custom_lists.clear();
            })
            .await
            .map_err(Error::SettingsError);

        if let Ok(true) = settings_changed {
            self.relay_selector
                .set_config(new_selector_config(&self.settings));

            if self.change_should_cause_reconnect(None) {
                log::info!("Initiating tunnel restart because a selected custom list was deleted");
                self.reconnect_tunnel();
            }
        }

        settings_changed?;
        Ok(())
    }

    /// Check whether we need to reconnect after changing custom lists.
    ///
    /// If `custom_list_id` is `Some`, only changes to that custom list will trigger a reconnect.
    fn change_should_cause_reconnect(&self, custom_list_id: Option<Id>) -> bool {
        use mullvad_types::states::TunnelState;
        let mut need_to_reconnect = false;

        let RelaySettings::Normal(relay_settings) = &self.settings.relay_settings else {
            return false;
        };

        if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
            &relay_settings.location
        {
            need_to_reconnect |= custom_list_id.map(|id| &id == list_id).unwrap_or(true);
        }

        if let TunnelState::Connecting {
            endpoint,
            location: _,
        }
        | TunnelState::Connected {
            endpoint,
            location: _,
        } = &self.tunnel_state
        {
            match endpoint.tunnel_type {
                TunnelType::Wireguard => {
                    if relay_settings.wireguard_constraints.multihop() {
                        if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
                            &relay_settings.wireguard_constraints.entry_location
                        {
                            need_to_reconnect |=
                                custom_list_id.map(|id| &id == list_id).unwrap_or(true);
                        }
                    }
                }

                TunnelType::OpenVpn => {
                    if !matches!(self.settings.bridge_state, BridgeState::Off) {
                        if let Ok(ResolvedBridgeSettings::Normal(bridge_settings)) =
                            self.settings.bridge_settings.resolve()
                        {
                            if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
                                &bridge_settings.location
                            {
                                need_to_reconnect |=
                                    custom_list_id.map(|id| &id == list_id).unwrap_or(true);
                            }
                        }
                    }
                }
            }
        }

        need_to_reconnect
    }
}
