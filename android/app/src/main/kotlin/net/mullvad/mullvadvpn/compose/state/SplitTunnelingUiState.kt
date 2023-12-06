package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.applist.AppData

data class SplitTunnelingUiState(
    val enabled: Boolean = false,
    val appListState: AppListState = AppListState.Disabled
)

sealed interface AppListState {
    data object Disabled : AppListState

    data object Loading : AppListState

    data class ShowAppList(
        val excludedApps: List<AppData> = emptyList(),
        val includedApps: List<AppData> = emptyList(),
        val showSystemApps: Boolean = false
    ) : AppListState
}
