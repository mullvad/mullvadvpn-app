//! The WFP objects owned by the app that outlive the `winfw` firewall: the persistent
//! and boot-time objects that keep blocking after the daemon has shut down.

mod persistent;

pub(super) use persistent::apply_persistent_blocking;
