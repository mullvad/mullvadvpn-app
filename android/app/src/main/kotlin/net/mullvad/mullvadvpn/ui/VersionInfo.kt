package net.mullvad.mullvadvpn.ui

import net.mullvad.mullvadvpn.BuildConfig

data class VersionInfo(
    val suggestedUpgradeVersion: String?,
    val isSupported: Boolean,
    val currentVersion: String = BuildConfig.VERSION_NAME
) {
    val isOutdated: Boolean = suggestedUpgradeVersion != null
}
