package net.mullvad.mullvadvpn.dataproxy

class AppVersionInfoCache {
    companion object {
        val KEY_CURRENT_IS_SUPPORTED = "current_is_supported"
        val KEY_LAST_UPDATED = "last_updated"
        val KEY_LATEST_STABLE = "latest_stable"
        val KEY_LATEST = "latest"
        val SHARED_PREFERENCES = "app_version_info_cache"
    }
}
