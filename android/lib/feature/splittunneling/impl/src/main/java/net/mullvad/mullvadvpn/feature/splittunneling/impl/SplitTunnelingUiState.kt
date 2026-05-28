package net.mullvad.mullvadvpn.feature.splittunneling.impl

import net.mullvad.mullvadvpn.lib.model.PackageName

data class Loading(val isModal: Boolean = false)

data class SplitTunnelingUiState(
    val enabled: Boolean = false,
    val excludedApps: List<AppItem> = emptyList(),
    val includedApps: List<AppItem> = emptyList(),
    val showSystemApps: Boolean = false,
    val isModal: Boolean = false,
)

data class AppItem(val appName: String, val packageName: PackageName)
