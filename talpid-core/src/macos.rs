use std::ffi::CStr;

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_group_id(group_name: &CStr) -> Option<u32> {
    let group = unsafe { libc::getgrnam(group_name.as_ptr() as *const _) };
    if group.is_null() {
        return None;
    }
    let gid = unsafe { (*group).gr_gid };
    Some(gid)
}
