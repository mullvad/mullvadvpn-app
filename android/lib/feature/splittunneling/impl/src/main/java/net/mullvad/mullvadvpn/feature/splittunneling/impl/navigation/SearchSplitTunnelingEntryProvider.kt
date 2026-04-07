package net.mullvad.mullvadvpn.feature.splittunneling.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.splittunneling.api.SearchSplitTunnelingNavKey
import net.mullvad.mullvadvpn.feature.splittunneling.impl.search.SearchSplitTunnelingScreen

fun EntryProviderScope<NavKey2>.searchSplitTunnelingEntry(navigator: Navigator) {
    entry<SearchSplitTunnelingNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { _ ->
        SearchSplitTunnelingScreen(navigator = navigator)
    }
}
