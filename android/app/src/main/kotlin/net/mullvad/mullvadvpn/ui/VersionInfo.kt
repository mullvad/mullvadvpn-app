package net.mullvad.mullvadvpn.ui

data class VersionInfo(
    @Deprecated(message = "Use BuildConfig.VERSION_NAME") val currentVersion: String?,
    val upgradeVersion: String?,
    val isOutdated: Boolean,
    val isSupported: Boolean
)
