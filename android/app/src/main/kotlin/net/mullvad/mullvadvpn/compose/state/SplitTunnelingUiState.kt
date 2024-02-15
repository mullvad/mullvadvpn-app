package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.applist.AppData

sealed interface SplitTunnelingUiState {
    val enabled: Boolean

    data class Loading(override val enabled: Boolean = false) : SplitTunnelingUiState

    data class ShowAppList(
        override val enabled: Boolean = false,
        val excludedApps: List<AppData> = emptyList(),
        val includedApps: List<AppData> = emptyList(),
        val showSystemApps: Boolean = false
    ) : SplitTunnelingUiState
}
