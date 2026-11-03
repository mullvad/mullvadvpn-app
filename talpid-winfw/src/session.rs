//! Tracking of WFP objects added within a session, with checkpoint/revert.
//!
//! The Windows Filtering Platform has no notion of rolling back a set of
//! objects, so the firewall records every object it adds. `revert_to`
//! uninstalls everything added after a checkpoint, newest first.

use std::io;

use wfp::{Guid, Transaction, delete_filter, delete_provider, delete_sublayer};

use crate::objects::ignore_not_found;

/// A WFP object added by the firewall.
#[derive(Clone, Copy, Debug)]
pub enum Record {
    Provider(Guid),
    Sublayer(Guid),
    Filter(u64),
}

/// The records of one firewall session.
#[derive(Default)]
pub struct Recorder {
    records: Vec<Record>,
}

impl Recorder {
    /// Record an added object.
    pub fn record(&mut self, record: Record) {
        self.records.push(record);
    }

    /// The number of objects recorded so far, for passing to
    /// [`Recorder::revert_to`].
    pub fn checkpoint(&self) -> usize {
        self.records.len()
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

    /// Uninstall every object recorded after `checkpoint`, newest first, and
    /// forget them. Used to roll a session back to a checkpoint.
    ///
    /// The deletions happen inside the given transaction: when it aborts,
    /// the engine restores the objects and the working set must be discarded
    /// by the caller.
    pub fn revert_to(
        &mut self,
        checkpoint: usize,
        transaction: &Transaction<'_>,
    ) -> io::Result<()> {
        while self.records.len() > checkpoint {
            let record = self.records.pop().expect("len checked above");
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
