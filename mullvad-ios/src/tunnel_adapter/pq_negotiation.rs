//! PQ ephemeral peer negotiation orchestration.
//!
//! This module contains the logic for determining which peers to negotiate with
//! during PQ key exchange. The orchestration is separated from I/O so it can be
//! unit tested without iOS.

use std::net::SocketAddr;

/// Describes which peer to negotiate with in each PQ phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqNegotiationPlan {
    pub phases: Vec<PqPhase>,
}

/// A single PQ negotiation phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqPhase {
    /// The peer to connect to for this negotiation.
    pub connect_to: PqPeerTarget,
    /// Which device key to use for the WireGuard connection in this phase.
    pub use_key: PqKeySource,
    /// The result of this phase applies to which final device.
    pub applies_to: PqAppliesTo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PqPeerTarget {
    /// Connect to the entry peer's endpoint.
    Entry,
    /// Connect to the exit peer's endpoint (only in singlehop or through entry tunnel).
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PqKeySource {
    /// Use the device's original private key.
    DeviceKey,
    /// Use the ephemeral key from a previous phase.
    PreviousPhaseKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PqAppliesTo {
    Entry,
    Exit,
}

/// Plan the PQ negotiation phases.
///
/// - Singlehop: 1 phase — negotiate with the exit relay using device key.
/// - Multihop: 2 phases:
///   1. Negotiate with entry relay using device key → entry ephemeral key.
///   2. Connect to entry with ephemeral key, negotiate with exit → exit ephemeral key.
///
/// In multihop, the exit relay is NEVER contacted directly — all traffic goes through entry.
pub fn plan_pq_negotiation(
    entry_endpoint: Option<SocketAddr>,
    exit_endpoint: SocketAddr,
) -> PqNegotiationPlan {
    if let Some(_entry_endpoint) = entry_endpoint {
        // Multihop: two phases
        PqNegotiationPlan {
            phases: vec![
                PqPhase {
                    connect_to: PqPeerTarget::Entry,
                    use_key: PqKeySource::DeviceKey,
                    applies_to: PqAppliesTo::Entry,
                },
                PqPhase {
                    connect_to: PqPeerTarget::Entry, // through entry tunnel to reach exit's config service
                    use_key: PqKeySource::PreviousPhaseKey,
                    applies_to: PqAppliesTo::Exit,
                },
            ],
        }
    } else {
        // Singlehop: one phase
        PqNegotiationPlan {
            phases: vec![PqPhase {
                connect_to: PqPeerTarget::Exit,
                use_key: PqKeySource::DeviceKey,
                applies_to: PqAppliesTo::Exit,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn singlehop_pq_has_one_phase_targeting_exit() {
        let plan = plan_pq_negotiation(None, addr("1.2.3.4:51820"));

        assert_eq!(plan.phases.len(), 1);
        assert_eq!(plan.phases[0].connect_to, PqPeerTarget::Exit);
        assert_eq!(plan.phases[0].use_key, PqKeySource::DeviceKey);
        assert_eq!(plan.phases[0].applies_to, PqAppliesTo::Exit);
    }

    #[test]
    fn multihop_pq_has_two_phases() {
        let plan = plan_pq_negotiation(Some(addr("1.1.1.1:51820")), addr("2.2.2.2:51820"));

        assert_eq!(plan.phases.len(), 2);

        // Phase 1: negotiate with entry using device key
        assert_eq!(plan.phases[0].connect_to, PqPeerTarget::Entry);
        assert_eq!(plan.phases[0].use_key, PqKeySource::DeviceKey);
        assert_eq!(plan.phases[0].applies_to, PqAppliesTo::Entry);

        // Phase 2: negotiate with exit via entry using ephemeral key
        assert_eq!(plan.phases[1].connect_to, PqPeerTarget::Entry);
        assert_eq!(plan.phases[1].use_key, PqKeySource::PreviousPhaseKey);
        assert_eq!(plan.phases[1].applies_to, PqAppliesTo::Exit);
    }

    #[test]
    fn multihop_pq_never_contacts_exit_directly() {
        let plan = plan_pq_negotiation(Some(addr("1.1.1.1:51820")), addr("2.2.2.2:51820"));

        for phase in &plan.phases {
            assert_ne!(
                phase.connect_to,
                PqPeerTarget::Exit,
                "In multihop, no PQ phase should connect directly to the exit relay"
            );
        }
    }
}
