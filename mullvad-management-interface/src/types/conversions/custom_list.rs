use crate::types::{proto, FromProtobufTypeError};
use mullvad_types::{custom_list::Id, relay_constraints::GeographicLocationConstraint};
use proto::RelayLocation;

impl From<(String, String)> for proto::CustomListRename {
    fn from(names: (String, String)) -> Self {
        proto::CustomListRename {
            name: names.0,
            new_name: names.1,
        }
    }
}

impl From<proto::CustomListRename> for (String, String) {
    fn from(names: proto::CustomListRename) -> Self {
        (names.name, names.new_name)
    }
}

impl From<&mullvad_types::custom_list::CustomListsSettings> for proto::CustomListSettings {
    fn from(settings: &mullvad_types::custom_list::CustomListsSettings) -> Self {
        Self {
            custom_lists: settings
                .custom_lists
                .iter()
                .map(|(id, custom_list)| {
                    (
                        id.0.to_string(),
                        proto::CustomList::from(custom_list.clone()),
                    )
                })
                .collect(),
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
                        Id::try_from(id.as_str()).map_err(|_| {
                            FromProtobufTypeError::InvalidArgument(
                                "Id could not be parsed to a uuid",
                            )
                        })?,
                        mullvad_types::custom_list::CustomList::try_from(custom_list)?,
                    ))
                })
                .collect::<Result<std::collections::HashMap<_, _>, _>>()?,
        })
    }
}

impl From<mullvad_types::custom_list::CustomListLocationUpdate>
    for proto::CustomListLocationUpdate
{
    fn from(custom_list: mullvad_types::custom_list::CustomListLocationUpdate) -> Self {
        use mullvad_types::relay_constraints::Constraint;
        match custom_list {
            mullvad_types::custom_list::CustomListLocationUpdate::Add { name, location } => {
                let location = match location {
                    Constraint::Any => None,
                    Constraint::Only(location) => Some(RelayLocation::from(location)),
                };
                Self {
                    state: i32::from(proto::custom_list_location_update::State::Add),
                    name,
                    location,
                }
            }
            mullvad_types::custom_list::CustomListLocationUpdate::Remove { name, location } => {
                let location = match location {
                    Constraint::Any => None,
                    Constraint::Only(location) => Some(RelayLocation::from(location)),
                };
                Self {
                    state: i32::from(proto::custom_list_location_update::State::Remove),
                    name,
                    location,
                }
            }
        }
    }
}

impl TryFrom<proto::CustomListLocationUpdate>
    for mullvad_types::custom_list::CustomListLocationUpdate
{
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomListLocationUpdate) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::Constraint;
        let location: Constraint<GeographicLocationConstraint> =
            Constraint::<GeographicLocationConstraint>::from(
                custom_list
                    .location
                    .ok_or(FromProtobufTypeError::InvalidArgument("missing location"))?,
            );
        match proto::custom_list_location_update::State::from_i32(custom_list.state) {
            Some(proto::custom_list_location_update::State::Add) => Ok(Self::Add {
                name: custom_list.name,
                location,
            }),
            Some(proto::custom_list_location_update::State::Remove) => Ok(Self::Remove {
                name: custom_list.name,
                location,
            }),
            None => Err(FromProtobufTypeError::InvalidArgument("incorrect state")),
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
            id: custom_list.id.0.to_string(),
            name: custom_list.name,
            locations,
        }
    }
}

impl TryFrom<proto::CustomList> for mullvad_types::custom_list::CustomList {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomList) -> Result<Self, Self::Error> {
        let locations: Result<Vec<GeographicLocationConstraint>, _> = custom_list
            .locations
            .into_iter()
            .map(GeographicLocationConstraint::try_from)
            .collect();
        let locations = locations.map_err(|_| {
            FromProtobufTypeError::InvalidArgument("Could not convert custom list from proto")
        })?;
        Ok(Self {
            id: Id::try_from(custom_list.id.as_str()).map_err(|_| {
                FromProtobufTypeError::InvalidArgument("Id could not be parsed to a uuid")
            })?,
            name: custom_list.name,
            locations,
        })
    }
}

impl TryFrom<proto::RelayLocation> for GeographicLocationConstraint {
    type Error = FromProtobufTypeError;

    fn try_from(relay_location: proto::RelayLocation) -> Result<Self, Self::Error> {
        match (
            relay_location.country.as_ref(),
            relay_location.city.as_ref(),
            relay_location.hostname.as_ref(),
        ) {
            ("", ..) => Err(FromProtobufTypeError::InvalidArgument(
                "Relay location formatted incorrectly",
            )),
            (_country, "", "") => Ok(GeographicLocationConstraint::Country(
                relay_location.country,
            )),
            (_country, _city, "") => Ok(GeographicLocationConstraint::City(
                relay_location.country,
                relay_location.city,
            )),
            (_country, city, _hostname) => {
                if city.is_empty() {
                    Err(FromProtobufTypeError::InvalidArgument(
                        "Relay location must contain a city if hostname is included",
                    ))
                } else {
                    Ok(GeographicLocationConstraint::Hostname(
                        relay_location.country,
                        relay_location.city,
                        relay_location.hostname,
                    ))
                }
            }
        }
    }
}

impl From<Vec<mullvad_types::custom_list::CustomList>> for proto::CustomLists {
    fn from(custom_lists: Vec<mullvad_types::custom_list::CustomList>) -> Self {
        let custom_lists = custom_lists
            .into_iter()
            .map(proto::CustomList::from)
            .collect();
        proto::CustomLists { custom_lists }
    }
}

impl TryFrom<proto::CustomLists> for Vec<mullvad_types::custom_list::CustomList> {
    type Error = FromProtobufTypeError;

    fn try_from(custom_lists: proto::CustomLists) -> Result<Self, Self::Error> {
        let mut new_custom_lists = Vec::with_capacity(custom_lists.custom_lists.len());
        for custom_list in custom_lists.custom_lists {
            new_custom_lists.push(mullvad_types::custom_list::CustomList::try_from(
                custom_list,
            )?);
        }
        Ok(new_custom_lists)
    }
}
