package net.mullvad.mullvadvpn.feature.splittunneling.impl

import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.AppData

data class Loading(val enabled: Boolean = false, val isModal: Boolean = false)

data class SplitTunnelingUiState(
    val enabled: Boolean = false,
    val excludedApps: List<AppData> = emptyList(),
    val includedApps: List<AppData> = emptyList(),
    val showSystemApps: Boolean = false,
    val isModal: Boolean = false,
)
