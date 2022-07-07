package net.mullvad.mullvadvpn.ui

data class VersionInfo(
    val currentVersion: String?,
    val upgradeVersion: String?,
    val isOutdated: Boolean,
    val isSupported: Boolean
)
