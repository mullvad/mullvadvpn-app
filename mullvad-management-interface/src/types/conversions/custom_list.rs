use std::{collections::BTreeSet, str::FromStr};

use crate::types::{proto, FromProtobufTypeError};
use mullvad_types::{
    custom_list::{CustomList, Id},
    relay_constraints::GeographicLocationConstraint,
};

impl From<mullvad_types::custom_list::CustomListsSettings> for proto::CustomListSettings {
    fn from(settings: mullvad_types::custom_list::CustomListsSettings) -> Self {
        Self {
            custom_lists: settings.into_iter().map(proto::CustomList::from).collect(),
        }
    }
}

impl TryFrom<proto::CustomListSettings> for mullvad_types::custom_list::CustomListsSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::CustomListSettings) -> Result<Self, Self::Error> {
        Ok(Self::from(
            settings
                .custom_lists
                .into_iter()
                .map(mullvad_types::custom_list::CustomList::try_from)
                .collect::<Result<Vec<CustomList>, _>>()?,
        ))
    }
}

impl From<mullvad_types::custom_list::CustomList> for proto::CustomList {
    fn from(custom_list: mullvad_types::custom_list::CustomList) -> Self {
        let locations = custom_list
            .locations
            .into_iter()
            .map(proto::GeographicLocationConstraint::from)
            .collect();
        Self {
            id: custom_list.id.to_string(),
            name: custom_list.name,
            locations,
        }
    }
}

impl TryFrom<proto::CustomList> for mullvad_types::custom_list::CustomList {
    type Error = FromProtobufTypeError;

    fn try_from(custom_list: proto::CustomList) -> Result<Self, Self::Error> {
        let locations = custom_list
            .locations
            .into_iter()
            .map(GeographicLocationConstraint::try_from)
            .collect::<Result<BTreeSet<_>, Self::Error>>()?;
        Ok(Self {
            id: Id::from_str(&custom_list.id)
                .map_err(|_| FromProtobufTypeError::InvalidArgument("Invalid list ID"))?,
            name: custom_list.name,
            locations,
        })
    }
}
