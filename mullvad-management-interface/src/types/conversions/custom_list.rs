use crate::types::{proto, FromProtobufTypeError};
use proto::RelayLocation;

impl From<&mullvad_types::custom_list::CustomListsSettings> for proto::CustomListSettings {
    fn from(settings: &mullvad_types::custom_list::CustomListsSettings) -> Self {
        Self {
            custom_lists: settings
                .custom_lists
                .iter()
                .map(|(name, custom_list)| {
                    (name.clone(), proto::CustomList::from(&custom_list.clone()))
                })
            .collect(),
            selected_list: settings.selected_list.clone(),
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
                .map(|(name, custom_list)| {
                    Ok((
                            name,
                            mullvad_types::custom_list::CustomList::try_from(custom_list)?,
                    ))
                })
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?,
            selected_list: settings.selected_list,
        })
    }
}

impl From<mullvad_types::custom_list::CustomListLocationUpdate> for proto::CustomListLocationUpdate {
    fn from(custom_list: mullvad_types::custom_list::CustomListLocationUpdate) -> Self {
        match custom_list {
            mullvad_types::custom_list::CustomListLocationUpdate::Add {
                name, location
            } => {
                Self {
                    state: 0,
                    name,
                    location: Some(RelayLocation::from(location)),
                }
            },
            mullvad_types::custom_list::CustomListLocationUpdate::Remove {
                name, location
            } => {
                Self {
                    state: 1,
                    name,
                    location: Some(RelayLocation::from(location)),
                }
            },
        }
    }
}

impl TryFrom<proto::CustomListLocationUpdate> for mullvad_types::custom_list::CustomListLocationUpdate {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomListLocationUpdate) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::{Constraint, LocationConstraint};
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

impl From<&mullvad_types::custom_list::CustomList> for proto::CustomList {
    fn from(custom_list: &mullvad_types::custom_list::CustomList) -> Self {
        let locations = custom_list
            .locations
            .iter()
            .cloned()
            .map(proto::RelayLocation::from)
            .collect();
        Self {
            id: custom_list.id.clone(),
            name: custom_list.name.clone(),
            locations,
        }
    }
}

impl TryFrom<proto::CustomList> for mullvad_types::custom_list::CustomList {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomList) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::{Constraint, LocationConstraint};
        let locations: Result<Vec<Constraint<LocationConstraint>>, _> = custom_list
            .locations
            .into_iter()
            .map(Constraint::<LocationConstraint>::try_from)
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
