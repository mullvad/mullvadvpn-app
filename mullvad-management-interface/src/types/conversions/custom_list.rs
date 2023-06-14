use crate::types::{proto, FromProtobufTypeError};
use mullvad_types::relay_constraints::LocationConstraint;
use proto::RelayLocation;

impl From<&mullvad_types::custom_list::CustomListsSettings> for proto::CustomListSettings {
    fn from(settings: &mullvad_types::custom_list::CustomListsSettings) -> Self {
        Self {
            custom_lists: settings
                .custom_lists
                .iter()
                .map(|(id, custom_list)| {
                    (id.clone(), proto::CustomList::from(custom_list.clone()))
                })
            .collect(),
            selected_list_entry: settings.selected_list_entry.clone(),
            selected_list_exit: settings.selected_list_exit.clone(),
        }
    }
}

impl TryFrom<proto::CustomListSettings> for mullvad_types::custom_list::CustomListsSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::CustomListSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            custom_lists: settings
                .custom_lists
                .into_iter()
                .map(|(id, custom_list)| {
                    Ok((
                            id,
                            mullvad_types::custom_list::CustomList::try_from(custom_list)?,
                    ))
                })
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?,
            selected_list_entry: settings.selected_list_entry,
            selected_list_exit: settings.selected_list_exit,
        })
    }
}

impl From<mullvad_types::custom_list::CustomListLocationUpdate> for proto::CustomListLocationUpdate {
    fn from(custom_list: mullvad_types::custom_list::CustomListLocationUpdate) -> Self {
        use mullvad_types::relay_constraints::Constraint;
        match custom_list {
            mullvad_types::custom_list::CustomListLocationUpdate::Add {
                name, location
            } => {
                let location = match location {
                    Constraint::Any => None,
                    Constraint::Only(location) => Some(RelayLocation::from(location)),
                };
                Self {
                    state: 0,
                    name,
                    location,
                }
            },
            mullvad_types::custom_list::CustomListLocationUpdate::Remove {
                name, location
            } => {
                let location = match location {
                    Constraint::Any => None,
                    Constraint::Only(location) => Some(RelayLocation::from(location)),
                };
                Self {
                    state: 1,
                    name,
                    location,
                }
            },
        }
    }
}

impl TryFrom<proto::CustomListLocationUpdate> for mullvad_types::custom_list::CustomListLocationUpdate {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomListLocationUpdate) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::Constraint;
        let location: Constraint<LocationConstraint> =
            Constraint::<LocationConstraint>::from(
                custom_list
                    .location
                    .ok_or(FromProtobufTypeError::InvalidArgument("missing location"))?,
            );
        match custom_list.state {
            0 => {
                Ok(Self::Add {
                    name: custom_list.name,
                    location,
                })
            },
            1 => {
                Ok(Self::Remove {
                    name: custom_list.name,
                    location,
                })
            },
            _ => {
                Err(FromProtobufTypeError::InvalidArgument("incorrect state"))
            }
        }
    }
}

impl From<mullvad_types::custom_list::CustomList> for proto::CustomList {
    fn from(custom_list: mullvad_types::custom_list::CustomList) -> Self {
        let locations = custom_list
            .locations
            .into_iter()
            .map(proto::RelayLocation::from)
            .collect();
        Self {
            id: custom_list.id,
            name: custom_list.name,
            locations,
        }
    }
}

impl TryFrom<proto::CustomList> for mullvad_types::custom_list::CustomList {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomList) -> Result<Self, Self::Error> {
        let locations: Result<Vec<LocationConstraint>, _> = custom_list
            .locations
            .into_iter()
            .map(LocationConstraint::try_from)
            .collect();
        let locations = locations.map_err(|_| {
            FromProtobufTypeError::InvalidArgument("Could not convert custom list from proto")
        })?;
        Ok(Self {
            id: custom_list.id,
            name: custom_list.name,
            locations,
        })
    }
}

impl TryFrom<proto::RelayLocation> for LocationConstraint {
    type Error = FromProtobufTypeError;

    fn try_from(relay_location: proto::RelayLocation) -> Result<Self, Self::Error> {
        match (relay_location.country.as_ref(), relay_location.city.as_ref(), relay_location.hostname.as_ref()) {
            ("", _, _) => {
                Err(FromProtobufTypeError::InvalidArgument("Relay location formatted incorrectly"))
            }
            (_country, "", "") => {
                Ok(LocationConstraint::Country(relay_location.country))
            }
            (_country, _city, "") => {
                Ok(LocationConstraint::City(relay_location.country, relay_location.city))
            }
            (_country, city, _hostname) => {
                if city.is_empty() {
                    Err(FromProtobufTypeError::InvalidArgument("Relay location must contain a city if hostname is included"))
                } else {
                    Ok(LocationConstraint::Hostname(relay_location.country, relay_location.city, relay_location.hostname))
                }
            }
        }
    }
}

impl From<Vec<mullvad_types::custom_list::CustomList>> for proto::CustomLists {
    fn from(custom_lists: Vec<mullvad_types::custom_list::CustomList>) -> Self {
        let custom_lists = custom_lists.into_iter().map(proto::CustomList::from).collect();
        proto::CustomLists { custom_lists }
    }
}

impl TryFrom<proto::CustomLists> for Vec<mullvad_types::custom_list::CustomList> {
    type Error = FromProtobufTypeError;

    fn try_from(custom_lists: proto::CustomLists) -> Result<Self, Self::Error> {
        let mut new_custom_lists = Vec::with_capacity(custom_lists.custom_lists.len());
        for custom_list in custom_lists.custom_lists {
            new_custom_lists.push(mullvad_types::custom_list::CustomList::try_from(custom_list)?);
        }
        Ok(new_custom_lists)
    }
}

