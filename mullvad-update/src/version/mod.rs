//! This module is used to extract the latest versions out of a raw [format::Response] using a query
//! [VersionParameters]. It also contains additional logic for filtering and validating the raw
//! deserialized response.
//!
//! The main input here is [VersionParameters], and the main output is [VersionInfo].
mod info;
mod parameters;
mod rollout;

pub use info::{MIN_VERIFY_METADATA_VERSION, Metadata, VersionInfo, is_version_supported};
pub use parameters::VersionParameters;
pub use rollout::{IGNORE, Rollout, is_complete_rollout};

pub use crate::format::Architecture;
pub use crate::format::installer::Installer;
pub use crate::format::release::Release;
pub use crate::format::response::Response;
