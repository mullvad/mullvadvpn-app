package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.applist.AppData

sealed interface SplitTunnelingUiState {
    object Loading : SplitTunnelingUiState
    data class Data(
        val excludedApps: List<AppData> = emptyList(),
        val includedApps: List<AppData> = emptyList(),
        val showSystemApps: Boolean = false
    ) : SplitTunnelingUiState
}
