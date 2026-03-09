package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.customlist.api.CustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists.CustomLists

fun EntryProviderScope<NavKey2>.customListEntry(navigator: Navigator) {
    entry<CustomListNavKey>(metadata = slideInHorizontalTransition()) {
        CustomLists(navigator = navigator)
    }

    createCustomListEntry(navigator)
    deleteCustomListEntry(navigator)
    editCustomListEntry(navigator)
    editCustomListLocationsEntry(navigator)
    editCustomListNameEntry(navigator)
    discardCustomListChangesEntry(navigator)
}
