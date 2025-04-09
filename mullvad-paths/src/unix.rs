use std::{
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use crate::{Error, Result, UserPermissions};

pub const PRODUCT_NAME: &str = "mullvad-vpn";

impl UserPermissions {
    fn fs_permissions(self) -> fs::Permissions {
        const OWNER_BITS: u32 = 0o700;

        let rbits = if self.read { 0o044 } else { 0 };
        let wbits = if self.write { 0o022 } else { 0 };
        let ebits = if self.execute { 0o011 } else { 0 };

        std::os::unix::fs::PermissionsExt::from_mode(OWNER_BITS | rbits | wbits | ebits)
    }
}

/// Create a directory at `dir`, setting the permissions given by `permissions`, unless it exists.
/// If the directory already exists, but the permissions are not at least as strict as expected,
/// then it will be deleted and recreated.
pub fn create_dir(dir: PathBuf, permissions: Option<UserPermissions>) -> Result<PathBuf> {
    let mut dir_builder = fs::DirBuilder::new();
    let fs_perms = permissions.as_ref().map(|perms| perms.fs_permissions());
    if let Some(fs_perms) = &fs_perms {
        dir_builder.mode(fs_perms.mode());
    }
    match dir_builder.create(&dir) {
        Ok(()) => Ok(dir),
        // The directory already exists
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // Recreate the directory if the ownership and permissions are unexpected
            if !dir_is_root_owned(&dir, fs_perms.as_ref())? {
                log::debug!(
                    "Removing old directory due to unexpected permissions: {}",
                    dir.display()
                );

                fs::remove_dir_all(&dir)
                    .or_else(|err| {
                        // If the path is not a directory, try to remove the file
                        if err.kind() == io::ErrorKind::NotADirectory {
                            fs::remove_file(&dir)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|e| Error::RemoveDir(dir.display().to_string(), e))?;

                // Try to create it again
                return create_dir(dir, permissions);
            }
            // Correct permissions, so we're done
            Ok(dir)
        }
        // Fail on any other error
        Err(error) => Err(Error::CreateDirFailed(dir.display().to_string(), error)),
    }
}

/// Return whether the directory is owned by root and, optionally, is no less strict
/// than the desired permissions
fn dir_is_root_owned(dir: &Path, perms: Option<&fs::Permissions>) -> Result<bool> {
    let meta = fs::symlink_metadata(dir)
        .map_err(|e| Error::GetDirPermissionFailed(dir.display().to_string(), e))?;
    let matching_perms = perms
        .map(|perms| has_at_most_mask(meta.permissions().mode(), perms.mode()))
        .unwrap_or(true);
    Ok(matching_perms && meta.uid() == 0)
}

/// Return whether `mask` is *at least* as strict as `at_most`
/// This only considers the read, write, and exec bits.
fn has_at_most_mask(mask: u32, at_most: u32) -> bool {
    // Ignore "D" bit, setuid bit, etc.
    const RELEVANT_BITS: u32 = 0o777;
    ((mask & RELEVANT_BITS) & !(at_most & RELEVANT_BITS)) == 0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_at_most_mask() {
        assert!(!has_at_most_mask(0o777, 0o577));
        assert!(!has_at_most_mask(0o777, 0o707));
        assert!(!has_at_most_mask(0o777, 0o770));

        assert!(has_at_most_mask(0o777, 0o777));
        assert!(has_at_most_mask(0o000, 0o777));
    }
}
