package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState

data class SplitTunnelingViewModelState(
    val excludedApps: Set<String> = emptySet(),
    val allApps: List<AppData>? = null,
    val showSystemApps: Boolean = false
) {
    fun toUiState(): SplitTunnelingUiState {
        return allApps
            ?.partition { appData -> excludedApps.contains(appData.packageName) }
            ?.let { (excluded, included) ->
                SplitTunnelingUiState.ShowAppList(
                    excludedApps = excluded.sortedBy { it.name },
                    includedApps =
                        if (showSystemApps) {
                                included
                            } else {
                                included.filter { appData -> !appData.isSystemApp }
                            }
                            .sortedBy { it.name },
                    showSystemApps = showSystemApps
                )
            } ?: SplitTunnelingUiState.Loading
    }
}
