//! Test manager configuration.

mod error;
mod io;
mod manifest;
mod vm;

use error::Error;
pub use io::ConfigFile;
pub use manifest::{Config, Display, EnvironmentConfig};
pub use vm::{Architecture, OsType, PackageType, Provisioner, VmConfig, VmType};
