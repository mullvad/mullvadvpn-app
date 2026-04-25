package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

import net.mullvad.mullvadvpn.feature.splittunneling.impl.AppItem

data class SearchSplitTunnelingUiState(
    val searchTerm: String,
    val excludedApps: List<AppItem> = emptyList(),
    val includedApps: List<AppItem> = emptyList(),
)
