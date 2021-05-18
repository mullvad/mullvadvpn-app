/// name of the group that should be excluded
const EXCLUSION_GROUP: &[u8] = b"mullvad-exclusion\0";

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_exclusion_gid() -> Option<u32> {
    let group = unsafe { libc::getgrnam(EXCLUSION_GROUP.as_ptr() as *const _) };
    if group.is_null() {
        return None;
    }
    let gid = unsafe { (*group).gr_gid };
    Some(gid)
}
