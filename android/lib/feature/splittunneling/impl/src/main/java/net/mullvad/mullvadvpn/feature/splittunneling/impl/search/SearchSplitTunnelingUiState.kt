package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.AppData

data class SearchSplitTunnelingUiState(
    val searchTerm: String,
    val excludedApps: List<AppData> = emptyList(),
    val includedApps: List<AppData> = emptyList(),
)
