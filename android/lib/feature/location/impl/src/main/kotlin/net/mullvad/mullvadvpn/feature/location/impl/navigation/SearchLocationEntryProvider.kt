package net.mullvad.mullvadvpn.feature.location.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.location.api.SearchLocationNavKey
import net.mullvad.mullvadvpn.feature.location.impl.search.SearchLocation

internal fun EntryProviderScope<NavKey2>.searchLocationEntry(navigator: Navigator) {
    entry<SearchLocationNavKey> { navKey ->
        SearchLocation(relayListType = navKey.relayListType, navigator = navigator)
    }
}
