use crate::{new_selector_config, Daemon, Error, EventListener};
use mullvad_types::{
    constraints::Constraint,
    custom_list::{CustomList, Id},
    relay_constraints::{BridgeState, LocationConstraint, RelaySettings, ResolvedBridgeSettings},
};
use talpid_types::net::TunnelType;

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn create_custom_list(&mut self, name: String) -> Result<Id, crate::Error> {
        if self
            .settings
            .custom_lists
            .iter()
            .any(|list| list.name == name)
        {
            return Err(Error::CustomListExists);
        }

        let new_list = CustomList::new(name);
        let id = new_list.id;

        self.settings
            .update(|settings| {
                settings.custom_lists.add(new_list);
            })
            .await
            .map_err(Error::SettingsError)?;

        Ok(id)
    }

    pub async fn delete_custom_list(&mut self, id: Id) -> Result<(), Error> {
        let Some(list_index) = self
            .settings
            .custom_lists
            .iter()
            .position(|elem| elem.id == id)
        else {
            return Err(Error::CustomListNotFound);
        };
        let settings_changed = self
            .settings
            .update(|settings| {
                // NOTE: Not using swap remove because it would make user output slightly
                // more confusing and the cost is so small.
                settings.custom_lists.remove(list_index);
            })
            .await
            .map_err(Error::SettingsError);

        if let Ok(true) = settings_changed {
            self.relay_selector
                .set_config(new_selector_config(&self.settings));

            if self.change_should_cause_reconnect(id) {
                log::info!("Initiating tunnel restart because a selected custom list was deleted");
                self.reconnect_tunnel();
            }
        }

        settings_changed?;
        Ok(())
    }

    pub async fn update_custom_list(&mut self, new_list: CustomList) -> Result<(), Error> {
        let Some((list_index, old_list)) = self
            .settings
            .custom_lists
            .iter()
            .enumerate()
            .find(|elem| elem.1.id == new_list.id)
        else {
            return Err(Error::CustomListNotFound);
        };
        let id = old_list.id;

        if old_list.name != new_list.name
            && self
                .settings
                .custom_lists
                .iter()
                .any(|list| list.name == new_list.name)
        {
            return Err(Error::CustomListExists);
        }

        let settings_changed = self
            .settings
            .update(|settings| {
                settings.custom_lists[list_index] = new_list;
            })
            .await
            .map_err(Error::SettingsError);

        if let Ok(true) = settings_changed {
            self.relay_selector
                .set_config(new_selector_config(&self.settings));

            if self.change_should_cause_reconnect(id) {
                log::info!("Initiating tunnel restart because a selected custom list changed");
                self.reconnect_tunnel();
            }
        }

        settings_changed?;
        Ok(())
    }

    fn change_should_cause_reconnect(&self, custom_list_id: Id) -> bool {
        use mullvad_types::states::TunnelState;
        let mut need_to_reconnect = false;

        if let RelaySettings::Normal(relay_settings) = &self.settings.relay_settings {
            if let Constraint::Only(LocationConstraint::CustomList { list_id }) =
                &relay_settings.location
            {
                need_to_reconnect |= list_id == &custom_list_id;
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
                                need_to_reconnect |= list_id == &custom_list_id;
                            }
                        }
                    }

                    TunnelType::OpenVpn => {
                        if !matches!(self.settings.bridge_state, BridgeState::Off) {
                            if let Ok(ResolvedBridgeSettings::Normal(bridge_settings)) =
                                self.settings.bridge_settings.resolve()
                            {
                                if let Constraint::Only(LocationConstraint::CustomList {
                                    list_id,
                                }) = &bridge_settings.location
                                {
                                    need_to_reconnect |= list_id == &custom_list_id;
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
