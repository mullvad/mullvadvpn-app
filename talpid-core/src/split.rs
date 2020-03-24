use std::{
    fs,
    io::{self, Write},
    path::Path,
};

const NETCLS_DIR: &str = "/sys/fs/cgroup/net_cls/";
/// Identifies packets coming from the cgroup.
const NETCLS_CLASSID: u32 = 0x4d9f41;
const CGROUP_NAME: &str = "mullvad-exclusions";

/// Errors related to split tunneling.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Unable to create cgroup.
    #[error(display = "Unable to create cgroup for excluded processes")]
    CreateCGroup(#[error(source)] io::Error),

    /// Unable to set class ID for cgroup.
    #[error(display = "Unable to set cgroup class ID")]
    SetCGroupClassId(#[error(source)] io::Error),
}

/// Set up cgroup used to track PIDs for split tunneling.
pub fn create_cgroup() -> Result<(), Error> {
    let exclusions_dir = Path::new(NETCLS_DIR).join(CGROUP_NAME);

    if !exclusions_dir.exists() {
        fs::create_dir(exclusions_dir.clone()).map_err(Error::CreateCGroup)?;
    }

    let classid_path = exclusions_dir.join("net_cls.classid");
    fs::write(classid_path, NETCLS_CLASSID.to_string().as_bytes()).map_err(Error::SetCGroupClassId)
}
