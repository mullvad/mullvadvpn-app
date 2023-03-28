use crate::types;
use std::path::PathBuf;
use talpid_types::split_tunnel::ExcludedProcess;

impl From<ExcludedProcess> for types::ExcludedProcess {
    fn from(value: ExcludedProcess) -> Self {
        types::ExcludedProcess {
            image: value.image.to_string_lossy().into_owned(),
            inherited: value.inherited,
            pid: value.pid,
        }
    }
}

impl From<types::ExcludedProcess> for ExcludedProcess {
    fn from(value: types::ExcludedProcess) -> Self {
        ExcludedProcess {
            image: PathBuf::from(value.image),
            inherited: value.inherited,
            pid: value.pid,
        }
    }
}
