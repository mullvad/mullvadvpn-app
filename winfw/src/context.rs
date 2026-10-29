//! The firewall session context.
//!
//! Owns the WFP engine session and the record of objects it added, applies
//! policies within transactions, and reverts to the base configuration
//! (provider and sublayers) between policies.

use std::{io, time::Duration};

use wfp::{FilterEngine, FilterEngineBuilder, Transaction};

use crate::{ActivePolicy, objects, rules, session::Recorder};

/// The firewall session context.
pub struct FwContext {
    engine: FilterEngine,
    recorder: Recorder,
    baseline: u32,
    active_policy: ActivePolicy,
}

impl FwContext {
    /// Connect to the engine, remove any leftover objects from
    /// earlier runs, install the structural objects and checkpoint them.
    pub fn new(timeout: Duration) -> io::Result<Self> {
        let engine = open_engine(timeout)?;
        let mut context = Self {
            engine,
            recorder: Recorder::default(),
            baseline: 0,
            active_policy: ActivePolicy::None,
        };
        context.apply_base_configuration()?;
        Ok(context)
    }

    /// The same as [`Self::new`], with the blocked policy applied within the
    /// same initialization transaction.
    pub fn new_blocked(timeout: Duration, ruleset: &[rules::Rule]) -> io::Result<Self> {
        let engine = open_engine(timeout)?;
        let mut context = Self {
            engine,
            recorder: Recorder::default(),
            baseline: 0,
            active_policy: ActivePolicy::None,
        };
        context.apply_blocked_base_configuration(ruleset)?;
        context.active_policy = ActivePolicy::Blocked;
        Ok(context)
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
        let baseline = self.baseline;
        self.execute_transaction(|transaction, recorder| {
            recorder.revert_to(baseline, transaction)
        })?;
        self.active_policy = ActivePolicy::None;
        Ok(())
    }

    pub fn active_policy(&self) -> ActivePolicy {
        self.active_policy
    }

    /// Purge everything and reinstall the
    /// structural objects, in one transaction.
    fn apply_base_configuration(&mut self) -> io::Result<()> {
        let mut baseline = 0;
        self.execute_transaction(|transaction, recorder| {
            objects::purge_all(transaction)?;
            objects::add_provider(transaction, recorder)?;
            objects::add_sublayer_baseline(transaction, recorder)?;
            objects::add_sublayer_dns(transaction, recorder)?;
            baseline = recorder.checkpoint();
            Ok(())
        })?;
        self.baseline = baseline;
        Ok(())
    }

    /// As above, with the blocking
    /// policy added in the same transaction, past the baseline checkpoint.
    fn apply_blocked_base_configuration(
        &mut self,
        blocked_ruleset: &[rules::Rule],
    ) -> io::Result<()> {
        let mut baseline = 0;
        self.execute_transaction(|transaction, recorder| {
            objects::purge_all(transaction)?;
            objects::add_provider(transaction, recorder)?;
            objects::add_sublayer_baseline(transaction, recorder)?;
            objects::add_sublayer_dns(transaction, recorder)?;
            baseline = recorder.checkpoint();
            apply_ruleset_directly(transaction, recorder, blocked_ruleset)
        })?;
        self.baseline = baseline;
        Ok(())
    }

    /// Revert to the baseline, then install the ruleset, in one transaction.
    fn apply_ruleset(&mut self, ruleset: &[rules::Rule]) -> io::Result<()> {
        let baseline = self.baseline;
        self.execute_transaction(|transaction, recorder| {
            recorder.revert_to(baseline, transaction)?;
            apply_ruleset_directly(transaction, recorder, ruleset)
        })
    }

    /// Run a transaction on a copy of the records, and adopt it only when it
    /// commits.
    fn execute_transaction(
        &mut self,
        operation: impl FnOnce(&Transaction<'_>, &mut Recorder) -> io::Result<()>,
    ) -> io::Result<()> {
        let transaction = Transaction::new(&mut self.engine)?;
        let mut recorder = self.recorder.snapshot();

        let result = operation(&transaction, &mut recorder);
        match result {
            Ok(()) => transaction.commit()?,
            Err(error) => {
                if let Err(abort_error) = transaction.abort() {
                    log::warn!("Failed to abort the WFP transaction: {abort_error}");
                }
                return Err(error);
            }
        }

        self.recorder.adopt(recorder);
        Ok(())
    }
}

/// Install every rule, recording objects.
fn apply_ruleset_directly(
    transaction: &Transaction<'_>,
    recorder: &mut Recorder,
    ruleset: &[rules::Rule],
) -> io::Result<()> {
    for rule in ruleset {
        rule.install(&mut rules::Cx {
            transaction,
            recorder,
        })?;
    }
    Ok(())
}

/// Open the standard WFP session with the transaction-lock wait the daemon
/// uses.
pub fn open_engine(timeout: Duration) -> io::Result<FilterEngine> {
    FilterEngineBuilder::default()
        .transaction_timeout(timeout)
        .open()
}

/// Purge every firewall-owned object, outside any session. This is what
/// [`crate::Manager::reset`] does when the module is not initialized, and how
/// a new run cleans up after a crashed one.
pub fn purge_standalone() -> io::Result<()> {
    let mut engine = open_engine(Duration::ZERO)?;
    let transaction = Transaction::new(&mut engine)?;
    objects::purge_all(&transaction)?;
    transaction.commit()
}
