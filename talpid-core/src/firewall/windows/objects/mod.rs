//! The WFP objects owned by the app, managed by talking to the Windows Filtering Platform
//! directly rather than going through the `winfw` C++ library.

mod guids;
mod persistent;

pub(super) use persistent::apply_persistent_blocking;
