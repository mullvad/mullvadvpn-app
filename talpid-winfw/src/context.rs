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
        let baseline = self.baseline;
        self.execute_transaction(|transaction, recorder| {
            recorder.revert_to(baseline, transaction)
        })?;
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
        self.execute_transaction(|transaction, _recorder| {
            persistent::replace_ephemeral_objects(transaction)
        })
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
        let mut context = Self {
            engine: open_engine(timeout)?,
            recorder: Recorder::default(),
            baseline: 0,
            active_policy: ActivePolicy::None,
        };

        let mut baseline = 0;
        context.execute_transaction(|transaction, recorder| {
            objects::purge_all(transaction)?;
            objects::add_provider(transaction, recorder)?;
            objects::add_sublayer_baseline(transaction, recorder)?;
            objects::add_sublayer_dns(transaction, recorder)?;
            baseline = recorder.checkpoint();
            match blocked_ruleset {
                Some(ruleset) => apply_ruleset_directly(transaction, recorder, ruleset),
                None => Ok(()),
            }
        })?;
        context.baseline = baseline;
        if blocked_ruleset.is_some() {
            context.active_policy = ActivePolicy::Blocked;
        }
        Ok(context)
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
