//! Tracking of WFP objects added within a session, with checkpoint/revert.
//!
//! The Windows Filtering Platform has no notion of rolling back a set of
//! objects, so the firewall records every object it adds. `revert_to`
//! uninstalls everything added after a checkpoint, newest first. Checkpoints
//! are shared per process: two firewall instances initialized in sequence
//! never confuse each other's records, matching the C++ implementation where
//! the key base is a process-global counter.

use std::{
    io,
    sync::atomic::{AtomicU32, Ordering},
};

use wfp::{Guid, Transaction, delete_filter, delete_provider, delete_sublayer};

use crate::objects::ignore_not_found;

/// A WFP object added by the firewall.
#[derive(Clone, Copy, Debug)]
pub enum Record {
    Provider(Guid),
    Sublayer(Guid),
    Filter(u64),
}

/// Process-global source of checkpoint keys, so keys are never reused.
static KEY_BASE: AtomicU32 = AtomicU32::new(1);

/// The records of one firewall session.
#[derive(Default)]
pub struct Recorder {
    records: Vec<(u32, Record)>,
}

impl Recorder {
    /// Record an added object.
    pub fn record(&mut self, record: Record) {
        let key = KEY_BASE.fetch_add(1, Ordering::Relaxed);
        self.records.push((key, record));
    }

    /// The key of the most recently added object, for passing to
    /// [`Recorder::revert_to`].
    pub fn checkpoint(&self) -> u32 {
        self.records.last().map_or(0, |(key, _)| *key)
    }

    /// Take a copy of the records, for a transaction working set.
    pub fn snapshot(&self) -> Self {
        Self {
            records: self.records.clone(),
        }
    }

    /// Replace the records with those of a committed transaction.
    pub fn adopt(&mut self, records: Recorder) {
        self.records = records.records;
    }

    /// Uninstall every object recorded after `key`, newest first, and forget
    /// them. Used to roll a session back to a checkpoint.
    ///
    /// The deletions happen inside the given transaction: when it aborts,
    /// the engine restores the objects and the working set must be discarded
    /// by the caller.
    pub fn revert_to(&mut self, key: u32, transaction: &Transaction<'_>) -> io::Result<()> {
        let index = if key == 0 {
            self.records.len()
        } else {
            self.records
                .iter()
                .position(|(record_key, _)| *record_key == key)
                .ok_or_else(|| {
                    io::Error::other("invalid checkpoint: it may have been overwritten")
                })?
                + 1
        };

        while self.records.len() > index {
            let (_, record) = self.records.pop().expect("len checked above");
            match record {
                Record::Provider(guid) => {
                    ignore_not_found(delete_provider(transaction, guid))?;
                }
                Record::Sublayer(guid) => {
                    ignore_not_found(delete_sublayer(transaction, guid))?;
                }
                Record::Filter(id) => {
                    ignore_not_found(delete_filter(transaction, id))?;
                }
            }
        }

        Ok(())
    }
}
