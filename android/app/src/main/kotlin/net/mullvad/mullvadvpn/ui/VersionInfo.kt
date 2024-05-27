package net.mullvad.mullvadvpn.ui

import net.mullvad.mullvadvpn.BuildConfig

data class VersionInfo(
    val currentVersion: String = BuildConfig.VERSION_NAME,
    val isSupported: Boolean,
    val suggestedUpgradeVersion: String?
) {
    val isUpdateAvailable: Boolean = suggestedUpgradeVersion != null
}
