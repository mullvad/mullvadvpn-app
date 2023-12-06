package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.state.AppListState
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState

data class SplitTunnelingViewModelState(
    val enabled: Boolean = false,
    val excludedApps: Set<String> = emptySet(),
    val allApps: List<AppData>? = null,
    val showSystemApps: Boolean = false
) {
    fun toUiState(): SplitTunnelingUiState {
        return if (enabled) {
            allApps
                ?.partition { appData -> excludedApps.contains(appData.packageName) }
                ?.let { (excluded, included) ->
                    SplitTunnelingUiState(
                        enabled = true,
                        appListState =
                            AppListState.ShowAppList(
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
                    )
                } ?: SplitTunnelingUiState(enabled = true, appListState = AppListState.Disabled)
        } else {
            SplitTunnelingUiState(enabled = false, appListState = AppListState.Disabled)
        }
    }
}
