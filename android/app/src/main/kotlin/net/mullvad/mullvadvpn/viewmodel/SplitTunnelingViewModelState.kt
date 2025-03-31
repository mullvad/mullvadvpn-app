package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.lib.model.AppId

data class SplitTunnelingViewModelState(
    val enabled: Boolean = false,
    val excludedApps: Set<AppId> = emptySet(),
    val allApps: List<AppData>? = null,
    val showSystemApps: Boolean = false,
) {
    fun toUiState(isModal: Boolean): SplitTunnelingUiState {
        return allApps
            ?.partition { appData ->
                if (enabled) {
                    excludedApps.contains(AppId(appData.packageName))
                } else {
                    false
                }
            }
            ?.let { (excluded, included) ->
                SplitTunnelingUiState.ShowAppList(
                    enabled = enabled,
                    excludedApps = excluded.sort(),
                    includedApps =
                        if (showSystemApps) {
                                included
                            } else {
                                included.filter { appData -> !appData.isSystemApp }
                            }
                            .sort(),
                    showSystemApps = showSystemApps,
                    isModal = isModal,
                )
            } ?: SplitTunnelingUiState.Loading(enabled = enabled, isModal)
    }
}

private fun List<AppData>.sort() = sortedBy { it.name.lowercase() }
