use std::ffi::CStr;
/// name of the group that should be excluded
const EXCLUSION_GROUP: &[u8] = b"mullvad-exclusion\0";

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_exclusion_gid() -> Option<u32> {
    let exclusion_group_name = unsafe { CStr::from_bytes_with_nul_unchecked(EXCLUSION_GROUP) };
    talpid_core::macos::get_group_id(exclusion_group_name)
}

/// Attempts to set the GID of the current process to `mullvad-exclusion`.
#[cfg(target_os = "macos")]
pub fn set_exclusion_gid() {
    if let Some(gid) = get_exclusion_gid() {
        if let Err(err) = talpid_core::macos::set_gid(gid) {
            log::error!("Failed to set group ID: {}", err);
        }
    } else {
        log::error!("No exclusion ID available");
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_exclusion_gid() {
        let _ = super::get_exclusion_gid();
    }
}
