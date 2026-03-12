package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CustomListsNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists.CustomLists
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists.CustomListsScreen

fun EntryProviderScope<NavKey>.customListsEntry(navigator: Navigator) {
    entry<CustomListsNavKey> {
//        CustomLists(navigator = navigator)
    }
}
