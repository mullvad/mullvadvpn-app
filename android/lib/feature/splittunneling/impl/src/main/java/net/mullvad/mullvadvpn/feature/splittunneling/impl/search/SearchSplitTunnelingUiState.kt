package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

import net.mullvad.mullvadvpn.lib.model.HighlightedString
import net.mullvad.mullvadvpn.lib.model.PackageName

data class SearchSplitTunnelingUiState(
    val searchTerm: String,
    val excludedApps: List<SearchAppItem> = emptyList(),
    val includedApps: List<SearchAppItem> = emptyList(),
)

sealed interface SearchAppItem {
    val packageName: PackageName

    data class Match(val appName: HighlightedString, override val packageName: PackageName) :
        SearchAppItem

    data class Default(val appName: String, override val packageName: PackageName) : SearchAppItem
}
