package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.applist.AppData

sealed interface SplitTunnelingUiState {
    data object Loading : SplitTunnelingUiState

    data class ShowAppList(
        val excludedApps: List<AppData> = emptyList(),
        val includedApps: List<AppData> = emptyList(),
        val showSystemApps: Boolean = false
    ) : SplitTunnelingUiState
}
