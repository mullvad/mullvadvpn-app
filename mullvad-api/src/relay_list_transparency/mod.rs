//! This module contains functions for parsing and validating the Sigsum transparency log
//! signatures and timestamps that are available at the `/trl/v0/timestamps/latest` and
//! `/trl/v0/data/` endpoints.

mod relay_list;
mod test;
mod validate;

pub(crate) use relay_list::download_and_verify_relay_list;
pub use relay_list::{RelayListDigest, RelayListSignature, Sha256Bytes};
pub use validate::parse_pubkeys;
