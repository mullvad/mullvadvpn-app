package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.model.SearchMatch

data class SearchSplitTunnelingUiState(
    val searchTerm: String,
    val excludedApps: List<SearchAppItem> = emptyList(),
    val includedApps: List<SearchAppItem> = emptyList(),
)

sealed interface SearchAppItem {
    val packageName: PackageName

    data class Match(val match: SearchMatch, override val packageName: PackageName) : SearchAppItem

    data class Default(val appName: String, override val packageName: PackageName) : SearchAppItem
}
