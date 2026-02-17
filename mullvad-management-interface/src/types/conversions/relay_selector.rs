use mullvad_types::constraints::Constraint;

use crate::types::{
    FromProtobufTypeError, IpVersion,
    relay_constraints::{try_ownership_constraint_from_i32, try_providers_from_proto},
    relay_selector::*,
};

impl TryFrom<Predicate> for mullvad_relay_selector::Predicate {
    type Error = FromProtobufTypeError;

    fn try_from(predicate: Predicate) -> Result<Self, Self::Error> {
        let Some(context) = predicate.context else {
            todo!("Return early");
        };
        match context {
            predicate::Context::Singlehop(constraints) => {
                let EntryConstraints {
                    general_constraints,
                    obfuscation_settings,
                    daita_settings,
                    ip_version,
                } = constraints;
                // TODO: It might be beneficial to consolidate this whole conversion into a single
                // type.
                let (location, providers, ownership) = {
                    match general_constraints {
                        None => Default::default(),
                        Some(constraints) => {
                            let location = constraints
                                .location
                                .map(mullvad_types::relay_constraints::LocationConstraint::try_from)
                                .transpose()?
                                .into();
                            let providers = try_providers_from_proto(constraints.providers)?;
                            let ownership =
                                try_ownership_constraint_from_i32(constraints.ownership)?;
                            (location, providers, ownership)
                        }
                    }
                };

                let ip_version = IpVersion::try_from(ip_version)
                    .map(talpid_types::net::IpVersion::from)
                    .map(Constraint::Only)
                    .map_err(|_| {
                        FromProtobufTypeError::InvalidArgument("invalid IP protocol version")
                    })?;

                let obfuscation_settings = obfuscation_settings
                    .map(mullvad_types::relay_constraints::ObfuscationSettings::try_from)
                    .transpose()?
                    .into();

                let daita = daita_settings
                    .map(mullvad_types::wireguard::DaitaSettings::from)
                    .into();

                Ok(Self::Singlehop {
                    location,
                    providers,
                    ownership,
                    obfuscation_settings,
                    daita,
                    ip_version,
                })
            }
            predicate::Context::Autohop(constraints) => {
                let EntryConstraints {
                    general_constraints,
                    obfuscation_settings,
                    daita_settings,
                    ip_version,
                } = constraints;

                // TODO: It might be beneficial to consolidate this whole conversion into a single
                // type.
                let (location, providers, ownership) = {
                    match general_constraints {
                        None => Default::default(),
                        Some(constraints) => {
                            let location = constraints
                                .location
                                .map(mullvad_types::relay_constraints::LocationConstraint::try_from)
                                .transpose()?
                                .into();
                            let providers = try_providers_from_proto(constraints.providers)?;
                            let ownership =
                                try_ownership_constraint_from_i32(constraints.ownership)?;
                            (location, providers, ownership)
                        }
                    }
                };

                let ip_version = IpVersion::try_from(ip_version)
                    .map(talpid_types::net::IpVersion::from)
                    .map(Constraint::Only)
                    .map_err(|_| {
                        FromProtobufTypeError::InvalidArgument("invalid IP protocol version")
                    })?;

                let obfuscation_settings = obfuscation_settings
                    .map(mullvad_types::relay_constraints::ObfuscationSettings::try_from)
                    .transpose()?
                    .into();

                let daita = daita_settings
                    .map(mullvad_types::wireguard::DaitaSettings::from)
                    .into();

                Ok(Self::Autohop {
                    location,
                    providers,
                    ownership,
                    obfuscation_settings,
                    daita,
                    ip_version,
                })
            }
            predicate::Context::Entry(_constraints) => Ok(Self::Entry),
            predicate::Context::Exit(_constraints) => Ok(Self::Exit),
        }
    }
}

impl From<mullvad_relay_selector::RelayPartitions> for RelayPartitions {
    fn from(result: mullvad_relay_selector::RelayPartitions) -> Self {
        let mullvad_relay_selector::RelayPartitions { matches, discards } = result;
        let matches = matches
            .into_iter()
            // TODO: Adapt with `WireguardRelay` type
            .map(|relay| relay.inner)
            .map(Relay::from)
            .collect();
        let discards = discards
            .into_iter()
            // TODO: Adapt with `WireguardRelay` type
            .map(|(relay, why)| (relay.inner, why))
            .map(|(relay, why)| DiscardedRelay {
                relay: Some(Relay::from(relay)),
                why: Some(IncompatibleConstraints::from(why)),
            })
            .collect();
        Self { matches, discards }
    }
}

impl From<mullvad_relay_selector::Relay> for Relay {
    fn from(relay: mullvad_relay_selector::Relay) -> Self {
        Relay {
            hostname: relay.hostname,
        }
    }
}

impl From<Vec<mullvad_relay_selector::Reason>> for IncompatibleConstraints {
    fn from(reasons: Vec<mullvad_relay_selector::Reason>) -> Self {
        use mullvad_relay_selector::Reason::*;
        let mut incompatible = IncompatibleConstraints::default();
        for reason in reasons {
            match reason {
                Inactive => incompatible.inactive = true,
                Ownership => incompatible.ownership = true,
                Location => incompatible.location = true,
                Providers => incompatible.providers = true,
                IpVersion => incompatible.ip_version = true,
                Daita => incompatible.daita = true,
                Obfuscation => incompatible.obfuscation = true,
                Port => incompatible.port = true,
                Conflict => incompatible.conflict_with_other_hop = true,
            };
        }
        incompatible
    }
}
