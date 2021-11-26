use std::{ffi::CStr, io};
/// name of the group that should be excluded
const EXCLUSION_GROUP: &[u8] = b"mullvad-exclusion\0";

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_exclusion_gid() -> io::Result<u32> {
    let exclusion_group_name = unsafe { CStr::from_bytes_with_nul_unchecked(EXCLUSION_GROUP) };
    talpid_core::macos::get_group_id(exclusion_group_name)
}

/// Attempts to set the GID of the current process to `mullvad-exclusion`.
pub fn set_exclusion_gid() -> io::Result<u32> {
    let gid = get_exclusion_gid()?;
    talpid_core::macos::set_gid(gid)?;
    Ok(gid)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_exclusion_gid() {
        let _ = super::get_exclusion_gid();
    }
}
