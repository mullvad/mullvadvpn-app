use mullvad_types::{
    constraints::Constraint,
    custom_list::CustomListsSettings,
    relay_list::{Relay, WireguardRelay},
    relay_selector::{
        EntryConstraints, EntrySpecificConstraints, ExitConstraints, MultihopConstraints,
        Predicate, Reason, RelayPartitions, ResolvedLocationConstraint,
    },
};

use crate::types::{
    FromProtobufTypeError, IpVersion, relay_constraints::try_ownership_constraint_from_i32,
};
use crate::types::{relay_constraints::providers_constraint_from_proto, relay_selector as proto};

impl proto::Predicate {
    pub fn into_domain(
        self,
        custom_lists: &CustomListsSettings,
    ) -> Result<Predicate, FromProtobufTypeError> {
        let Some(context) = self.context else {
            return Err(FromProtobufTypeError::invalid_argument(
                "context must be provided",
            ));
        };
        match context {
            proto::predicate::Context::Singlehop(constraints) => constraints
                .into_domain(custom_lists)
                .map(Predicate::Singlehop),
            proto::predicate::Context::Autohop(constraints) => constraints
                .into_domain(custom_lists)
                .map(Predicate::Autohop),
            proto::predicate::Context::Entry(proto::MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = entry.into_domain(custom_lists)?;
                let exit = exit.into_domain(custom_lists)?;
                let constraints = MultihopConstraints { entry, exit };
                Ok(Predicate::Entry(constraints))
            }
            proto::predicate::Context::Exit(proto::MultiHopConstraints {
                entry: Some(entry),
                exit: Some(exit),
            }) => {
                let entry = entry.into_domain(custom_lists)?;
                let exit = exit.into_domain(custom_lists)?;
                let constraints = MultihopConstraints { entry, exit };
                Ok(Predicate::Exit(constraints))
            }
            proto::predicate::Context::Entry(_) | proto::predicate::Context::Exit(_) => Err(
                FromProtobufTypeError::invalid_argument("entry + exit must be provided"),
            ),
        }
    }
}

impl proto::EntryConstraints {
    fn into_domain(
        self,
        custom_lists: &CustomListsSettings,
    ) -> Result<EntryConstraints, FromProtobufTypeError> {
        let proto::EntryConstraints {
            general_constraints,
            obfuscation_settings,
            daita_settings,
            ip_version,
        } = self;
        let general = general_constraints
            .map(|gc| gc.into_domain(custom_lists))
            .transpose()?
            .unwrap_or_default();

        let ip_version = Constraint::from(
            ip_version
                .map(IpVersion::try_from)
                .transpose()
                .map_err(|_| {
                    FromProtobufTypeError::invalid_argument("invalid IP protocol version")
                })?
                .map(talpid_types::net::IpVersion::from),
        );

        let obfuscation_settings = obfuscation_settings
            .map(mullvad_types::relay_constraints::ObfuscationSettings::try_from)
            .transpose()?
            .unwrap_or_default();
        let obfuscation = mullvad_types::relay_constraints::obfuscation_constraint_from_settings(
            obfuscation_settings,
        );

        let daita: Constraint<_> = daita_settings.map(|ds| ds.enabled).into();

        Ok(EntryConstraints {
            general,
            entry_specific: EntrySpecificConstraints {
                obfuscation,
                daita,
                ip_version,
            },
        })
    }
}

impl proto::ExitConstraints {
    fn into_domain(
        self,
        custom_lists: &CustomListsSettings,
    ) -> Result<ExitConstraints, FromProtobufTypeError> {
        let proto::ExitConstraints {
            location,
            providers,
            ownership,
        } = self;
        let location_constraint: Constraint<_> = location
            .map(mullvad_types::relay_constraints::LocationConstraint::try_from)
            .transpose()?
            .into();
        let location =
            ResolvedLocationConstraint::from_constraint(location_constraint, custom_lists);
        let providers = providers_constraint_from_proto(&providers);
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
        // Display concern (not selection): surface `include_in_country = false` relays as
        // matches even though the relay selector itself never picks them at country level.
        // The UI consumes this list to show what the user could select, and these relays
        // remain individually selectable via city or hostname constraints — so dropping
        // them entirely would hide selectable relays from the relay-list view.
        let (fallbacks, true_discards): (Vec<_>, Vec<_>) = discards
            .into_iter()
            .partition(|(_relay, why)| matches!(why.as_slice(), [Reason::IncludeInCountry]));

        let matches = fallbacks
            .into_iter()
            .map(|(relay, _)| relay)
            .chain(matches)
            .map(proto::MatchingRelay::from)
            .collect();

        let discards = true_discards
            .into_iter()
            .map(|(relay, why)| proto::DiscardedRelay {
                relay: Some(proto::Relay::from(relay.inner)),
                why: Some(proto::IncompatibleConstraints::from(why)),
            })
            .collect();
        Self { matches, discards }
    }
}

impl From<WireguardRelay> for proto::MatchingRelay {
    fn from(relay: WireguardRelay) -> Self {
        Self {
            relay: Some(relay.inner.into()),
            metadata: Some(proto::Metadata {
                needs_other_entry: relay.needs_other_entry,
            }),
        }
    }
}

impl From<Relay> for proto::Relay {
    fn from(value: Relay) -> Self {
        Self {
            hostname: value.hostname,
        }
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
                IncludeInCountry => continue,
            };
        }
        incompatible
    }
}
