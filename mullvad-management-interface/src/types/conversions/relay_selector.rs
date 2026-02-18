use mullvad_types::constraints::Constraint;

use crate::types::{
    FromProtobufTypeError, IpVersion, invalid_argument,
    relay_constraints::{try_ownership_constraint_from_i32, try_providers_from_proto},
    relay_selector::*,
};

impl TryFrom<Predicate> for mullvad_relay_selector::Predicate {
    type Error = FromProtobufTypeError;

    fn try_from(predicate: Predicate) -> Result<Self, Self::Error> {
        let Some(context) = predicate.context else {
            return Err(invalid_argument("context must be provided"));
        };
        match context {
            predicate::Context::Singlehop(constraints) => {
                mullvad_relay_selector::EntryConstraints::try_from(constraints).map(Self::Singlehop)
            }
            predicate::Context::Autohop(constraints) => {
                mullvad_relay_selector::EntryConstraints::try_from(constraints).map(Self::Autohop)
            }
            predicate::Context::Entry(MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = mullvad_relay_selector::EntryConstraints::try_from(entry)?;
                let exit = mullvad_relay_selector::ExitConstraints::try_from(exit)?;
                let constraints = mullvad_relay_selector::MultihopConstraints { entry, exit };
                Ok(Self::Entry(constraints))
            }
            predicate::Context::Exit(MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = mullvad_relay_selector::EntryConstraints::try_from(entry)?;
                let exit = mullvad_relay_selector::ExitConstraints::try_from(exit)?;
                let constraints = mullvad_relay_selector::MultihopConstraints { entry, exit };
                Ok(Self::Exit(constraints))
            }
            predicate::Context::Entry(_) | predicate::Context::Exit(_) => {
                Err(invalid_argument("entry + exit must be provided"))
            }
        }
    }
}

impl TryFrom<EntryConstraints> for mullvad_relay_selector::EntryConstraints {
    type Error = FromProtobufTypeError;

    fn try_from(
        EntryConstraints {
            general_constraints,
            obfuscation_settings,
            daita_settings,
            ip_version,
        }: EntryConstraints,
    ) -> Result<Self, Self::Error> {
        let general = general_constraints
            .map(mullvad_relay_selector::ExitConstraints::try_from)
            .transpose()?
            .unwrap_or_default();

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

        Ok(mullvad_relay_selector::EntryConstraints {
            general,
            obfuscation_settings,
            daita,
            ip_version,
        })
    }
}

impl TryFrom<ExitConstraints> for mullvad_relay_selector::ExitConstraints {
    type Error = FromProtobufTypeError;

    fn try_from(
        ExitConstraints {
            location,
            providers,
            ownership,
        }: ExitConstraints,
    ) -> Result<Self, Self::Error> {
        let location: Constraint<_> = location
            .map(mullvad_types::relay_constraints::LocationConstraint::try_from)
            .transpose()?
            .into();
        let providers = try_providers_from_proto(providers)?;
        let ownership = try_ownership_constraint_from_i32(ownership)?;

        Ok(mullvad_relay_selector::ExitConstraints {
            location,
            providers,
            ownership,
        })
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
    fn from(mullvad_relay_selector::Relay { hostname, .. }: mullvad_relay_selector::Relay) -> Self {
        Relay { hostname }
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
