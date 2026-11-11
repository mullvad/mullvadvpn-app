//! The firewall session context.
//!
//! Owns the WFP engine session and the record of objects it added, applies
//! policies within transactions, and reverts to the base configuration
//! (provider and sublayers) between policies.

use std::{io, time::Duration};

use wfp::{FilterEngine, FilterEngineBuilder, Transaction};

use crate::{ActivePolicy, objects, persistent, rules, session::Recorder};

/// The firewall session context.
pub struct FwContext {
    engine: FilterEngine,
    recorder: Recorder,
    baseline: usize,
    active_policy: ActivePolicy,
}

impl FwContext {
    /// Connect to the engine, remove any leftover objects from
    /// earlier runs, install the structural objects and checkpoint them.
    pub fn new(timeout: Duration) -> io::Result<Self> {
        Self::with_base_configuration(timeout, None)
    }

    /// The same as [`Self::new`], with the blocked policy applied within the
    /// same initialization transaction.
    pub fn new_blocked(timeout: Duration, ruleset: &[rules::Rule]) -> io::Result<Self> {
        Self::with_base_configuration(timeout, Some(ruleset))
    }

    pub fn apply_policy_connecting(&mut self, ruleset: &[rules::Rule]) -> io::Result<()> {
        self.apply_ruleset(ruleset)?;
        self.active_policy = ActivePolicy::Connecting;
        Ok(())
    }

    pub fn apply_policy_connected(&mut self, ruleset: &[rules::Rule]) -> io::Result<()> {
        self.apply_ruleset(ruleset)?;
        self.active_policy = ActivePolicy::Connected;
        Ok(())
    }

    pub fn apply_policy_blocked(&mut self, ruleset: &[rules::Rule]) -> io::Result<()> {
        self.apply_ruleset(ruleset)?;
        self.active_policy = ActivePolicy::Blocked;
        Ok(())
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.apply_ruleset(&[])?;
        self.active_policy = ActivePolicy::None;
        Ok(())
    }

    /// Replace the ephemeral objects with persistent and boot-time blocking filters, and
    /// close the session. If this fails, the ephemeral objects are left in place.
    ///
    /// # Blocking
    ///
    /// Waits for the WFP transaction lock for up to the timeout the context was created
    /// with, and then as long as the filter engine needs to replace the filters.
    pub fn apply_persistent_blocking(mut self) -> io::Result<()> {
        let transaction = Transaction::new(&mut self.engine)?;
        persistent::replace_ephemeral_objects(&transaction)?;
        transaction.commit()
    }

    /// Remove every firewall-owned object, not just the current policy, and close the
    /// session.
    ///
    /// # Blocking
    ///
    /// Waits for the WFP transaction lock for up to the timeout the context was created
    /// with, and then as long as the filter engine needs to remove the objects.
    pub fn purge(mut self) -> io::Result<()> {
        let transaction = Transaction::new(&mut self.engine)?;
        objects::purge_all(&transaction)?;
        transaction.commit()
    }

    pub fn active_policy(&self) -> ActivePolicy {
        self.active_policy
    }

    /// Purge everything and reinstall the structural objects, in one
    /// transaction. The blocked ruleset, if any, is added in the same
    /// transaction, past the baseline checkpoint.
    fn with_base_configuration(
        timeout: Duration,
        blocked_ruleset: Option<&[rules::Rule]>,
    ) -> io::Result<Self> {
        let mut engine = open_engine(timeout)?;
        let mut recorder = Recorder::default();

        let transaction = Transaction::new(&mut engine)?;
        objects::purge_all(&transaction)?;
        objects::add_provider(&transaction, &mut recorder)?;
        objects::add_sublayer_baseline(&transaction, &mut recorder)?;
        objects::add_sublayer_dns(&transaction, &mut recorder)?;
        let baseline = recorder.checkpoint();
        if let Some(ruleset) = blocked_ruleset {
            recorder = apply_ruleset_directly(&transaction, recorder, ruleset)?;
        }
        transaction.commit()?;

        Ok(Self {
            engine,
            recorder,
            baseline,
            active_policy: match blocked_ruleset {
                Some(_) => ActivePolicy::Blocked,
                None => ActivePolicy::None,
            },
        })
    }

    /// Revert to the baseline, then install the ruleset, in one transaction. The
    /// records are only updated if the transaction commits.
    fn apply_ruleset(&mut self, ruleset: &[rules::Rule]) -> io::Result<()> {
        let transaction = Transaction::new(&mut self.engine)?;
        let mut recorder = self
            .recorder
            .clone()
            .revert_to(self.baseline, &transaction)?;

        recorder = apply_ruleset_directly(&transaction, recorder, ruleset)?;
        transaction.commit()?;

        self.recorder = recorder;
        Ok(())
    }
}

/// Install every rule, recording objects.
fn apply_ruleset_directly(
    transaction: &Transaction<'_>,
    mut recorder: Recorder,
    ruleset: &[rules::Rule],
) -> io::Result<Recorder> {
    for rule in ruleset {
        rule.install(&mut rules::Cx {
            transaction,
            recorder: &mut recorder,
        })?;
    }
    Ok(recorder)
}

/// Open the standard WFP session with the transaction-lock wait the daemon
/// uses.
///
/// [`Duration::ZERO`] lets the Base Filtering Engine pick its own default
/// timeout.
pub fn open_engine(timeout: Duration) -> io::Result<FilterEngine> {
    FilterEngineBuilder::default()
        .transaction_timeout(timeout)
        .open()
}

/// Purge every firewall-owned object, outside any session. This is how the
/// firewall is reset when the module is not initialized.
pub fn purge_standalone() -> io::Result<()> {
    let mut engine = open_engine(Duration::ZERO)?;
    let transaction = Transaction::new(&mut engine)?;
    objects::purge_all(&transaction)?;
    transaction.commit()
}
