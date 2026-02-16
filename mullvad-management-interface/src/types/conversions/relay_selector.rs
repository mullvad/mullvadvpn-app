use crate::types::relay_selector::*;

impl TryFrom<Predicate> for mullvad_relay_selector::Predicate {
    type Error = String;

    fn try_from(predicate: Predicate) -> Result<Self, Self::Error> {
        let Some(context) = predicate.context else {
            todo!("Return early");
        };
        match context {
            predicate::Context::Singlehop(_constraints) => Ok(Self::Singlehop),
            predicate::Context::Autohop(_constraints) => Ok(Self::Autohop),
            predicate::Context::Entry(_constraints) => Ok(Self::Entry),
            predicate::Context::Exit(_constraints) => Ok(Self::Exit),
        }
    }
}

impl From<mullvad_relay_selector::RelayPartitions> for RelayPartitions {
    fn from(result: mullvad_relay_selector::RelayPartitions) -> Self {
        let mullvad_relay_selector::RelayPartitions { matches, discards } = result;
        let matches = matches.into_iter().map(Relay::from).collect();
        let discards = discards
            .into_iter()
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

impl From<Vec<mullvad_relay_selector::Reject>> for IncompatibleConstraints {
    fn from(reasons: Vec<mullvad_relay_selector::Reject>) -> Self {
        use mullvad_relay_selector::Reject::*;
        let mut incompatible = IncompatibleConstraints::default();
        for reason in reasons {
            match reason {
                Inactive => incompatible.inactive = true,
            };
        }
        incompatible
    }
}
