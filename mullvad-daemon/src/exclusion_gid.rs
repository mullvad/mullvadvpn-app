use std::{ffi::CStr, io};
/// name of the group that should be excluded
const EXCLUSION_GROUP: &[u8] = b"mullvad-exclusion\0";

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_exclusion_gid() -> io::Result<u32> {
    let exclusion_group_name = CStr::from_bytes_with_nul(EXCLUSION_GROUP).unwrap();
    get_group_id(exclusion_group_name)
}

/// Attempts to set the GID of the current process to `mullvad-exclusion`.
pub fn set_exclusion_gid() -> io::Result<u32> {
    let gid = get_exclusion_gid()?;
    set_gid(gid)?;
    Ok(gid)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_exclusion_gid() {
        let _ = super::get_exclusion_gid();
    }
}

/// Returns the GID of the specified group name
fn get_group_id(group_name: &CStr) -> io::Result<u32> {
    // SAFETY: group_name is a valid CString
    let group = unsafe { libc::getgrnam(group_name.as_ptr() as *const _) };
    if group.is_null() {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }
    // SAFETY: group is not null
    let gid = unsafe { (*group).gr_gid };
    Ok(gid)
}

/// Sets group ID for the current process
fn set_gid(gid: u32) -> io::Result<()> {
    if unsafe { libc::setgid(gid) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
#[test]
fn test_unknown_group() {
    let unknown_group = CStr::from_bytes_with_nul(b"asdunknown\0").unwrap();
    let group_err = get_group_id(unknown_group).unwrap_err();
    assert!(group_err.kind() == io::ErrorKind::NotFound)
}
