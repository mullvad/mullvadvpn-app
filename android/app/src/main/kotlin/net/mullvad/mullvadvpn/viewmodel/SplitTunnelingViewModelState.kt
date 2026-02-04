package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc
import net.mullvad.mullvadvpn.lib.model.AppId

data class SplitTunnelingViewModelState(
    val enabled: Boolean = false,
    val excludedApps: Set<AppId> = emptySet(),
    val allApps: List<AppData>? = null,
    val showSystemApps: Boolean = false,
) {
    fun toUiState(isModal: Boolean): Lc<Loading, SplitTunnelingUiState> {
        return allApps
            ?.partition { appData ->
                if (enabled) {
                    excludedApps.contains(AppId(appData.packageName))
                } else {
                    false
                }
            }
            ?.let { (excluded, included) ->
                SplitTunnelingUiState(
                        enabled = enabled,
                        excludedApps = excluded.sortedWith(descendingByNameComparator),
                        includedApps =
                            if (showSystemApps) {
                                    included
                                } else {
                                    included.filter { appData -> !appData.isSystemApp }
                                }
                                .sortedWith(descendingByNameComparator),
                        showSystemApps = showSystemApps,
                        isModal = isModal,
                    )
                    .toLc()
            } ?: Lc.Loading(Loading(enabled = enabled, isModal))
    }

    companion object {
        private val descendingByNameComparator = compareBy<AppData> { it.name.lowercase() }
    }
}
