use mullvad_types::{
    constraints::Constraint,
    relay_list::Relay,
    relay_selector::{
        EntryConstraints, ExitConstraints, MultihopConstraints, Predicate, Reason, RelayPartitions,
    },
};

use crate::types::relay_selector as proto;
use crate::types::{
    FromProtobufTypeError, IpVersion, invalid_argument,
    relay_constraints::{try_ownership_constraint_from_i32, try_providers_from_proto},
};

impl TryFrom<proto::Predicate> for Predicate {
    type Error = FromProtobufTypeError;

    fn try_from(predicate: proto::Predicate) -> Result<Self, Self::Error> {
        let Some(context) = predicate.context else {
            return Err(invalid_argument("context must be provided"));
        };
        match context {
            proto::predicate::Context::Singlehop(constraints) => {
                EntryConstraints::try_from(constraints).map(Self::Singlehop)
            }
            proto::predicate::Context::Autohop(constraints) => {
                EntryConstraints::try_from(constraints).map(Self::Autohop)
            }
            proto::predicate::Context::Entry(proto::MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = EntryConstraints::try_from(entry)?;
                let exit = ExitConstraints::try_from(exit)?;
                let constraints = MultihopConstraints { entry, exit };
                Ok(Self::Entry(constraints))
            }
            proto::predicate::Context::Exit(proto::MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = EntryConstraints::try_from(entry)?;
                let exit = ExitConstraints::try_from(exit)?;
                let constraints = MultihopConstraints { entry, exit };
                Ok(Self::Exit(constraints))
            }
            proto::predicate::Context::Entry(_) | proto::predicate::Context::Exit(_) => {
                Err(invalid_argument("entry + exit must be provided"))
            }
        }
    }
}

impl TryFrom<proto::EntryConstraints> for EntryConstraints {
    type Error = FromProtobufTypeError;

    fn try_from(
        proto::EntryConstraints {
            general_constraints,
            obfuscation_settings,
            daita_settings,
            ip_version,
        }: proto::EntryConstraints,
    ) -> Result<Self, Self::Error> {
        let general = general_constraints
            .map(ExitConstraints::try_from)
            .transpose()?
            .unwrap_or_default();

        // NOTE: Provider and Ownership filters are mirrored from the provided exit constraints.
        // This is somewhat of hack to make the relay selector logic easier to implement.
        // TODO: When split filters have been implemented, grab these filters from the provided
        // entry constraints.
        let providers = general.providers.clone();
        let ownership = general.ownership;

        let ip_version: Constraint<_> = IpVersion::try_from(ip_version)
            .map_err(|_| invalid_argument("invalid IP protocol version"))
            .map(talpid_types::net::IpVersion::from)?
            .into();

        let obfuscation_settings: Constraint<_> = obfuscation_settings
            .map(mullvad_types::relay_constraints::ObfuscationSettings::try_from)
            .transpose()?
            .into();

        let daita: Constraint<_> = daita_settings
            .map(mullvad_types::wireguard::DaitaSettings::from)
            .into();

        Ok(EntryConstraints {
            general,
            obfuscation_settings,
            daita,
            ip_version,
            providers,
            ownership,
        })
    }
}

impl TryFrom<proto::ExitConstraints> for ExitConstraints {
    type Error = FromProtobufTypeError;

    fn try_from(
        proto::ExitConstraints {
            location,
            providers,
            ownership,
        }: proto::ExitConstraints,
    ) -> Result<Self, Self::Error> {
        let location: Constraint<_> = location
            .map(mullvad_types::relay_constraints::LocationConstraint::try_from)
            .transpose()?
            .into();
        let providers = try_providers_from_proto(providers)?;
        let ownership = try_ownership_constraint_from_i32(ownership)?;

        Ok(ExitConstraints {
            location,
            providers,
            ownership,
        })
    }
}

impl From<RelayPartitions> for proto::RelayPartitions {
    fn from(RelayPartitions { matches, discards }: RelayPartitions) -> Self {
        let matches = matches
            .into_iter()
            .map(|relay| relay.inner)
            .map(proto::Relay::from)
            .collect();
        let discards = discards
            .into_iter()
            .map(|(relay, why)| (relay.inner, why))
            .map(|(relay, why)| proto::DiscardedRelay {
                relay: Some(proto::Relay::from(relay)),
                why: Some(proto::IncompatibleConstraints::from(why)),
            })
            .collect();
        Self { matches, discards }
    }
}

impl From<Relay> for proto::Relay {
    fn from(Relay { hostname, .. }: Relay) -> Self {
        Self { hostname }
    }
}

impl From<Vec<Reason>> for proto::IncompatibleConstraints {
    fn from(reasons: Vec<Reason>) -> Self {
        use Reason::*;
        let mut incompatible = Self::default();
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
