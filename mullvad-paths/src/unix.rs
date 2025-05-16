use std::{
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use crate::{Error, Result};

pub const PRODUCT_NAME: &str = "mullvad-vpn";

#[derive(Clone, Copy, PartialEq)]
pub enum Permissions {
    /// Do not set any particular permissions. They will be inherited instead.
    Any,
    /// Only root should have write access. Other users will have
    /// read and execute permissions (0o755).
    ReadExecOnly,
}

impl Permissions {
    fn fs_permissions(self) -> Option<fs::Permissions> {
        match self {
            Permissions::Any => None,
            Permissions::ReadExecOnly => Some(std::os::unix::fs::PermissionsExt::from_mode(0o755)),
        }
    }
}

pub fn create_and_return(dir: PathBuf, permissions: Permissions) -> Result<PathBuf> {
    let mut dir_builder = fs::DirBuilder::new();
    let fs_perms = permissions.fs_permissions();
    if let Some(fs_perms) = fs_perms.as_ref() {
        dir_builder.mode(fs_perms.mode());
    }
    match dir_builder.create(&dir) {
        Ok(()) => Ok(dir),
        // The directory already exists
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // If the permissions are wrong, delete the directory and recreate it
            if !dir_is_root_owned(&dir, fs_perms.as_ref())? {
                fs::remove_dir_all(&dir)
                    .or_else(|err| {
                        // ENOTDIR: If the path is not a directory, try to remove the file
                        if err.raw_os_error() == Some(20) {
                            fs::remove_file(&dir)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|e| Error::RemoveDir(dir.display().to_string(), e))?;

                // Try to create it again
                return create_and_return(dir, permissions);
            }
            // Correct permissions, so we're done
            Ok(dir)
        }
        // Fail on any other error
        Err(error) => Err(Error::CreateDirFailed(dir.display().to_string(), error)),
    }
}

/// Return whether the directofy is owned by root and, optionally, has the given permissions set
fn dir_is_root_owned(dir: &Path, perms: Option<&fs::Permissions>) -> Result<bool> {
    const RELEVANT_BITS: u32 = 0o777;

    let meta = fs::symlink_metadata(&dir)
        .map_err(|e| Error::GetDirPermissionFailed(dir.display().to_string(), e))?;
    let matching_perms = perms
        .map(|perms| (perms.mode() & RELEVANT_BITS) == (meta.permissions().mode() & RELEVANT_BITS))
        .unwrap_or(true);
    Ok(matching_perms && meta.uid() == 0)
}
