package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.state.AppListState
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState

data class SplitTunnelingViewModelState(
    val checked: Boolean = false,
    val excludedApps: Set<String> = emptySet(),
    val allApps: List<AppData>? = null,
    val showSystemApps: Boolean = false
) {
    fun toUiState(): SplitTunnelingUiState {
        return if (checked) {
            allApps
                ?.partition { appData -> excludedApps.contains(appData.packageName) }
                ?.let { (excluded, included) ->
                    SplitTunnelingUiState(
                        checked = true,
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = excluded.sortedBy { it.name },
                                includedApps =
                                    if (showSystemApps) included
                                    else included.filter { !it.isSystemApp }.sortedBy { it.name },
                                showSystemApps = showSystemApps
                            )
                    )
                }
                ?: SplitTunnelingUiState(checked = true, appListState = AppListState.Loading)
        } else {
            SplitTunnelingUiState(checked = false, appListState = AppListState.Disabled)
        }
    }
}
